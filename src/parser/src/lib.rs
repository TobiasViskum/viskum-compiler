use std::time::Instant;

use ast::{
    Ast,
    AstArena,
    BlockExpr,
    Expr,
    ExprWithBlock,
    FunctionStmt,
    GlobalScope,
    IdentExpr,
    IfExpr,
    IfFalseExpr,
    IntegerExpr,
    ItemStmt,
    Stmt,
};
use ast_state::AstUnvalidated;
use expr_builder::ExprBuilder;
use lexer::Lexer;
use make_parse_rule::make_parse_rule;
use op::BinaryOp;
use precedence::Precedence;
use span::Span;
use token::{ Token, TokenKind };
mod make_parse_rule;

const PARSE_RULE_COUNT: usize = enum_iterator::cardinality::<TokenKind>();

pub type ParseRuleMethod = for<'a, 'b> fn(&'b mut Parser<'a>, &'b mut ExprBuilder<'a>);

#[derive(Debug)]
pub struct ParseRule {
    pub prefix_method: Option<ParseRuleMethod>,
    pub prefix_prec: Precedence,
    pub infix_method: Option<ParseRuleMethod>,
    pub infix_prec: Precedence,
    pub postfix_method: Option<ParseRuleMethod>,
    pub postfix_prec: Precedence,
}

