use ast::{
    Ast,
    AstArena,
    AstQuerySystem,
    AstState0,
    BlockExpr,
    BoolExpr,
    BreakExpr,
    ContinueExpr,
    Expr,
    ExprKind,
    ExprWithBlock,
    ExprWithoutBlock,
    FunctionStmt,
    GlobalScope,
    IdentExpr,
    IfExpr,
    IfFalseBranchExpr,
    IntegerExpr,
    ItemStmt,
    LoopExpr,
    Stmt,
};
use error::Error;
use expr_builder::ExprBuilder;
use ir_defs::{ NodeId, ResultLoc };
use lexer::Lexer;
use make_parse_rule::make_parse_rule;
use op::{ ArithmeticOp, BinaryOp, ComparisonOp };
use precedence::Precedence;
use span::Span;
use token::{ Token, TokenKind };
mod make_parse_rule;
mod expr_builder;
mod precedence;

const PARSE_RULE_COUNT: usize = enum_iterator::cardinality::<TokenKind>();

pub type ParseRuleMethod = for<'a, 'b> fn(&'b mut Parser<'a>, &'b mut ExprBuilder<'a>);

/// Each token kind is asociated with a ParseRule (specified in function: Parser::create_parse_rule)
///
/// A parserule contains the token kind precedences and possibly parse methods
#[derive(Debug, Clone, Copy)]
pub(crate) struct ParseRule {
    pub prefix_method: Option<ParseRuleMethod>,
    pub prefix_prec: Precedence,
    pub infix_method: Option<ParseRuleMethod>,
    pub infix_prec: Precedence,
    pub postfix_method: Option<ParseRuleMethod>,
    pub postfix_prec: Precedence,
}

impl ParseRule {
    pub const fn dummy() -> Self {
        Self {
            prefix_method: None,
            prefix_prec: Precedence::PrecNone,
            infix_method: None,
            infix_prec: Precedence::PrecNone,
            postfix_method: None,
            postfix_prec: Precedence::PrecNone,
        }
    }
}

pub(crate) trait ParserHandle {
    fn get_ast_node_id(&mut self) -> NodeId;

    fn forget_node(&mut self, node_id: NodeId);
}

impl<'a> ParserHandle for Parser<'a> {
    fn forget_node(&mut self, node_id: NodeId) {
        self.forgotten_nodes += 1;
    }

    fn get_ast_node_id(&mut self) -> NodeId {
        Parser::get_ast_node_id(self)
    }
}

