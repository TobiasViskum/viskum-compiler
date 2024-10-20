use std::collections::VecDeque;

use ast::{
    Ast,
    AstArena,
    AstQuerySystem,
    AstState0,
    BlockExpr,
    BoolExpr,
    BreakExpr,
    ContinueExpr,
    EnumItem,
    EnumVariant,
    Expr,
    ExprWithoutBlock,
    Field,
    FieldInitialization,
    FnItem,
    GlobalScope,
    IdentNode,
    IfDefExpr,
    IfExpr,
    IfExprKind,
    IfFalseBranchExpr,
    IntegerExpr,
    ItemStmt,
    LoopExpr,
    Pat,
    Path,
    PathField,
    PlaceExpr,
    ReturnExpr,
    Stmt,
    StructItem,
    TuplePat,
    TypedefItem,
    Typing,
    ValueExpr,
};
use error::Error;
use expr_builder::ExprBuilder;
use fxhash::FxHashMap;
use ir::{ DefId, NodeId, Symbol };
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

pub(crate) trait ParserHandle<'ast> {
    fn get_ast_node_id(&mut self) -> NodeId;

    fn forget_node(&mut self, node_id: NodeId);

    fn try_as_pat(&mut self, expr: Expr<'ast>) -> Option<Pat<'ast>>;

    fn try_as_place_expr(&mut self, expr: Expr<'ast>) -> Option<PlaceExpr<'ast>>;

    fn try_as_ident(&mut self, expr: Expr<'ast>) -> Option<&'ast IdentNode>;

    fn try_as_path(&mut self, expr: Expr<'ast>) -> Option<Path<'ast>>;
}

impl<'a> ParserHandle<'a> for Parser<'a> {
    fn forget_node(&mut self, node_id: NodeId) {
        self.forgotten_nodes += 1;
    }

    fn get_ast_node_id(&mut self) -> NodeId {
        Parser::get_ast_node_id(self)
    }

    fn try_as_path(&mut self, expr: Expr<'a>) -> Option<Path<'a>> {
        match expr {
            Expr::ExprWithBlock(_) => None,
            Expr::ExprWithoutBlock(expr) => {
                match expr {
                    ExprWithoutBlock::PlaceExpr(place_expr) => {
                        match place_expr {
                            PlaceExpr::IdentExpr(ident_expr) => {
                                Some(Path::PathSegment(ident_expr))
                            }
                            PlaceExpr::TupleFieldExpr(_) => None,
                            PlaceExpr::FieldExpr(field_expr) => {
                                let lhs = self.try_as_path(field_expr.lhs);

                                match lhs {
                                    Some(lhs) => {
                                        Some(
                                            Path::PathField(
                                                self.ast_arena.alloc_expr_or_stmt(
                                                    PathField::new(
                                                        lhs,
                                                        field_expr.rhs,
                                                        field_expr.ast_node_id
                                                    )
                                                )
                                            )
                                        )
                                    }
                                    None => None,
                                }
                            }
                        }
                    }
                    ExprWithoutBlock::BreakExpr(_) => None,
                    ExprWithoutBlock::ContinueExpr(_) => None,
                    ExprWithoutBlock::ReturnExpr(_) => None,
                    ExprWithoutBlock::ValueExpr(value_expr) => {
                        match value_expr {
                            ValueExpr::BinaryExpr(_) => None,
                            ValueExpr::CallExpr(_) => None,
                            ValueExpr::ConstExpr(_) => None,
                            ValueExpr::GroupExpr(_) => None,
                            ValueExpr::StructExpr(_) => None,
                            ValueExpr::TupleExpr(_) => None,
                        }
                    }
                }
            }
        }
    }

    fn try_as_pat(&mut self, expr: Expr<'a>) -> Option<Pat<'a>> {
        match expr {
            Expr::ExprWithBlock(_) => None,
            Expr::ExprWithoutBlock(expr) => {
                match expr {
                    ExprWithoutBlock::PlaceExpr(expr) => {
                        match expr {
                            PlaceExpr::TupleFieldExpr(_) => None,
                            PlaceExpr::FieldExpr(_) => None,
                            PlaceExpr::IdentExpr(ident_expr) =>
                                Some(
                                    Pat::IdentPat(
                                        self.ast_arena.alloc_expr_or_stmt(ident_expr.get_copy())
                                    )
                                ),
                        }
                    }
                    ExprWithoutBlock::BreakExpr(_) => None,
                    ExprWithoutBlock::ContinueExpr(_) => None,
                    ExprWithoutBlock::ReturnExpr(_) => None,
                    ExprWithoutBlock::ValueExpr(expr) => {
                        match expr {
                            ValueExpr::BinaryExpr(_) => None,
                            ValueExpr::CallExpr(call_expr) => {
                                let path = match self.try_as_path(call_expr.callee) {
                                    Some(path) => path,
                                    None => {
                                        return None;
                                    }
                                };

                                let mut has_args_failed = false;

                                let pat_args = call_expr.args
                                    .iter()
                                    .filter_map(|arg| {
                                        let pat = self.try_as_pat(*arg);
                                        if pat.is_none() {
                                            has_args_failed = true;
                                        }
                                        pat
                                    })
                                    .collect::<Vec<_>>();

                                if has_args_failed {
                                    return None;
                                }

                                let final_pat = Pat::TuplePat(
                                    self.ast_arena.alloc_expr_or_stmt(
                                        TuplePat::new(
                                            path,
                                            self.ast_arena.alloc_vec(pat_args),
                                            call_expr.ast_node_id
                                        )
                                    )
                                );

                                Some(final_pat)
                            }
                            ValueExpr::ConstExpr(_) => None,
                            ValueExpr::GroupExpr(_) => None,
                            ValueExpr::StructExpr(_) => None,
                            ValueExpr::TupleExpr(tuple_expr) =>
                                todo!("As pattern: {:#?}", tuple_expr),
                        }
                    }
                }
            }
        }
    }

    fn try_as_place_expr(&mut self, expr: Expr<'a>) -> Option<PlaceExpr<'a>> {
        match expr {
            Expr::ExprWithBlock(_) => None,
            Expr::ExprWithoutBlock(expr) => {
                match expr {
                    ExprWithoutBlock::PlaceExpr(expr) => { Some(expr) }
                    ExprWithoutBlock::BreakExpr(_) => None,
                    ExprWithoutBlock::ReturnExpr(_) => None,
                    ExprWithoutBlock::ContinueExpr(_) => None,
                    ExprWithoutBlock::ValueExpr(expr) => {
                        match expr {
                            ValueExpr::BinaryExpr(_) => None,
                            ValueExpr::ConstExpr(_) => None,
                            ValueExpr::GroupExpr(_) => None,
                            ValueExpr::CallExpr(_) => None,
                            ValueExpr::StructExpr(_) => None,
                            ValueExpr::TupleExpr(tuple_expr) =>
                                todo!("As place expr: {:#?}", tuple_expr),
                        }
                    }
                }
            }
        }
    }

    fn try_as_ident(&mut self, expr: Expr<'a>) -> Option<&'a IdentNode> {
        match expr {
            Expr::ExprWithBlock(_) => None,
            Expr::ExprWithoutBlock(expr) => {
                match expr {
                    ExprWithoutBlock::BreakExpr(_) => None,
                    ExprWithoutBlock::ReturnExpr(_) => None,
                    ExprWithoutBlock::ContinueExpr(_) => None,
                    ExprWithoutBlock::ValueExpr(_) => None,
                    ExprWithoutBlock::PlaceExpr(expr) => {
                        match expr {
                            PlaceExpr::IdentExpr(expr) => Some(expr),
                            PlaceExpr::TupleFieldExpr(_) => None,
                            PlaceExpr::FieldExpr(_) => None,
                        }
                    }
                }
            }
        }
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
    parsed_fn_count: usize,

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
            parsed_fn_count: 0,
            prev: Token::dummy(),
            next_ast_node_id: NodeId(0),

            forgotten_nodes: 0,
            errors: Vec::new(),
        }
    }

    pub fn parse_into_ast(mut self) -> Ast<'a, AstState0> {
        let global_scope = self.parse_global_scope();

        let nodes_count = (self.next_ast_node_id.0 as usize) - self.forgotten_nodes;

        Ast::new(global_scope, self.parsed_fn_count, AstQuerySystem::new(nodes_count))
    }

    fn get_symbol_from_ident_node(&self, ident_node: &IdentNode) -> Symbol {
        Symbol::new(&self.src[ident_node.span.get_byte_range()])
    }

    pub(crate) fn statement(&mut self) -> Stmt<'a> {
        match self.current.get_kind() {
            TokenKind::Def => self.function_statement(),
            TokenKind::Typedef => self.typedef_statement(),
            TokenKind::Struct => self.struct_item(),
            TokenKind::Enum => self.enum_item(),
            TokenKind::Mut => self.mut_stmt(),
            TokenKind::Break => self.break_expr(),
            TokenKind::Continue => self.continue_expr(),
            TokenKind::Fn => self.function_statement(),
            TokenKind::Return => self.return_expr(),
            _ => self.expression_statement(),
        }
    }

    pub(crate) fn typedef_statement(&mut self) -> Stmt<'a> {
        self.advance();
        let ident_node = self.consume_ident("Expected ident after `typedef`");
        let ty = self.parse_typing().expect("TODO: Error handling, Expected type");

        let typedef_stmt = ItemStmt::TypedefItem(
            self.ast_arena.alloc_expr_or_stmt(
                TypedefItem::new(
                    self.ast_arena.alloc_expr_or_stmt(ident_node),
                    ty,
                    self.get_ast_node_id()
                )
            )
        );

        Stmt::ItemStmt(typedef_stmt)
    }

    pub(crate) fn enum_item(&mut self) -> Stmt<'a> {
        self.advance();
        let ident_node = self.consume_ident("Expected ident after `enum`");
        let mut variants = Vec::with_capacity(8);

        self.consume(TokenKind::LeftCurly, "Expected `{` before enum variants");

        while !self.is_eof() && !self.is_curr_kind(TokenKind::RightCurly) {
            let variant_name = self.ast_arena.alloc_expr_or_stmt(
                self.consume_ident("Expected ident in enum variant")
            );

            match self.current.get_kind() {
                TokenKind::Comma => {
                    self.advance();
                    variants.push(EnumVariant::new(variant_name, None));
                    continue;
                }
                TokenKind::RightCurly => {
                    variants.push(EnumVariant::new(variant_name, None));
                    break;
                }
                TokenKind::LeftParen => {
                    self.advance();
                    let mut tys = Vec::with_capacity(8);
                    loop {
                        let ty = self.parse_typing().expect("Expected type in enum variant");
                        tys.push(ty);

                        if self.is_curr_kind(TokenKind::Comma) {
                            self.advance();
                            continue;
                        }

                        break;
                    }
                    variants.push(
                        EnumVariant::new(variant_name, Some(self.ast_arena.alloc_vec(tys)))
                    );

                    self.consume(TokenKind::RightParen, "Expected `)` after enum variant");

                    if self.is_curr_kind(TokenKind::Comma) {
                        self.advance();
                    }
                    continue;
                }
                _ => panic!("Unexpected token in enum variant"),
            }
        }

        self.consume(TokenKind::RightCurly, "Expected `}` after enum variants");

        let enum_item = EnumItem::new(
            self.ast_arena.alloc_expr_or_stmt(ident_node),
            self.ast_arena.alloc_vec(variants),
            self.get_ast_node_id()
        );

        Stmt::ItemStmt(ItemStmt::EnumItem(self.ast_arena.alloc_expr_or_stmt(enum_item)))
    }

    pub(crate) fn struct_item(&mut self) -> Stmt<'a> {
        self.advance();
        let ident_node = self.consume_ident("Expected identifier after struct");
        let mut fields = Vec::with_capacity(8);

        self.consume(TokenKind::LeftCurly, "Expected `{` before struct fields");

        while !self.is_eof() && !self.is_curr_kind(TokenKind::RightCurly) {
            let field_name = self.consume_ident("Expected ident in field");
            let ty = self.parse_typing().expect("TODO: Error handling, Expected type");
            let field = Field::new(self.ast_arena.alloc_expr_or_stmt(field_name), ty);

            fields.push(self.ast_arena.alloc_expr_or_stmt(field));

            if self.is_curr_kind(TokenKind::Comma) {
                self.advance();
                continue;
            }

            match self.current.get_kind() {
                TokenKind::RightCurly => {}
                TokenKind::Ident =>
                    todo!("Error: You are probably missing a `,` in struct declaration"),
                t =>
                    todo!("Error: Unexpected token `{}` in struct declaration. Expected `,` or `}}`", t),
            }
            break;
        }

        self.consume(TokenKind::RightCurly, "Expected `}` after struct");

        let fields = self.ast_arena.alloc_vec(fields);
        let struct_stmt = StructItem::new(
            self.ast_arena.alloc_expr_or_stmt(ident_node),
            fields,
            self.get_ast_node_id()
        );

        Stmt::ItemStmt(ItemStmt::StructItem(self.ast_arena.alloc_expr_or_stmt(struct_stmt)))
    }

    pub(crate) fn parse_typing(&mut self) -> Option<Typing<'a>> {
        fn parse_many_typings<'a>(
            parser: &mut Parser<'a>,
            mut tuple_typing: Vec<Typing<'a>>
        ) -> &'a [Typing<'a>] {
            while !parser.is_eof() && !parser.is_curr_kind(TokenKind::RightParen) {
                let typing = parser.parse_typing().expect("Expected typing");
                tuple_typing.push(typing);
                if parser.is_curr_kind(TokenKind::Comma) {
                    parser.advance();
                    continue;
                }
                break;
            }
            parser.consume(TokenKind::RightParen, "Expected `)` after tuple typing");
            parser.ast_arena.alloc_vec(tuple_typing)
        }

        if self.is_curr_kind(TokenKind::Ident) {
            let ident = self.consume_ident_span("Expected ident");
            Some(Typing::Ident(ident))
        } else if self.is_curr_kind(TokenKind::LeftParen) {
            self.advance();
            let typing = self.parse_typing().expect("Expected typing");

            match self.current.get_kind() {
                TokenKind::RightParen => {
                    self.advance();
                    Some(typing)
                }
                TokenKind::Comma => {
                    self.advance();
                    let tuple_typing = parse_many_typings(self, vec![typing]);
                    Some(Typing::Tuple(tuple_typing))
                }
                t => panic!("Unexpected token in typing: {}", t),
            }
        } else if self.is_curr_kind(TokenKind::Fn) {
            self.advance();
            self.consume(TokenKind::LeftParen, "Expected `(` before function args");
            let args_typing = parse_many_typings(self, vec![]);
            let ret_typing = self.parse_typing().map(|x| self.ast_arena.alloc_expr_or_stmt(x));

            Some(Typing::Fn(args_typing, ret_typing))
        } else {
            None
        }
    }

    pub(crate) fn expression_statement(&mut self) -> Stmt<'a> {
        let mut expr_builder = ExprBuilder::new(self.ast_arena, None);
        self.expression(&mut expr_builder);
        let stmt = expr_builder.take_stmt().expect("TODO: Error handling");
        stmt
    }

    pub(crate) fn break_expr(&mut self) -> Stmt<'a> {
        let break_expr = self.ast_arena.alloc_expr_or_stmt(
            BreakExpr::new(None, self.get_ast_node_id())
        );
        let expr = Expr::ExprWithoutBlock(ExprWithoutBlock::BreakExpr(break_expr));
        self.advance();
        Stmt::ExprStmt(expr)
    }

    pub(crate) fn continue_expr(&mut self) -> Stmt<'a> {
        let continue_expr = self.ast_arena.alloc_expr_or_stmt(
            ContinueExpr::new(self.get_ast_node_id())
        );
        let expr = Expr::ExprWithoutBlock(ExprWithoutBlock::ContinueExpr(continue_expr));
        self.advance();
        Stmt::ExprStmt(expr)
    }

    pub(crate) fn function_statement(&mut self) -> Stmt<'a> {
        self.advance();

        let ident_expr = self.consume_ident("Expected ident after `def`");

        self.consume(TokenKind::LeftParen, "Expected '(' after function name");

        let mut args = Vec::with_capacity(8);
        while !self.is_eof() && !self.is_curr_kind(TokenKind::RightParen) {
            let arg = {
                let arg_ident = self.consume_ident("Expected ident in function args");
                let arg_ty = self.parse_typing().expect("Expected type in function args");
                let arg = Field::new(self.ast_arena.alloc_expr_or_stmt(arg_ident), arg_ty);
                self.ast_arena.alloc_expr_or_stmt(arg)
            };
            args.push(arg);

            if self.is_curr_kind(TokenKind::Comma) {
                self.advance();
                continue;
            }

            break;
        }
        let args = self.ast_arena.alloc_vec(args);

        self.consume(TokenKind::RightParen, "Expected ')' after function args");

        let return_ty = self.parse_typing();

        self.consume(TokenKind::LeftCurly, "Expected `{` before function body");

        let body = self.parse_block_as_stmts();

        self.consume(TokenKind::RightCurly, "Expected `}` after function body");

        let fn_stmt = self.ast_arena.alloc_expr_or_stmt(
            FnItem::new(
                self.ast_arena.alloc_expr_or_stmt(ident_expr),
                body,
                args,
                return_ty,
                self.get_ast_node_id()
            )
        );

        self.parsed_fn_count += 1;

        Stmt::ItemStmt(ItemStmt::FnItem(fn_stmt))
    }

    fn return_expr(&mut self) -> Stmt<'a> {
        self.advance();

        let ret_value_expr = if self.get_parse_rule_of_current().prefix_method.is_some() {
            Some(self.parse_expr_and_take(Precedence::PrecAssign.get_next()))
        } else {
            None
        };

        let return_expr = self.ast_arena.alloc_expr_or_stmt(
            ReturnExpr::new(ret_value_expr, self.get_ast_node_id())
        );

        let expr = Expr::ExprWithoutBlock(ExprWithoutBlock::ReturnExpr(return_expr));

        Stmt::ExprStmt(expr)
    }

    pub(crate) fn mut_stmt(&mut self) -> Stmt<'a> {
        let mut_span = self.prev.get_span();
        self.advance();
        let mut expr_builder = ExprBuilder::new(self.ast_arena, None);
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

    pub(crate) fn expression(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.parse_precedence(expr_builder.get_base_prec(), expr_builder);
    }

    /// Parse rule method: `define`
    pub(crate) fn define(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        expr_builder.set_base_prec(Precedence::PrecAssign.get_next());
        self.expression(expr_builder);
        expr_builder.emit_define_stmt(self)
    }

    /// Parse rule method: `assign`
    pub(crate) fn assign(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        expr_builder.set_base_prec(Precedence::PrecAssign.get_next());
        self.expression(expr_builder);
        expr_builder.emit_assign_stmt(self)
    }

    /// Parse rule method: `block`
    pub(crate) fn block_expr(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let block_expr = self.parse_block();
        self.consume(TokenKind::RightCurly, "Expected `}` after block");
        expr_builder.emit_block_expr(block_expr);
    }

    /// Parse rule method: `grouping`, called by prefix `(`
    pub(crate) fn grouping(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let mut exprs = Vec::new();
        while !self.is_eof() {
            let expr = self.parse_expr_and_take(expr_builder.get_base_prec());
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

    /// Parse rule method: `call`
    pub(crate) fn call(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let mut args = Vec::with_capacity(8);
        while !self.is_eof() && !self.is_curr_kind(TokenKind::RightParen) {
            let arg = self.parse_expr_and_take(Precedence::PrecAssign.get_next());
            args.push(arg);

            if self.is_curr_kind(TokenKind::Comma) {
                self.advance();
                continue;
            }

            break;
        }

        self.consume(TokenKind::RightParen, "Expected ')' after function call");

        expr_builder.emit_call_expr(self, args);
    }

    /// Parse rule method: `ident`
    pub(crate) fn ident(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let ident_expr = IdentNode::new(self.prev.get_span(), self.get_ast_node_id());
        expr_builder.emit_ident_expr(ident_expr);
    }

    /// Parse rule method: `true_literal`
    pub(crate) fn true_lit(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let bool_expr = BoolExpr::new(true, self.get_ast_node_id());
        expr_builder.emit_bool_expr(bool_expr)
    }

    /// Parse rule method: `false_literal`
    pub(crate) fn false_lit(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let bool_expr = BoolExpr::new(false, self.get_ast_node_id());
        expr_builder.emit_bool_expr(bool_expr)
    }

    /// Parse rule method: `integer`
    pub(crate) fn integer(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let integer_expr = self.parse_integer_expr();
        expr_builder.emit_integer_expr(integer_expr);
    }

    pub(crate) fn parse_integer_expr(&mut self) -> IntegerExpr {
        let lexeme = self.get_lexeme_of_prev();
        let val = lexeme.parse::<i64>().expect("TODO: Error handling");
        IntegerExpr::new(val, self.get_ast_node_id())
    }

    /// Parse rule method: `dot_float`
    pub(crate) fn dot_float(&mut self, _expr_builder: &mut ExprBuilder<'a>) {
        todo!()
    }

    /// Parse rule method: `struct_expr`
    pub(crate) fn struct_expr(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let mut initialization_fields = Vec::with_capacity(8);
        while !self.is_eof() && !self.is_curr_kind(TokenKind::RightCurly) {
            let field_ident = self.consume_ident("Expected ident");
            self.consume(TokenKind::Colon, "Expected colon");
            let expr = self.parse_expr_and_take(Precedence::PrecAssign.get_next());
            let field_init = self.ast_arena.alloc_expr_or_stmt(
                FieldInitialization::new(self.ast_arena.alloc_expr_or_stmt(field_ident), expr)
            );
            initialization_fields.push(field_init);

            if self.is_curr_kind(TokenKind::Comma) {
                self.advance();
                continue;
            }

            break;
        }

        self.consume(TokenKind::RightCurly, "Expected `}` after struct expression");

        expr_builder.emit_struct_expr(initialization_fields, self)
    }

    /// Parse rule method: `dot_expr`
    pub(crate) fn field_expr(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        match self.current.get_kind() {
            TokenKind::Integer => {
                self.advance();
                let integer_expr = self.parse_integer_expr();
                expr_builder.emit_tuple_field_expr(integer_expr, self);
            }
            TokenKind::Ident => {
                let ident_node = self.consume_ident("Unreachable");
                expr_builder.emit_field_expr(ident_node, self);
            }
            _ => {
                println!("{}", self.current.get_kind());
                todo!()
                // self.expression(expr_builder);
                // // Emit dot expr
            }
        }
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

    pub(crate) fn post_inc(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.advance();
        expr_builder.emit_post_inc_expr(self)
    }

    pub(crate) fn post_dec(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        todo!()
        // self.advance();
        // expr_builder.emit_post_dec_expr(self)
    }

    pub(crate) fn pre_inc(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        todo!()
        // self.advance();
        // expr_builder.emit_pre_inc_expr(self)
    }

    pub(crate) fn pre_dec(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        todo!()
        // self.advance();
        // expr_builder.emit_pre_dec_expr(self)
    }

    /// Parse rule method: `loop_expr`
    pub(crate) fn loop_expr(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        self.consume(TokenKind::LeftCurly, "Expected `{` before loop");
        let block = self.parse_block();
        self.consume(TokenKind::RightCurly, "Expected `}` after loop");

        let loop_expr = self.ast_arena.alloc_expr_or_stmt(
            LoopExpr::new(block, self.get_ast_node_id())
        );

        expr_builder.emit_loop_expr(loop_expr);
    }

    /// Parse rule method: `if_expr`
    pub(crate) fn if_expr(&mut self, expr_builder: &mut ExprBuilder<'a>) {
        let if_expr = self.parse_if_expr();

        expr_builder.emit_if_expr(if_expr);
    }

    pub(crate) fn parse_expr_and_take_with_terminate_infix_token(
        &mut self,
        prec: Precedence,
        terminate_infix_token: Option<TokenKind>
    ) -> Expr<'a> {
        let mut expr_builder = ExprBuilder::new(self.ast_arena, terminate_infix_token);
        self.parse_precedence(prec, &mut expr_builder);
        expr_builder.take_expr().expect("TODO:Error handling")
    }

    pub(crate) fn parse_expr_and_take(&mut self, prec: Precedence) -> Expr<'a> {
        let mut expr_builder = ExprBuilder::new(self.ast_arena, None);
        self.parse_precedence(prec, &mut expr_builder);
        expr_builder.take_expr().expect("TODO:Error handling")
    }

    pub(crate) fn parse_if_expr(&mut self) -> IfExprKind<'a> {
        let cond = self.parse_expr_and_take_with_terminate_infix_token(
            Precedence::PrecAssign.get_next(),
            Some(TokenKind::LeftCurly)
        );

        let if_def_expr_data = if self.is_curr_kind(TokenKind::Define) {
            // We have an IfDefExpr
            let pat = self.try_as_pat(cond).expect("Expected pattern");

            self.advance();

            let rhs = self.parse_expr_and_take_with_terminate_infix_token(
                Precedence::PrecAssign.get_next(),
                Some(TokenKind::LeftCurly)
            );

            Some((pat, rhs))
        } else {
            None
        };

        self.consume(TokenKind::LeftCurly, "Expected `{` after if condition");

        let true_block = self.parse_block();

        self.consume(TokenKind::RightCurly, "Expected `}` after if block");

        self.advance_if(matches!(self.current.get_kind(), TokenKind::Else | TokenKind::Elif));

        let false_block = match self.prev.get_kind() {
            TokenKind::Else => {
                self.consume(TokenKind::LeftCurly, "Expected `{` after else");
                let block_expr = self.parse_block();
                self.consume(TokenKind::RightCurly, "Expected `}` after else block");
                Some(IfFalseBranchExpr::ElseExpr(block_expr))
            }
            TokenKind::Elif => Some(IfFalseBranchExpr::IfExprKind(self.parse_if_expr())),
            _ => None,
        };

        match if_def_expr_data {
            Some((pat, rhs)) => {
                let if_def_expr = self.ast_arena.alloc_expr_or_stmt(
                    IfDefExpr::new(
                        pat,
                        rhs,
                        true_block,
                        false_block,
                        self.get_ast_node_id(),
                        Span::dummy()
                    )
                );

                return IfExprKind::IfDefExpr(if_def_expr);
            }
            None => {
                let if_expr = self.ast_arena.alloc_expr_or_stmt(
                    IfExpr::new(
                        cond,
                        true_block,
                        false_block,
                        self.get_ast_node_id(),
                        Span::dummy()
                    )
                );

                return IfExprKind::IfExpr(if_expr);
            }
        }
    }

    fn parse_block_as_stmts(&mut self) -> &'a [Stmt<'a>] {
        let mut stmts = Vec::with_capacity(32);
        while !self.is_curr_kind(TokenKind::RightCurly) && !self.is_eof() {
            stmts.push(self.statement());
            // Self::push_stmt(&mut stmts, self.statement());
        }

        self.ast_arena.alloc_vec(stmts)
    }

    fn parse_block(&mut self) -> &'a BlockExpr<'a> {
        let stmts = self.parse_block_as_stmts();

        let block_expr = self.ast_arena.alloc_expr_or_stmt(
            BlockExpr::new(stmts, self.get_ast_node_id())
        );

        block_expr
    }

    fn parse_global_scope(&mut self) -> GlobalScope<'a> {
        let mut stmts = Vec::with_capacity(32);

        while !self.is_eof() {
            stmts.push(self.statement());
            // Self::push_stmt(&mut stmts, self.statement());
        }

        let stmts = self.ast_arena.alloc_vec(stmts);

        GlobalScope::new(stmts)
    }

    // fn push_stmt(vec_deque: &mut VecDeque<Stmt<'a>>, stmt: Stmt<'a>) {
    //     match stmt {
    //         Stmt::ItemStmt(ItemStmt::StructItem(_) | ItemStmt::TypedefItem(_)) => {
    //             vec_deque.push_front(stmt);
    //         }
    //         stmt => vec_deque.push_back(stmt),
    //     }
    // }

    pub(crate) fn parse_precedence(
        &mut self,
        mut prec: Precedence,
        expr_builder: &mut ExprBuilder<'a>
    ) {
        self.advance();

        let parse_rule = self.get_parse_rule_of_prev();

        if let Some(prefix_method) = parse_rule.prefix_method {
            macro_rules! is_terminate_infix_token {
                (current) => {
                    match expr_builder.terminate_infix_token {
                        Some(token_kind) => self.is_curr_kind(token_kind),
                        None => false,
                    }
                };
                (prev) => {
                    match expr_builder.terminate_infix_token {
                        Some(token_kind) => self.prev.get_kind() == token_kind,
                        None => false,
                    }
                };
            }

            if is_terminate_infix_token!(prev) {
                return;
            }

            prefix_method(self, expr_builder);

            loop {
                while
                    prec <= self.get_parse_rule_of_current().infix_prec &&
                    !is_terminate_infix_token!(current)
                {
                    // If this is true we don't want to parse another `:=` or `=` in expr without block
                    if self.current.get_kind().has_assign_prec() {
                        prec = Precedence::PrecAssign.get_next();
                    }

                    self.advance();

                    if let Some(infix_rule) = self.get_parse_rule_of_prev().infix_method {
                        infix_rule(self, expr_builder);
                    }
                }

                if let Some(postfix_rule) = self.get_parse_rule_of_current().postfix_method {
                    postfix_rule(self, expr_builder);
                    continue;
                } else {
                    break;
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

    /// Converts e.g. `69` into `0.69` (this is a fast version by chat)
    pub(crate) fn _integer_to_dot_float(int: i64) -> f64 {
        // Compute the number of digits using logarithms
        let num_digits = if int == 0 { 1 } else { (int.abs() as f64).log10().ceil() as u32 };

        // Compute the divisor
        let divisor = (10.0_f64).powi(-(num_digits as i32));

        // Compute the float
        (int as f64) * divisor
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

    pub(crate) fn consume_ident(&mut self, err_msg: &str) -> IdentNode {
        match self.current.get_kind() {
            TokenKind::Ident => {
                let ident_expr = IdentNode::new(self.current.get_span(), self.get_ast_node_id());
                self.advance();
                ident_expr
            }
            _ => panic!("{}", err_msg),
        }
    }

    pub(crate) fn consume_ident_span(&mut self, err_msg: &str) -> Span {
        match self.current.get_kind() {
            TokenKind::Ident => {
                self.advance();
                self.prev.get_span()
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

    /*  TOKENKIND        PREFIX                 INFIX                               POSTFIX             */
    /*                   method     prec        method      prec                    method      prec    */
        LeftParen   = { (grouping   None),      (call       PrecCall        ),      (None       None) },
        RightParen  = { (None       None),      (None       None            ),      (None       None) },
        LeftCurly   = { (block_expr None),      (struct_expr PrecPrimary            ),      (None       None) },
        RightCurly  = { (None       None),      (None       None            ),      (None       None) },
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
        Dot         = { (dot_float  None),      (field_expr PrecCall        ),      (None       None) },
        Comma       = { (None       None),      (None       None            ),      (None       None) },
        Bang        = { (None       None),      (None       None            ),      (None       None) },
        Increment   = { (pre_inc    None),      (None       None            ),      (post_inc   None) },
        Decrement   = { (pre_dec    None),      (None       None            ),      (post_dec   None) },
        
            
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
        Fn          = { (None       None),      (None       None            ),      (None       None) },
        Typedef     = { (None       None),      (None       None            ),      (None       None) },
        Mut         = { (None       None),      (None       None            ),      (None       None) },
        Class       = { (None       None),      (None       None            ),      (None       None) },
        Struct      = { (None       None),      (None       None            ),      (None       None) },
        Enum        = { (None       None),      (None       None            ),      (None       None) },
        While       = { (None       None),      (None       None            ),      (None       None) },
        If          = { (if_expr    None),      (None       None            ),      (None       None) },
        Loop        = { (loop_expr  None),      (None       None            ),      (None       None) },
        Break       = { (None       None),      (None       None            ),      (None       None) },
        Continue    = { (None       None),      (None       None            ),      (None       None) },
        Return      = { (None       None),      (None       None            ),      (None       None) },
        Else        = { (None       None),      (None       None            ),      (None       None) },
        Elif        = { (None       None),      (None       None            ),      (None       None) },
        Eof         = { (None       None),      (None       None            ),      (None       None) }
        
        )
    }
}
