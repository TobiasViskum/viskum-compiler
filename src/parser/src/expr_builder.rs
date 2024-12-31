use ast::{
    AssignStmt,
    AstArenaObject,
    BinaryExpr,
    BlockExpr,
    BoolExpr,
    CallExpr,
    ConstExpr,
    DefineStmt,
    Expr,
    ExprWithBlock,
    ExprWithoutBlock,
    FieldExpr,
    FieldInitialization,
    GroupExpr,
    IdentNode,
    IfExpr,
    IndexExpr,
    IntegerExpr,
    LoopExpr,
    NullExpr,
    PkgIdentNode,
    PlaceExpr,
    Stmt,
    StringExpr,
    StructExpr,
    TupleExpr,
    TupleFieldExpr,
    ValueExpr,
};
use op::BinaryOp;
use span::Span;
use token::TokenKind;

use crate::{ precedence::Precedence, ParserHandle };

pub(crate) struct ExprBuilder<'ast, 'b> {
    final_stmt: Option<Stmt<'ast>>,
    exprs: Vec<Expr<'ast>>,
    /// Token that can be used to stop parsing the current expression
    /// Primarily used for parsing if|while <Expr> { ... }, since we don't want to parse '{' as an infix operator,
    /// And therefore terminate the expression at the '{' token
    pub terminate_infix_token: Option<TokenKind>,
    ast_arena: &'b AstArenaObject<'ast>,
    base_precedence: Precedence,
    mut_span: Option<Span>,
}

impl<'ast, 'b> ExprBuilder<'ast, 'b> {
    pub fn new(
        ast_arena: &'b AstArenaObject<'ast>,
        terminate_infix_token: Option<TokenKind>
    ) -> Self {
        Self {
            ast_arena,
            terminate_infix_token,
            exprs: Vec::with_capacity(32),
            final_stmt: None,
            base_precedence: Precedence::PrecAssign,
            mut_span: None,
        }
    }

    pub fn new_with_mut_span(
        ast_arena: &'b AstArenaObject<'ast>,
        terminate_infix_token: Option<TokenKind>,
        mut_span: Span
    ) -> Self {
        Self {
            ast_arena,
            terminate_infix_token,
            exprs: Vec::with_capacity(32),
            final_stmt: None,
            base_precedence: Precedence::PrecAssign,
            mut_span: Some(mut_span),
        }
    }

    pub fn take_stmt(mut self) -> Option<Stmt<'ast>> {
        if self.final_stmt.is_none() {
            self.final_stmt = Some(Stmt::ExprStmt(self.exprs.pop().expect("TODO: Error handling")));
        }