pub struct Parser<'a> {
    parse_rules: [ParseRule; PARSE_RULE_COUNT],
    lexer: Lexer<'a>,
    ast_arena: &'a AstArena,
    src: &'a str,
    current: Token,
    prev: Token,
    next_ast_node_id: NodeId,
    forgotten_nodes: usize,
    result_locs: Vec<ResultLoc>,
    next_result_loc: ResultLoc,

    /// Used for error reporting
    errors: Vec<Error>,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str, ast_arena: &'a AstArena) -> Self {
        let mut lexer = Lexer::new(src);

        Self {
            current: lexer.scan_token(),
            src,
            parse_rules: Self::create_parse_rules(),
            ast_arena,
            lexer,
            prev: Token::dummy(),
            next_ast_node_id: NodeId(0),
            forgotten_nodes: 0,
            result_locs: vec![],
            next_result_loc: ResultLoc(0),
            errors: Vec::new(),
        }
    }

    pub fn parse_into_ast(mut self) -> Ast<'a, AstState0> {
        let global_scope = self.parse_global_scope();

        let nodes_count = (self.next_ast_node_id.0 as usize) - self.forgotten_nodes;

        Ast::new(global_scope, AstQuerySystem::new(nodes_count))
    }

    pub(crate) fn statement(&mut self) -> &'a Stmt<'a> {
        let stmt = match self.current.get_kind() {
            TokenKind::Def => self.function_statement(),
            TokenKind::Mut => self.mut_stmt(),
            TokenKind::Break => self.break_expr(),
            TokenKind::Continue => self.continue_expr(),
            _ => self.expression_statement(),
        };

        self.ast_arena.alloc_expr_or_stmt(stmt)
    }

    pub(crate) fn expression_statement(&mut self) -> Stmt<'a> {
        let mut expr_builder = ExprBuilder::new(self.ast_arena);
        self.parse_precedence(expr_builder.get_base_prec(), &mut expr_builder);
        let stmt = expr_builder.take_stmt().expect("TODO: Error handling");
        stmt
    }

    pub(crate) fn break_expr(&mut self) -> Stmt<'a> {
        let break_expr = self.ast_arena.alloc_expr_or_stmt(
            BreakExpr::new(None, self.get_ast_node_id())
        );
        let expr_kind = ExprKind::ExprWithoutBlock(ExprWithoutBlock::BreakExpr(break_expr));
        let expr = self.ast_arena.alloc_expr_or_stmt(Expr::new(expr_kind, self.get_ast_node_id()));
        self.advance();
        Stmt::ExprStmt(expr)
    }

    pub(crate) fn continue_expr(&mut self) -> Stmt<'a> {
        let continue_expr = self.ast_arena.alloc_expr_or_stmt(
            ContinueExpr::new(self.get_ast_node_id())
        );
        let expr_kind = ExprKind::ExprWithoutBlock(ExprWithoutBlock::ContinueExpr(continue_expr));
        let expr = self.ast_arena.alloc_expr_or_stmt(Expr::new(expr_kind, self.get_ast_node_id()));
        self.advance();
        Stmt::ExprStmt(expr)
    }

    pub(crate) fn function_statement(&mut self) -> Stmt<'a> {
        self.advance();

        let ident_expr = self.consume_ident("Expected ident after `def`");

        self.consume(TokenKind::LeftParen, "Expected '(' after function name");
        // Args here
        self.consume(TokenKind::RightParen, "Expected ')' after function name");

        let body = self.parse_block();

        self.consume(TokenKind::End, "Expected `end` after function def");

        let expr_kind = ExprKind::ExprWithBlock(ExprWithBlock::BlockExpr(body));
        let expr = self.ast_arena.alloc_expr_or_stmt(Expr::new(expr_kind, self.get_ast_node_id()));

        let fn_stmt = self.ast_arena.alloc_expr_or_stmt(
            FunctionStmt::new(ident_expr, expr, self.get_ast_node_id())
        );

        Stmt::ItemStmt(ItemStmt::FunctionStmt(fn_stmt))
    }

    pub(crate) fn mut_stmt(&mut self) -> Stmt<'a> {
        let mut_span = self.prev.get_span();
        self.advance();
        let mut expr_builder = ExprBuilder::new(self.ast_arena);
        self.parse_precedence(expr_builder.get_base_prec(), &mut expr_builder);
        let stmt = expr_builder.take_stmt().expect("TODO: Error handling");
        Self::test_and_set_mutable(&stmt, mut_span).expect("Misuse of mut");
        stmt
    }

    fn test_and_set_mutable<'b>(stmt: &'b Stmt<'a>, mut_span: Span) -> Result<(), &'b str> {
        match stmt {
            Stmt::DefineStmt(define_stmt) => {
                define_stmt.mut_span.set(Some(mut_span));
                Ok(())
            }
            Stmt::AssignStmt(_) => Err("Unexpected token `mut` in assignment"),
            Stmt::ExprStmt(_) => Err("Unexpected token `mut` in expression statement"),
            Stmt::ItemStmt(_) => Err("Unexpected token `mut` before item statement"),
        }
    }

    /// Parse rule method: `define`
    pub(crate) fn define(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        expr_builder.set_base_prec(Precedence::PrecAssign.get_next());
        self.parse_precedence(expr_builder.get_base_prec(), expr_builder);
        expr_builder.emit_define_stmt(self)
    }

    /// Parse rule method: `assign`
    pub(crate) fn assign(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        expr_builder.set_base_prec(Precedence::PrecAssign.get_next());
        self.parse_precedence(expr_builder.get_base_prec(), expr_builder);
        expr_builder.emit_assign_stmt(self)
    }

    /// Parse rule method: `block`
    pub(crate) fn block_expr(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let block_expr = self.parse_block();
        self.consume(TokenKind::End, "Expected `end`");
        expr_builder.emit_block_expr(block_expr, self);
    }

    /// Parse rule method: `grouping`, called by prefix `(`
    pub(crate) fn grouping(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let mut exprs = Vec::new();
        while !self.is_eof() {
            let expr = {
                let base_prec = expr_builder.get_base_prec();
                let mut expr_builder = ExprBuilder::new(self.ast_arena);
                self.parse_precedence(base_prec, &mut expr_builder);
                self.ast_arena.alloc_expr_or_stmt(
                    expr_builder.take_expr().expect("TODO:Error handling")
                )
            };

            exprs.push(expr);

            if self.is_curr_kind(TokenKind::Comma) {
                self.advance();
                continue;
            }

            self.consume(TokenKind::RightParen, "Expected ')' after group");
            break;
        }

        expr_builder.emit_grouping_or_tuple_expr(self, exprs)
    }

    /// Parse rule method: `ident`
    pub(crate) fn ident(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let ident_expr = IdentExpr::new(self.prev.get_span(), self.get_ast_node_id());
        expr_builder.emit_ident_expr(ident_expr, self);
    }

    /// Parse rule method: `true_literal`
    pub(crate) fn true_lit(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let bool_expr = BoolExpr::new(true, self.get_ast_node_id());
        expr_builder.emit_bool_expr(bool_expr, self)
    }

    /// Parse rule method: `false_literal`
    pub(crate) fn false_lit(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let bool_expr = BoolExpr::new(false, self.get_ast_node_id());
        expr_builder.emit_bool_expr(bool_expr, self)
    }

    /// Parse rule method: `integer`
    pub(crate) fn integer(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let lexeme = self.get_lexeme_of_prev();
        let val = lexeme.parse::<i64>().expect("TODO: Error handling");
        let integer_expr = IntegerExpr::new(val, self.get_ast_node_id());
        expr_builder.emit_integer_expr(integer_expr, self);
    }

    /// Parse rule method: `dot_float`
    pub(crate) fn dot_float(&mut self, _expr_builder: &mut ExprBuilder<'a>) {
        todo!()
    }

    /// Parse rule method: `float`
    pub(crate) fn float(&mut self, _expr_builder: &mut ExprBuilder<'a>) {
        todo!("Floats not implemented yet: {:?}", self.get_lexeme_of_prev().parse::<f64>())
    }

    /// Parse rule method: `eq`
    pub(crate) fn eq(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.binary(expr_builder, BinaryOp::ComparisonOp(ComparisonOp::Eq));
    }

    /// Parse rule method: `ne`
    pub(crate) fn ne(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.binary(expr_builder, BinaryOp::ComparisonOp(ComparisonOp::Ne));
    }

    /// Parse rule method: `ge`
    pub(crate) fn ge(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.binary(expr_builder, BinaryOp::ComparisonOp(ComparisonOp::Ge));
    }

    /// Parse rule method: `gt`
    pub(crate) fn gt(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.binary(expr_builder, BinaryOp::ComparisonOp(ComparisonOp::Gt));
    }

    /// Parse rule method: `le`
    pub(crate) fn le(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.binary(expr_builder, BinaryOp::ComparisonOp(ComparisonOp::Le));
    }

    /// Parse rule method: `lt`
    pub(crate) fn lt(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.binary(expr_builder, BinaryOp::ComparisonOp(ComparisonOp::Lt));
    }

    /// Parse rule method: `add`
    pub(crate) fn add(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.binary(expr_builder, BinaryOp::ArithmeticOp(ArithmeticOp::Add))
    }

    /// Parse rule method: `sub`
    pub(crate) fn sub(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.binary(expr_builder, BinaryOp::ArithmeticOp(ArithmeticOp::Sub))
    }

    /// Parse rule method: `mul`
    pub(crate) fn mul(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.binary(expr_builder, BinaryOp::ArithmeticOp(ArithmeticOp::Mul))
    }

    /// Parse rule method: `div`
    pub(crate) fn div(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.binary(expr_builder, BinaryOp::ArithmeticOp(ArithmeticOp::Div))
    }

    /// Logic of binary parse rule methods
    pub(crate) fn binary(&mut self, expr_builder: &mut ExprBuilder<'a>, binary_op: BinaryOp) {
        self.parse_precedence(self.get_parse_rule_of_prev().infix_prec.get_next(), expr_builder);

        expr_builder.emit_binary_expr(binary_op, self)
    }

    /// Parse rule method: `loop_expr`
    pub(crate) fn loop_expr(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.push_result_loc();

        let block = self.parse_block();
        self.consume(TokenKind::End, "Expected `end` after loop");

        let loop_expr = self.ast_arena.alloc_expr_or_stmt(
            LoopExpr::new(block, self.get_ast_node_id(), self.read_result_loc())
        );

        expr_builder.emit_loop_expr(loop_expr, self);

        self.pop_result_loc();
    }

    /// Parse rule method: `if_expr`
    pub(crate) fn if_expr(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.push_result_loc();
        let if_expr = self.parse_if_expr();

        expr_builder.emit_if_expr(if_expr, self);
        self.pop_result_loc();
    }

    pub fn parse_expr_and_take(&mut self) -> &'a Expr<'a> {
        let mut expr_builder = ExprBuilder::new(self.ast_arena);
        self.parse_precedence(expr_builder.get_base_prec(), &mut expr_builder);
        self.ast_arena.alloc_expr_or_stmt(expr_builder.take_expr().expect("TODO:Error handling"))
    }

    pub(crate) fn parse_if_expr(&mut self) -> &'a IfExpr<'a> {
        let cond = self.parse_expr_and_take();

        self.consume(TokenKind::Then, "Expected 'then' after if-condition");

        let true_block = self.parse_block();

        self.advance_if(matches!(self.current.get_kind(), TokenKind::Else | TokenKind::Elif));
        let false_block = match self.prev.get_kind() {
            TokenKind::Else => {
                let block_expr = self.parse_block();
                self.consume(TokenKind::End, "Expected `end` after if expression");
                Some(IfFalseBranchExpr::ElseExpr(block_expr))
            }
            TokenKind::Elif => Some(IfFalseBranchExpr::ElifExpr(self.parse_if_expr())),
            _ => {
                self.consume(TokenKind::End, "Expected `end` after if expression");
                None
            }
        };

        let if_expr = self.ast_arena.alloc_expr_or_stmt(
            IfExpr::new(
                cond,
                true_block,
                false_block,
                self.get_ast_node_id(),
                Span::dummy(),
                self.read_result_loc()
            )
        );

        if_expr
    }

    fn parse_block(&mut self) -> &'a BlockExpr<'a> {
        let mut stmts = Vec::with_capacity(32);
        while !self.current.get_kind().can_end_scope() && !self.is_eof() {
            let stmt = self.statement();
            stmts.push(stmt);
        }
        let stmts = self.ast_arena.alloc_vec_stmts(stmts);

        let block_expr = self.ast_arena.alloc_expr_or_stmt(
            BlockExpr::new(stmts, self.get_ast_node_id())
        );

        block_expr
    }

    fn parse_global_scope(&mut self) -> GlobalScope<'a> {
        let mut stmts = Vec::with_capacity(32);

        while !self.is_eof() {
            let stmt = self.statement();
            stmts.push(stmt);
        }

        let stmts = self.ast_arena.alloc_vec_stmts(stmts);

        GlobalScope::new(stmts)
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

    /* Helper methods */

    pub(crate) fn get_ast_node_id(&mut self) -> NodeId {
        let prev = self.next_ast_node_id;
        self.next_ast_node_id = NodeId(prev.0 + 1);
        prev
    }

    pub(crate) fn push_result_loc(&mut self) {
        let next_result_loc = {
            let prev = self.next_result_loc;
            self.next_result_loc = ResultLoc(prev.0 + 1);
            prev
        };

        self.result_locs.push(next_result_loc);
    }

    pub(crate) fn pop_result_loc(&mut self) {
        self.result_locs.pop();
    }

    pub(crate) fn read_result_loc(&mut self) -> ResultLoc {
        *self.result_locs.last().expect("TODO: Error handling (read_result_loc)")
    }

    /// Converts e.g. `69` into `0.69` (this is a fast version by chat)
    pub(crate) fn _integer_to_dot_float(int: i64) -> f64 {
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

    pub(crate) fn is_curr_kind(&self, kind: TokenKind) -> bool {
        self.current.get_kind() == kind
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
            self.advance();
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

    pub(crate) fn consume_ident(&mut self, err_msg: &str) -> IdentExpr {
        match self.current.get_kind() {
            TokenKind::Ident => {
                let ident_expr = IdentExpr::new(self.current.get_span(), self.get_ast_node_id());
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

    pub(crate) const fn create_parse_rule(kind: TokenKind) -> ParseRule {
        make_parse_rule!(kind,

    /*  TOKENKIND        INFIX                  PREFIX                              POSTFIX             */
    /*                   method     prec        method      prec                    method      prec    */
        LeftParen   = { (grouping   None),      (None       None            ),      (None       None) },
        RightParen  = { (None       None),      (None       None            ),      (None       None) },
        Eq          = { (None       None),      (eq         PrecEquality    ),      (None       None) },
        Ne          = { (None       None),      (ne         PrecEquality    ),      (None       None) },
        Ge          = { (None       None),      (ge         PrecComparison  ),      (None       None) },
        Gt          = { (None       None),      (gt         PrecComparison  ),      (None       None) },
        Le          = { (None       None),      (le         PrecComparison  ),      (None       None) },
        Lt          = { (None       None),      (lt         PrecComparison  ),      (None       None) },
        Plus        = { (None       None),      (add        PrecTerm        ),      (None       None) },
        Minus       = { (None       None),      (sub        PrecTerm        ),      (None       None) },
        Star        = { (None       None),      (mul        PrecFactor      ),      (None       None) },
        Slash       = { (None       None),      (div        PrecFactor      ),      (None       None) },
        Colon       = { (None       None),      (None       None            ),      (None       None) },
        Define      = { (None       None),      (define     PrecAssign      ),      (None       None) },
        Assign      = { (None       None),      (assign     PrecAssign      ),      (None       None) },
        Dot         = { (dot_float  None),      (None       None            ),      (None       None) },
        Comma       = { (None       None),      (None       None            ),      (None       None)},
        Bang        = { (None       None),      (None       None            ),      (None       None)   },
        
            
        // Numbers
        Integer     = { (integer    None),      (None       None            ),      (None       None) },
        Float       = { (float      None),      (None       None            ),      (None       None) },

        // Booleans
        True        = { (true_lit   None),      (None       None            ),      (None       None)   },
        False       = { (false_lit  None),      (None       None            ),      (None       None)   },
            
        // Identifier
        Ident       = { (ident      None),      (None       None            ),      (None       None) },

        // Keywords
        Def         = { (None       None),      (None       None            ),      (None       None) },
        Mut         = { (None       None),      (None       None            ),      (None       None) },
        Class       = { (None       None),      (None       None            ),      (None       None) },
        While       = { (None       None),      (None       None            ),      (None       None) },
        If          = { (if_expr    None),      (None       None            ),      (None       None) },
        Loop        = { (loop_expr  None),      (None       None            ),      (None       None) },
        Break       = { (None       None),      (None       None            ),      (None       None) },
        Continue    = { (None       None),      (None       None            ),      (None       None) },
        Return      = { (None       None),      (None       None            ),      (None       None) },
        Do          = { (block_expr None),      (None       None            ),      (None       None) },
        Then        = { (None       None),      (None       None            ),      (None       None) },
        Else        = { (None       None),      (None       None            ),      (None       None) },
        Elif        = { (None       None),      (None       None            ),      (None       None) },
        End         = { (None       None),      (None       None            ),      (None       None) },
        Eof         = { (None       None),      (None       None            ),      (None       None) }
        
        )
    }
}
