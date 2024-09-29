use ast::{
    AssignStmt,
    AstArena,
    BinaryExpr,
    BlockExpr,
    BoolExpr,
    ConstExpr,
    DefineStmt,
    Expr,
    ExprWithBlock,
    ExprWithoutBlock,
    GroupExpr,
    IdentExpr,
    IfExpr,
    IntegerExpr,
    LoopExpr,
    Pat,
    PlaceExpr,
    Stmt,
    TupleExpr,
    TupleFieldExpr,
    ValueExpr,
};
use op::BinaryOp;
use span::Span;

use crate::{ precedence::Precedence, ParserHandle };

pub(crate) struct ExprBuilder<'ast> {
    final_stmt: Option<Stmt<'ast>>,
    exprs: Vec<Expr<'ast>>,
    ast_arena: &'ast AstArena,
    base_precedence: Precedence,
}

impl<'ast> ExprBuilder<'ast> {
    pub fn new(ast_arena: &'ast AstArena) -> Self {
        Self {
            ast_arena,
            exprs: Vec::with_capacity(32),
            final_stmt: None,
            base_precedence: Precedence::PrecAssign,
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

    pub fn emit_define_stmt(&mut self, parser_handle: &mut impl ParserHandle) {
        let value_expr = self.exprs.pop().expect("TODO: Error handling");
        let setter_expr = self.exprs.pop().expect("TODO: Error handling");
        let pattern_expr = self
            .try_as_pattern_expr(setter_expr)
            .expect("TODO: Error handling (invalid pattern expr)");

        let define_stmt = self.ast_arena.alloc_expr_or_stmt(
            DefineStmt::new(pattern_expr, value_expr, parser_handle.get_ast_node_id())
        );

        self.final_stmt = Some(Stmt::DefineStmt(define_stmt));
    }

    pub fn emit_tuple_field_expr(
        &mut self,
        integer_expr: IntegerExpr,
        parser_handle: &mut impl ParserHandle
    ) {
        let lhs = self.exprs.pop().expect("TODO: Error handling");
        let tuple_field_expr = TupleFieldExpr::new(
            lhs,
            self.ast_arena.alloc_expr_or_stmt(integer_expr),
            parser_handle.get_ast_node_id()
        );

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::PlaceExpr(
                PlaceExpr::TupleFieldExpr(self.ast_arena.alloc_expr_or_stmt(tuple_field_expr))
            )
        );
        self.exprs.push(expr);
    }

    pub fn emit_assign_stmt(&mut self, parser_handle: &mut impl ParserHandle) {
        let value_expr = self.exprs.pop().expect("TODO: Error handling");
        let setter_expr = self.exprs.pop().expect("TODO: Error handling");
        let place_expr = self
            .try_as_place_expr(setter_expr)
            .expect("TODO: Error handling (invalid pattern expr)");

        let assign_stmt = self.ast_arena.alloc_expr_or_stmt(
            AssignStmt::new(place_expr, value_expr, parser_handle.get_ast_node_id(), Span::dummy())
        );

        self.final_stmt = Some(Stmt::AssignStmt(assign_stmt));
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
        parser_handle: &mut impl ParserHandle,
        exprs: Vec<Expr<'ast>>
    ) {
        assert_eq!(true, exprs.len() > 0, "Expected at least one expr in group (got 0)");

        // Now we have a regular grouping expr
        if exprs.len() == 1 {
            let group_expr = self.ast_arena.alloc_expr_or_stmt(
                GroupExpr::new(exprs[0], parser_handle.get_ast_node_id())
            );
            let expr = Expr::ExprWithoutBlock(
                ExprWithoutBlock::ValueExpr(ValueExpr::GroupExpr(group_expr))
            );
            self.exprs.push(expr);
        } else {
            let fields = self.ast_arena.alloc_vec_exprs(exprs);
            let tuple_expr = self.ast_arena.alloc_expr_or_stmt(
                TupleExpr::new(fields, parser_handle.get_ast_node_id())
            );
            let expr = Expr::ExprWithoutBlock(
                ExprWithoutBlock::ValueExpr(ValueExpr::TupleExpr(tuple_expr))
            );
            self.exprs.push(expr);
        }
    }

    pub fn emit_ident_expr(&mut self, ident_expr: IdentExpr) {
        let ident_expr = self.ast_arena.alloc_expr_or_stmt(ident_expr);

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::PlaceExpr(PlaceExpr::IdentExpr(ident_expr))
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

    pub fn emit_binary_expr(&mut self, op: BinaryOp, parser_handle: &mut impl ParserHandle) {
        let rhs = self.exprs.pop().expect("TODO: Error handling");
        let lhs = self.exprs.pop().expect("TODO: Error handling");

        let binary_expr = self.ast_arena.alloc_expr_or_stmt(
            BinaryExpr::new(lhs, op, rhs, parser_handle.get_ast_node_id())
        );

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(ValueExpr::BinaryExpr(binary_expr))
        );

        self.exprs.push(expr);
    }

    pub fn get_base_prec(&self) -> Precedence {
        self.base_precedence
    }

    pub fn set_base_prec(&mut self, prec: Precedence) {
        self.base_precedence = prec;
    }

    fn try_as_pattern_expr(&mut self, expr: Expr<'ast>) -> Option<Pat<'ast>> {
        match expr {
            Expr::ExprWithBlock(_) => None,
            Expr::ExprWithoutBlock(expr) => {
                match expr {
                    ExprWithoutBlock::PlaceExpr(expr) => {
                        match expr {
                            PlaceExpr::TupleFieldExpr(_) => None,
                            PlaceExpr::IdentExpr(ident_expr) =>
                                Some(
                                    Pat::IdentPat(
                                        self.ast_arena.alloc_expr_or_stmt(ident_expr.get_as_pat())
                                    )
                                ),
                        }
                    }
                    ExprWithoutBlock::BreakExpr(_) => None,
                    ExprWithoutBlock::ContinueExpr(_) => None,
                    ExprWithoutBlock::ValueExpr(expr) => {
                        match expr {
                            ValueExpr::BinaryExpr(_) => None,
                            ValueExpr::ConstExpr(_) => None,
                            ValueExpr::GroupExpr(_) => None,
                            ValueExpr::TupleExpr(tuple_expr) =>
                                todo!("As pattern: {:#?}", tuple_expr),
                        }
                    }
                }
            }
        }
    }

    fn try_as_place_expr(&mut self, expr: Expr<'ast>) -> Option<PlaceExpr<'ast>> {
        match expr {
            Expr::ExprWithBlock(_) => None,
            Expr::ExprWithoutBlock(expr) => {
                match expr {
                    ExprWithoutBlock::PlaceExpr(expr) => { Some(expr) }
                    ExprWithoutBlock::BreakExpr(_) => None,
                    ExprWithoutBlock::ContinueExpr(_) => None,
                    ExprWithoutBlock::ValueExpr(expr) => {
                        match expr {
                            ValueExpr::BinaryExpr(_) => None,
                            ValueExpr::ConstExpr(_) => None,
                            ValueExpr::GroupExpr(_) => None,
                            ValueExpr::TupleExpr(tuple_expr) =>
                                todo!("As place expr: {:#?}", tuple_expr),
                        }
                    }
                }
            }
        }
    }
}
