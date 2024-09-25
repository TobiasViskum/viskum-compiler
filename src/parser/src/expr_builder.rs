use ast::{
    AssignStmt,
    AstArena,
    BinaryExpr,
    BlockExpr,
    BoolExpr,
    ConstExpr,
    DefineStmt,
    Expr,
    ExprKind,
    ExprWithBlock,
    ExprWithoutBlock,
    GroupExpr,
    IdentExpr,
    IfExpr,
    IntegerExpr,
    Pat,
    PatKind,
    PlaceExpr,
    PlaceKind,
    Stmt,
    ValueExpr,
};
use ir_defs::NodeId;
use op::BinaryOp;
use span::Span;

use crate::precedence::Precedence;

pub struct ExprBuilder<'ast> {
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
            self.final_stmt = Some(
                Stmt::ExprStmt(
                    self.ast_arena.alloc_expr_or_stmt(
                        self.exprs.pop().expect("TODO: Error handling")
                    )
                )
            );
        }

        self.final_stmt
    }

    pub fn take_expr(mut self) -> Option<Expr<'ast>> {
        self.exprs.pop()
    }

    pub fn emit_define_stmt(&mut self, ast_node_id: NodeId) {
        let value_expr = self.exprs.pop().expect("TODO: Error handling");
        let setter_expr = self.exprs.pop().expect("TODO: Error handling");
        let pattern_expr = self
            .try_as_pattern_expr(&setter_expr)
            .expect("TODO: Error handling (invalid pattern expr)");

        let define_stmt = self.ast_arena.alloc_expr_or_stmt(
            DefineStmt::new(
                self.ast_arena.alloc_expr_or_stmt(pattern_expr),
                self.ast_arena.alloc_expr_or_stmt(value_expr),
                ast_node_id
            )
        );

        self.final_stmt = Some(Stmt::DefineStmt(define_stmt));
    }

    pub fn emit_assign_stmt(&mut self, ast_node_id: NodeId) {
        let value_expr = self.exprs.pop().expect("TODO: Error handling");
        let setter_expr = self.exprs.pop().expect("TODO: Error handling");
        let place_expr = self
            .try_as_place_expr(setter_expr)
            .expect("TODO: Error handling (invalid pattern expr)");

        let assign_stmt = self.ast_arena.alloc_expr_or_stmt(
            AssignStmt::new(
                self.ast_arena.alloc_expr_or_stmt(place_expr),
                self.ast_arena.alloc_expr_or_stmt(value_expr),
                ast_node_id,
                Span::dummy()
            )
        );

        self.final_stmt = Some(Stmt::AssignStmt(assign_stmt));
    }

    pub fn emit_if_expr(&mut self, if_expr: &'ast IfExpr<'ast>, ast_node_id: NodeId) {
        let expr_kind = ExprKind::ExprWithBlock(ExprWithBlock::IfExpr(if_expr));
        let expr = Expr::new(expr_kind, ast_node_id);

        self.exprs.push(expr);
    }

    pub fn emit_block_expr(&mut self, block_expr: &'ast BlockExpr<'ast>, ast_node_id: NodeId) {
        let expr_kind = ExprKind::ExprWithBlock(ExprWithBlock::BlockExpr(block_expr));
        let expr = Expr::new(expr_kind, ast_node_id);

        self.exprs.push(expr);
    }

    pub fn emit_grouping_expr(&mut self, ast_node_id: NodeId) {
        let group_expr = self.exprs.pop().expect("TODO: Error handling");
        let group_expr = self.ast_arena.alloc_expr_or_stmt(
            GroupExpr::new(self.ast_arena.alloc_expr_or_stmt(group_expr), ast_node_id)
        );

        let expr_kind = ExprKind::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(ValueExpr::GroupExpr(group_expr))
        );
        let expr = Expr::new(expr_kind, ast_node_id);

        self.exprs.push(expr);
    }

    pub fn emit_ident_expr(
        &mut self,
        ident_expr: IdentExpr,
        ast_node_id_1: NodeId,
        ast_node_id_2: NodeId
    ) {
        let ident_expr = self.ast_arena.alloc_expr_or_stmt(ident_expr);

        let expr_kind = ExprKind::ExprWithoutBlock(
            ExprWithoutBlock::PlaceExpr(
                self.ast_arena.alloc_expr_or_stmt(
                    PlaceExpr::new(PlaceKind::IdentExpr(ident_expr), ast_node_id_1)
                )
            )
        );
        let expr = Expr::new(expr_kind, ast_node_id_2);

        self.exprs.push(expr);
    }

    pub fn emit_bool_expr(&mut self, bool_expr: BoolExpr, ast_node_id: NodeId) {
        let bool_expr = self.ast_arena.alloc_expr_or_stmt(bool_expr);

        let expr_kind = ExprKind::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(ValueExpr::ConstExpr(ConstExpr::BoolExpr(bool_expr)))
        );
        let expr = Expr::new(expr_kind, ast_node_id);

        self.exprs.push(expr);
    }

    pub fn emit_integer_expr(&mut self, integer_expr: IntegerExpr, ast_node_id: NodeId) {
        let integer_expr = self.ast_arena.alloc_expr_or_stmt(integer_expr);

        let expr_kind = ExprKind::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(ValueExpr::ConstExpr(ConstExpr::IntegerExpr(integer_expr)))
        );
        let expr = Expr::new(expr_kind, ast_node_id);

        self.exprs.push(expr);
    }

    pub fn emit_binary_expr(&mut self, op: BinaryOp, ast_node_id: NodeId) {
        let rhs = self.exprs.pop().expect("TODO: Error handling");
        let lhs = self.exprs.pop().expect("TODO: Error handling");

        let binary_expr = self.ast_arena.alloc_expr_or_stmt(
            BinaryExpr::new(
                self.ast_arena.alloc_expr_or_stmt(lhs),
                op,
                self.ast_arena.alloc_expr_or_stmt(rhs),
                ast_node_id
            )
        );

        let expr_kind = ExprKind::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(ValueExpr::BinaryExpr(binary_expr))
        );
        let expr = Expr::new(expr_kind, ast_node_id);

        self.exprs.push(expr);
    }

    pub fn get_base_prec(&self) -> Precedence {
        self.base_precedence
    }

    pub fn set_base_prec(&mut self, prec: Precedence) {
        self.base_precedence = prec;
    }

    fn try_as_pattern_expr(&mut self, expr: &Expr<'ast>) -> Option<Pat<'ast>> {
        let ast_node_id = expr.ast_node_id;
        match &expr.kind {
            ExprKind::ExprWithBlock(_) => None,
            ExprKind::ExprWithoutBlock(expr) => {
                match expr {
                    ExprWithoutBlock::PlaceExpr(expr) => {
                        match &expr.kind {
                            PlaceKind::IdentExpr(ident_expr) =>
                                Some(
                                    Pat::new(
                                        PatKind::IdentPat(
                                            self.ast_arena.alloc_expr_or_stmt(
                                                ident_expr.get_as_pat()
                                            )
                                        ),
                                        ast_node_id
                                    )
                                ),
                        }
                    }
                    ExprWithoutBlock::ValueExpr(expr) => {
                        match expr {
                            ValueExpr::BinaryExpr(_) => None,
                            ValueExpr::ConstExpr(_) => None,
                            ValueExpr::GroupExpr(_) => None,
                        }
                    }
                }
            }
        }
    }

    fn try_as_place_expr(&mut self, expr: Expr<'ast>) -> Option<PlaceExpr<'ast>> {
        let ast_node_id = expr.ast_node_id;
        match expr.kind {
            ExprKind::ExprWithBlock(_) => None,
            ExprKind::ExprWithoutBlock(expr) => {
                match expr {
                    ExprWithoutBlock::PlaceExpr(expr) => {
                        Some(PlaceExpr::new(expr.kind, ast_node_id))
                    }
                    ExprWithoutBlock::ValueExpr(expr) => {
                        match expr {
                            ValueExpr::BinaryExpr(_) => None,
                            ValueExpr::ConstExpr(_) => None,
                            ValueExpr::GroupExpr(_) => None,
                        }
                    }
                }
            }
        }
    }
}