        self.final_stmt
    }

    pub fn take_expr(mut self) -> Option<Expr<'ast>> {
        self.exprs.pop()
    }

    pub fn emit_define_stmt(&mut self, parser_handle: &mut impl ParserHandle<'ast>) {
        let value_expr = self.exprs.pop().expect("TODO: Error handling");
        let pattern_expr = {
            let setter_expr = self.exprs.pop().expect("TODO: Error handling");
            parser_handle
                .try_as_pat(setter_expr)
                .expect("TODO: Error handling (invalid pattern expr)")
        };

        let define_stmt = self.ast_arena.alloc_expr_or_stmt(
            DefineStmt::new(
                self.mut_span,
                pattern_expr,
                value_expr,
                Span::dummy(),
                parser_handle.get_ast_node_id()
            )
        );

        self.final_stmt = Some(Stmt::DefineStmt(define_stmt));
    }

    pub fn emit_field_expr(
        &mut self,
        ident_node: IdentNode,
        parser_handle: &mut impl ParserHandle<'ast>
    ) {
        let field_expr = {
            let lhs = self.exprs.pop().expect("TODO: Error handling");
            let field_expr = FieldExpr::new(
                lhs,
                self.ast_arena.alloc_expr_or_stmt(ident_node),
                Span::dummy(),
                parser_handle.get_ast_node_id()
            );
            self.ast_arena.alloc_expr_or_stmt(field_expr)
        };

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::PlaceExpr(PlaceExpr::FieldExpr(field_expr))
        );
        self.exprs.push(expr);
    }

    pub fn emit_tuple_field_expr(
        &mut self,
        integer_expr: IntegerExpr,
        parser_handle: &mut impl ParserHandle<'ast>
    ) {
        let tuple_field_expr = {
            let lhs = self.exprs.pop().expect("TODO: Error handling");
            let tuple_field_expr = TupleFieldExpr::new(
                lhs,
                self.ast_arena.alloc_expr_or_stmt(integer_expr),
                Span::dummy(),
                parser_handle.get_ast_node_id()
            );
            self.ast_arena.alloc_expr_or_stmt(tuple_field_expr)
        };

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::PlaceExpr(PlaceExpr::TupleFieldExpr(tuple_field_expr))
        );
        self.exprs.push(expr);
    }

    pub fn emit_index_expr(
        &mut self,
        value_expr: Expr<'ast>,
        parser_handle: &mut impl ParserHandle<'ast>
    ) {
        let index_expr = {
            let lhs = self.exprs.pop().expect("TODO: Error handling");
            let index_expr = IndexExpr::new(
                lhs,
                value_expr,
                Span::dummy(),
                parser_handle.get_ast_node_id()
            );
            self.ast_arena.alloc_expr_or_stmt(index_expr)
        };

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::PlaceExpr(PlaceExpr::IndexExpr(index_expr))
        );

        self.exprs.push(expr);
    }

    pub fn emit_struct_expr(
        &mut self,
        initialization_fields: Vec<&'ast FieldInitialization<'ast>>,
        parser_handle: &mut impl ParserHandle<'ast>
    ) {
        let ident_node = {
            let ident_expr = self.exprs.pop().expect("TODO: Error handlings");
            parser_handle
                .try_as_ident(ident_expr)
                .expect("TODO: Error handling (expected ident in struct expr)")
        };

        let struct_expr = {
            let struct_expr = StructExpr::new(
                ident_node,
                self.ast_arena.alloc_vec(initialization_fields),
                Span::dummy(),
                parser_handle.get_ast_node_id()
            );

            self.ast_arena.alloc_expr_or_stmt(struct_expr)
        };

        self.exprs.push(
            Expr::ExprWithoutBlock(ExprWithoutBlock::ValueExpr(ValueExpr::StructExpr(struct_expr)))
        );
    }

    pub fn emit_assign_stmt(&mut self, parser_handle: &mut impl ParserHandle<'ast>) {
        let value_expr = self.exprs.pop().expect("TODO: Error handling");
        let setter_expr = self.exprs.pop().expect("TODO: Error handling");
        let place_expr = parser_handle
            .try_as_asignee_expr(setter_expr)
            .expect("TODO: Error handling (invalid pattern expr)");

        let assign_stmt = self.ast_arena.alloc_expr_or_stmt(
            AssignStmt::new(place_expr, value_expr, Span::dummy(), parser_handle.get_ast_node_id())
        );

        self.final_stmt = Some(Stmt::AssignStmt(assign_stmt));
    }

    pub fn emit_call_expr(
        &mut self,
        parser_handle: &mut impl ParserHandle<'ast>,
        args: Vec<Expr<'ast>>
    ) {
        let callee = self.exprs.pop().expect("TODO: Error handling");
        let args = self.ast_arena.alloc_vec(args);

        let call_expr = self.ast_arena.alloc_expr_or_stmt(
            CallExpr::new(callee, args, Span::dummy(), parser_handle.get_ast_node_id())
        );

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(ValueExpr::CallExpr(call_expr))
        );

        self.exprs.push(expr);
    }

    pub fn emit_loop_expr(&mut self, loop_expr: &'ast LoopExpr<'ast>) {
        let expr = Expr::ExprWithBlock(ExprWithBlock::LoopExpr(loop_expr));
        self.exprs.push(expr);
    }

    pub fn emit_if_expr(&mut self, if_expr: &'ast IfExpr<'ast>) {
        let expr = Expr::ExprWithBlock(ExprWithBlock::IfExpr(if_expr));

        self.exprs.push(expr);
    }

    pub fn emit_block_expr(&mut self, block_expr: &'ast BlockExpr<'ast>) {
        let expr = Expr::ExprWithBlock(ExprWithBlock::BlockExpr(block_expr));
        self.exprs.push(expr);
    }

    pub fn emit_grouping_or_tuple_expr(
        &mut self,
        parser_handle: &mut impl ParserHandle<'ast>,
        exprs: Vec<Expr<'ast>>
    ) {
        assert!(!exprs.is_empty(), "Expected at least one expr in group (got 0)");

        // Now we have a regular grouping expr
        if exprs.len() == 1 {
            let group_expr = self.ast_arena.alloc_expr_or_stmt(
                GroupExpr::new(exprs[0], Span::dummy(), parser_handle.get_ast_node_id())
            );
            let expr = Expr::ExprWithoutBlock(
                ExprWithoutBlock::ValueExpr(ValueExpr::GroupExpr(group_expr))
            );
            self.exprs.push(expr);
        } else {
            let fields = self.ast_arena.alloc_vec(exprs);
            let tuple_expr = self.ast_arena.alloc_expr_or_stmt(
                TupleExpr::new(fields, Span::dummy(), parser_handle.get_ast_node_id())
            );
            let expr = Expr::ExprWithoutBlock(
                ExprWithoutBlock::ValueExpr(ValueExpr::TupleExpr(tuple_expr))
            );
            self.exprs.push(expr);
        }
    }

    pub fn emit_pkg_ident_expr(&mut self, pkg_ident_expr: PkgIdentNode) {
        let pkg_ident_expr = self.ast_arena.alloc_expr_or_stmt(pkg_ident_expr);

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::PlaceExpr(PlaceExpr::PkgIdentExpr(pkg_ident_expr))
        );

        self.exprs.push(expr);
    }

    pub fn emit_ident_expr(&mut self, ident_node: IdentNode) {
        let ident_node = self.ast_arena.alloc_expr_or_stmt(ident_node);

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::PlaceExpr(PlaceExpr::IdentExpr(ident_node))
        );

        self.exprs.push(expr);
    }

    pub fn emit_string_expr(&mut self, string_expr: StringExpr) {
        let string_expr = self.ast_arena.alloc_expr_or_stmt(string_expr);

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(ValueExpr::ConstExpr(ConstExpr::StringExpr(string_expr)))
        );

        self.exprs.push(expr);
    }

    pub fn emit_null_expr(&mut self, null_expr: NullExpr) {
        let null_expr = self.ast_arena.alloc_expr_or_stmt(null_expr);

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(ValueExpr::ConstExpr(ConstExpr::NullExpr(null_expr)))
        );

        self.exprs.push(expr);
    }

    pub fn emit_bool_expr(&mut self, bool_expr: BoolExpr) {
        let bool_expr = self.ast_arena.alloc_expr_or_stmt(bool_expr);

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(ValueExpr::ConstExpr(ConstExpr::BoolExpr(bool_expr)))
        );

        self.exprs.push(expr);
    }

    pub fn emit_integer_expr(&mut self, integer_expr: IntegerExpr) {
        let integer_expr = self.ast_arena.alloc_expr_or_stmt(integer_expr);

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(ValueExpr::ConstExpr(ConstExpr::IntegerExpr(integer_expr)))
        );

        self.exprs.push(expr);
    }

    pub fn emit_binary_expr(&mut self, op: BinaryOp, parser_handle: &mut impl ParserHandle<'ast>) {
        let rhs = self.exprs.pop().expect("TODO: Error handling");
        let lhs = self.exprs.pop().expect("TODO: Error handling");

        let binary_expr = self.ast_arena.alloc_expr_or_stmt(
            BinaryExpr::new(lhs, op, rhs, Span::dummy(), parser_handle.get_ast_node_id())
        );

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(ValueExpr::BinaryExpr(binary_expr))
        );

        self.exprs.push(expr);
    }

    pub fn emit_post_inc_expr(&mut self, parser_handle: &mut impl ParserHandle<'ast>) {
        todo!()

        // let expr = self.exprs.pop().expect("TODO: Error handling");

        // let assign_stmt = Stmt::AssignStmt(
        //     self.ast_arena.alloc_expr_or_stmt(
        //         AssignStmt::new(
        //             parser_handle.try_as_place_expr(expr).expect("TODO: Error handling"),
        //             Expr::ExprWithoutBlock(
        //                 ExprWithoutBlock::ValueExpr(
        //                     ValueExpr::BinaryExpr(
        //                         self.ast_arena.alloc_expr_or_stmt(
        //                             BinaryExpr::new(
        //                                 expr,
        //                                 BinaryOp::ArithmeticOp(ArithmeticOp::Add),
        //                                 Expr::ExprWithoutBlock(
        //                                     ExprWithoutBlock::ValueExpr(
        //                                         ValueExpr::ConstExpr(
        //                                             ConstExpr::IntegerExpr(
        //                                                 self.ast_arena.alloc_expr_or_stmt(
        //                                                     IntegerExpr::new(
        //                                                         1,
        //                                                         parser_handle.get_ast_node_id()
        //                                                     )
        //                                                 )
        //                                             )
        //                                         )
        //                                     )
        //                                 ),
        //                                 parser_handle.get_ast_node_id()
        //                             )
        //                         )
        //                     )
        //                 )
        //             ),
        //             parser_handle.get_ast_node_id(),
        //             Span::dummy()
        //         )
        //     )
        // );

        // let minus_one_stmt = Stmt::ExprStmt(
        //     Expr::ExprWithoutBlock(
        //         ExprWithoutBlock::ValueExpr(
        //             ValueExpr::BinaryExpr(
        //                 self.ast_arena.alloc_expr_or_stmt(
        //                     BinaryExpr::new(
        //                         expr,
        //                         BinaryOp::ArithmeticOp(ArithmeticOp::Sub),
        //                         Expr::ExprWithoutBlock(
        //                             ExprWithoutBlock::ValueExpr(
        //                                 ValueExpr::ConstExpr(
        //                                     ConstExpr::IntegerExpr(
        //                                         self.ast_arena.alloc_expr_or_stmt(
        //                                             IntegerExpr::new(
        //                                                 1,
        //                                                 parser_handle.get_ast_node_id()
        //                                             )
        //                                         )
        //                                     )
        //                                 )
        //                             )
        //                         ),
        //                         parser_handle.get_ast_node_id()
        //                     )
        //                 )
        //             )
        //         )
        //     )
        // );

        // let stmts = vec![assign_stmt, minus_one_stmt];

        // let block_expr = self.ast_arena.alloc_expr_or_stmt(
        //     BlockExpr::new(self.ast_arena.alloc_vec(stmts), parser_handle.get_ast_node_id())
        // );

        // let expr = Expr::ExprWithBlock(ExprWithBlock::BlockExpr(block_expr));

        // self.exprs.push(expr);
    }

    pub fn get_base_prec(&self) -> Precedence {
        self.base_precedence
    }

    pub fn set_base_prec(&mut self, prec: Precedence) {
        self.base_precedence = prec;
    }
}
