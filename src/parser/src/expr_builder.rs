use ast::{
    ast_state::AstUnvalidated,
    AssignStmt,
    AstArena,
    BinaryExpr,
    BlockExpr,
    ConstExpr,
    DefineStmt,
    Expr,
    ExprWithBlock,
    ExprWithoutBlock,
    GroupExpr,
    IdentExpr,
    IfExpr,
    IntegerExpr,
    Pat,
    PlaceExpr,
    Stmt,
    ValueExpr,
};
use op::BinaryOp;
use ty::NodeId;

use crate::precedence::Precedence;

pub struct ExprBuilder<'ast> {
    final_stmt: Option<Stmt<'ast>>,
    exprs: Vec<Expr<'ast>>,
    ast_arena: &'ast AstArena<'ast>,
    base_precedence: Precedence,
}

impl<'ast> ExprBuilder<'ast> {
    pub fn new(ast_arena: &'ast AstArena<'ast>) -> Self {
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

    pub fn emit_define_stmt(&mut self, ast_node_id: NodeId) {
        let value_expr = self.exprs.pop().expect("TODO: Error handling");
        let setter_expr = self.exprs.pop().expect("TODO: Error handling");
        let pattern_expr = Self::try_as_pattern_expr(&setter_expr).expect(
            "TODO: Error handling (invalid pattern expr)"
        );

        let define_stmt = Stmt::DefineStmt(DefineStmt::new(pattern_expr, value_expr, ast_node_id));
        self.final_stmt = Some(define_stmt);
    }

    pub fn emit_assign_stmt(&mut self, ast_node_id: NodeId) {
        let value_expr = self.exprs.pop().expect("TODO: Error handling");
        let setter_expr = self.exprs.pop().expect("TODO: Error handling");
        let place_expr = Self::try_as_place_expr(&setter_expr).expect(
            "TODO: Error handling (invalid pattern expr)"
        );

        let assign_stmt = Stmt::AssignStmt(AssignStmt::new(place_expr, value_expr, ast_node_id));
        self.final_stmt = Some(assign_stmt);
    }

    pub fn emit_if_expr(&mut self, if_expr: &'ast IfExpr<'ast>) {
        let expr = Expr::ExprWithBlock(ExprWithBlock::IfExpr(if_expr));

        self.exprs.push(expr);
    }

    pub fn emit_block_expr(&mut self, block_expr: &'ast BlockExpr<'ast>) {
        let expr = Expr::ExprWithBlock(ExprWithBlock::BlockExpr(block_expr));

        self.exprs.push(expr);
    }

    pub fn emit_grouping_expr(&mut self, ast_node_id: NodeId) {
        let group_expr = self.exprs.pop().expect("TODO: Error handling");
        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(
                ValueExpr::GroupExpr(
                    GroupExpr::new(self.ast_arena.alloc_expr_or_stmt(group_expr), ast_node_id)
                )
            )
        );
        self.exprs.push(expr);
    }

    pub fn emit_ident_expr(&mut self, ident_expr: IdentExpr) {
        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::PlaceExpr(PlaceExpr::IdentExpr(ident_expr))
        );
        self.exprs.push(expr);
    }

    pub fn emit_integer_expr(&mut self, integer_expr: IntegerExpr) {
        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(ValueExpr::ConstExpr(ConstExpr::IntegerExpr(integer_expr)))
        );
        self.exprs.push(expr);
    }

    pub fn emit_binary_expr(&mut self, op: BinaryOp, ast_node_id: NodeId) {
        let rhs = self.exprs.pop().expect("TODO: Error handling");
        let lhs = self.exprs.pop().expect("TODO: Error handling");

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(
                ValueExpr::BinaryExpr(
                    BinaryExpr::new(
                        self.ast_arena.alloc_expr_or_stmt(lhs),
                        op,
                        self.ast_arena.alloc_expr_or_stmt(rhs),
                        ast_node_id
                    )
                )
            )
        );

        self.exprs.push(expr);
    }

    pub fn get_base_prec(&self) -> Precedence {
        self.base_precedence
    }

    pub fn set_base_prec(&mut self, prec: Precedence) {
        self.base_precedence = prec;
    }

    fn try_as_pattern_expr(expr: &Expr<'ast>) -> Option<Pat> {
        match expr {
            Expr::ExprWithBlock(_) => None,
            Expr::ExprWithoutBlock(expr) => {
                match expr {
                    ExprWithoutBlock::PlaceExpr(expr) => {
                        match expr {
                            PlaceExpr::IdentExpr(ident_expr) =>
                                Some(Pat::IdentPat(ident_expr.as_pat())),
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

    fn try_as_place_expr(expr: &Expr<'ast>) -> Option<PlaceExpr> {
        match expr {
            Expr::ExprWithBlock(_) => None,
            Expr::ExprWithoutBlock(expr) => {
                match expr {
                    ExprWithoutBlock::PlaceExpr(expr) => { Some(*expr) }
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