pub struct Parser<'a> {
    parse_rules: [ParseRule; PARSE_RULE_COUNT],
    lexer: Lexer<'a>,
    ast_arena: &'a AstArena<'a>,
    src: &'a str,
    current: Token,
    prev: Token,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str, ast_arena: &'a AstArena<'a>) -> Self {
        let mut lexer = Lexer::new(src);

        Self {
            current: lexer.scan_token(),
            src,
            parse_rules: Self::create_parse_rules(),
            ast_arena,
            lexer,
            prev: Token::dummy(),
        }
    }

    pub fn parse_into_ast(mut self) -> Ast<'a, AstUnvalidated> {
        let global_scope = self.parse_global_scope();
        Ast::new(global_scope, self.ast_arena)
    }

    pub(crate) fn statement(&mut self) -> Stmt<'a, AstUnvalidated> {
        match self.current.get_kind() {
            TokenKind::Def => self.function_statement(),
            _ => self.expression_statement(),
        }
    }

    pub(crate) fn function_statement(&mut self) -> Stmt<'a, AstUnvalidated> {
        self.advance();

        let ident_expr = self.consume_ident("Expected ident after `def`");

        self.consume(TokenKind::LeftParen, "Expected '(' after function name");
        // Args here
        self.consume(TokenKind::RightParen, "Expected ')' after function name");

        let body = self.parse_block();

        self.consume(TokenKind::End, "Expected `end` after function def");

        Stmt::ItemStmt(
            ItemStmt::FunctionStmt(
                FunctionStmt::new(ident_expr, Expr::ExprWithBlock(ExprWithBlock::BlockExpr(body)))
            )
        )
    }

    pub(crate) fn expression_statement(&mut self) -> Stmt<'a, AstUnvalidated> {
        let mut expr_builder = ExprBuilder::new(self.ast_arena);
        self.parse_precedence(expr_builder.get_base_prec(), &mut expr_builder);
        let stmt = expr_builder.take_stmt().expect("TODO: Error handling");
        stmt
    }

    pub(crate) fn parse_precedence(
        &mut self,
        mut prec: Precedence,
        expr_builder: &mut ExprBuilder<'a>
    ) {
        self.advance();

        let parse_rule = self.get_parse_rule_of_prev();

        if let Some(prefix_method) = parse_rule.prefix_method {
            prefix_method(self, expr_builder);

            while prec <= self.get_parse_rule_of_current().infix_prec {
                // Now we don't want to parse another `:=` or `=` in expr without block
                if self.current.get_kind().has_assign_prec() {
                    prec = Precedence::PrecAssign.get_next();
                }

                self.advance();

                if let Some(infix_rule) = self.get_parse_rule_of_prev().infix_method {
                    infix_rule(self, expr_builder);
                }
            }
        } else {
            panic!("Unexpected token: {}", self.prev.get_kind())
        }
    }

    /// Parse rule method: `define`
    pub(crate) fn define(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        expr_builder.set_base_prec(Precedence::PrecAssign.get_next());
        self.parse_precedence(expr_builder.get_base_prec(), expr_builder);
        expr_builder.emit_define_stmt()
    }

    /// Parse rule method: `assign`
    pub(crate) fn assign(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        expr_builder.set_base_prec(Precedence::PrecAssign.get_next());
        self.parse_precedence(expr_builder.get_base_prec(), expr_builder);
        expr_builder.emit_assign_stmt()
    }

    /// Parse rule method: `block`
    pub(crate) fn block_expr(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let block_expr = self.parse_block();
        self.consume(TokenKind::End, "Expected `end`");
        expr_builder.emit_block_expr(block_expr);
    }

    /// Parse rule method: `grouping`
    pub(crate) fn grouping(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.parse_precedence(expr_builder.get_base_prec(), expr_builder);
        self.consume(TokenKind::RightParen, "Expected ')' after group");
        expr_builder.emit_grouping_expr()
    }

    /// Parse rule method: `ident`
    pub(crate) fn ident(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        expr_builder.emit_ident_expr(IdentExpr::new(self.prev.get_span()));
    }

    /// Parse rule method: `integer`
    pub(crate) fn integer(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let lexeme = self.get_lexeme_of_prev();
        let val = lexeme.parse::<i32>().expect("TODO: Error handling");
        expr_builder.emit_integer_expr(IntegerExpr::new(val));
    }

    /// Parse rule method: `dot_float`
    pub(crate) fn dot_float(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        todo!()
    }

    /// Parse rule method: `float`
    pub(crate) fn float(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        todo!()
    }

    /// Parse rule method: `add`
    pub(crate) fn add(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.binary(expr_builder, BinaryOp::Add)
    }

    /// Parse rule method: `sub`
    pub(crate) fn sub(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.binary(expr_builder, BinaryOp::Sub)
    }

    /// Parse rule method: `mul`
    pub(crate) fn mul(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.binary(expr_builder, BinaryOp::Mul)
    }

    /// Parse rule method: `div`
    pub(crate) fn div(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.binary(expr_builder, BinaryOp::Div)
    }

    /// Logic of binary parse rule methods
    pub(crate) fn binary(&mut self, expr_builder: &mut ExprBuilder<'a>, binary_op: BinaryOp) {
        self.parse_precedence(self.get_parse_rule_of_prev().infix_prec.get_next(), expr_builder);

        expr_builder.emit_binary_expr(binary_op)
    }

    /// Parse rule method: `div`
    pub(crate) fn if_expr(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let if_expr = self.parse_if_expr();

        expr_builder.emit_if_expr(if_expr)
    }

    pub(crate) fn parse_if_expr(&mut self) -> IfExpr<'a, AstUnvalidated> {
        let mut expr_builder = ExprBuilder::new(self.ast_arena);
        self.parse_precedence(expr_builder.get_base_prec(), &mut expr_builder);
        self.consume(TokenKind::Do, "Expected 'do' after if-condition");

        let condition = self.ast_arena.alloc_expr(
            expr_builder.take_expr().expect("TODO:Error handling")
        );
        let true_block = self.parse_block();

        self.advance_if(matches!(self.current.get_kind(), TokenKind::Else | TokenKind::Elif));
        let false_block = match self.prev.get_kind() {
            TokenKind::Else => Some(IfFalseExpr::ElseExpr(self.parse_block())),
            TokenKind::Elif => Some(IfFalseExpr::ElifExpr(self.parse_and_alloc_if_expr())),
            _ => None,
        };

        self.consume(TokenKind::End, "Expected `end` after if expression");

        IfExpr::new(condition, true_block, false_block)
    }

    pub(crate) fn parse_and_alloc_if_expr(&mut self) -> &'a mut IfExpr<'a, AstUnvalidated> {
        self.ast_arena.alloc_if_expr(self.parse_if_expr())
    }

    fn parse_block(&mut self) -> BlockExpr<'a, AstUnvalidated> {
        let mut stmts = Vec::with_capacity(8);

        while !self.current.get_kind().can_end_scope() && !self.is_eof() {
            let stmt = self.statement();
            stmts.push(stmt);
        }

        BlockExpr::new(stmts)
    }

    fn parse_global_scope(&mut self) -> GlobalScope<'a, AstUnvalidated> {
        let mut stmts = Vec::with_capacity(8);

        while !self.is_eof() {
            let stmt = self.statement();
            stmts.push(stmt);
        }

        GlobalScope::new(stmts)
    }

    /* Helper methods */

    /// Converts e.g. `69` into `0.69` (this is a fast version by chat)
    pub(crate) fn integer_to_dot_float(int: i64) -> f64 {
        // Compute the number of digits using logarithms
        let num_digits = if int == 0 { 1 } else { (int.abs() as f64).log10().ceil() as u32 };

        // Compute the divisor as 10^num_digits
        let divisor = (10_i64).pow(num_digits);

        // Perform the division
        let result = (int as f64) / (divisor as f64);

        result
    }

    pub(crate) fn get_lexeme_of_prev(&self) -> &str {
        self.get_lexeme(self.prev.get_span())
    }

    pub(crate) fn is_eof(&self) -> bool {
        self.lexer.is_eof() && self.current.get_kind() == TokenKind::Eof
    }

    pub(crate) fn get_lexeme(&self, span: Span) -> &str {
        &self.src[span.get_byte_start()..span.get_byte_end()]
    }

    pub(crate) fn advance(&mut self) {
        self.prev = self.current;
        self.current = self.lexer.scan_token();
    }

    pub(crate) fn advance_if(&mut self, cond: bool) {
        if cond {
            self.advance()
        }
    }

    pub(crate) fn consume(&mut self, kind: TokenKind, err_msg: &str) {
        if self.current.get_kind() == kind {
            self.advance();
        } else {
            panic!("Error: {}", err_msg);

            // self.advance();
        }
    }

    pub(crate) fn consume_ident(&mut self, err_msg: &str) -> IdentExpr<AstUnvalidated> {
        match self.current.get_kind() {
            TokenKind::Ident => {
                let ident_expr = IdentExpr::new(self.current.get_span());
                self.advance();
                ident_expr
            }
            _ => panic!("{}", err_msg),
        }
    }

    pub(crate) fn get_parse_rule_of_current(&self) -> &ParseRule {
        &self.parse_rules[self.current.get_kind() as usize]
    }

    pub(crate) fn get_parse_rule_of_prev(&self) -> &ParseRule {
        &self.parse_rules[self.prev.get_kind() as usize]
    }

    pub(crate) fn create_parse_rules() -> [ParseRule; PARSE_RULE_COUNT] {
        array_init::array_init(|i| {
            let kind = enum_iterator::all::<TokenKind>().nth(i).unwrap();
            Self::create_parse_rule(kind)
        })
    }

    pub(crate) fn create_parse_rule(kind: TokenKind) -> ParseRule {
        make_parse_rule!(kind,
    /*  TOKENKIND        INFIX                  PREFIX                          POSTFIX             */
    /*                   method     prec        method      prec                method      prec    */
        LeftParen   = { (grouping   None),      (None       None        ),      (None       None) },
        RightParen  = { (None       None),      (None       None        ),      (None       None) },
        Eq          = { (None       None),      (None       None        ),      (None       None) },
        Ne          = { (None       None),      (None       None        ),      (None       None) },
        Ge          = { (None       None),      (None       None        ),      (None       None) },
        Gt          = { (None       None),      (None       None        ),      (None       None) },
        Le          = { (None       None),      (None       None        ),      (None       None) },
        Lt          = { (None       None),      (None       None        ),      (None       None) },
        Plus        = { (None       None),      (add        PrecTerm    ),      (None       None) },
        Minus       = { (None       None),      (sub        PrecTerm    ),      (None       None) },
        Star        = { (None       None),      (mul        PrecFactor  ),      (None       None) },
        Slash       = { (None       None),      (div        PrecFactor  ),      (None       None) },
        Colon       = { (None       None),      (None       None        ),      (None       None) },
        Define      = { (None       None),      (define     PrecAssign  ),      (None       None) },
        Assign      = { (None       None),      (assign     PrecAssign  ),      (None       None) },
        Dot         = { (dot_float  None),      (None       None        ),      (None       None)},
            
            
        // Numbers
        Integer     = { (integer    None),      (None       None        ),      (None       None) },
        Float       = { (float      None),      (None       None        ),      (None       None) },
            
        // Identifier
        Ident       = { (ident      None),      (None       None        ),      (None       None) },

        // Keywords
        Def         = { (None       None),      (None       None        ),      (None       None) },
        Class       = { (None       None),      (None       None        ),      (None       None) },
        While       = { (None       None),      (None       None        ),      (None       None) },
        If          = { (if_expr    None),      (None       None        ),      (None       None) },
        Loop        = { (None       None),      (None       None        ),      (None       None) },
        Break       = { (None       None),      (None       None        ),      (None       None) },
        Do          = { (block_expr None),      (None       None        ),      (None       None) },
        Else        = { (None       None),      (None       None        ),      (None       None) },
        Elif        = { (None       None),      (None       None        ),      (None       None) },
        End         = { (None       None),      (None       None        ),      (None       None) },
        Eof         = { (None       None),      (None       None        ),      (None       None) }
        )
    }
}
