use ast_arena::AstArena;
use expr::{
    BinaryExpr,
    ConstExpr,
    DefineStmt,
    Expr,
    ExprWithoutBlock,
    GroupExpr,
    IdentExpr,
    IntegerExpr,
    PlaceExpr,
    Stmt,
    ValueExpr,
};
use op::BinaryOp;
use precedence::Precedence;

#[derive(Debug)]
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

    pub fn take_stmt(self) -> Option<Stmt<'ast>> {
        self.final_stmt
    }

    pub fn emit_define_stmt(&mut self) {
        let value_expr = self.exprs.pop().expect("TODO: Error handling");
        let setter_expr = self.exprs.pop().expect("TODO: Error handling");
        let define_stmt = Stmt::DefineStmt(DefineStmt::new(setter_expr, value_expr));
        self.final_stmt = Some(define_stmt);
    }

    pub fn emit_group(&mut self) {
        let group_expr = self.exprs.pop().expect("TODO: Error handling");
        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(
                ValueExpr::GruopExpr(GroupExpr::new(self.ast_arena.alloc_expr(group_expr)))
            )
        );
        self.exprs.push(expr);
    }

    pub fn emit_ident(&mut self, ident_expr: IdentExpr) {
        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::PlaceExpr(PlaceExpr::IdentExpr(ident_expr))
        );
        self.exprs.push(expr);
    }

    pub fn emit_integer(&mut self, integer_expr: IntegerExpr) {
        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(ValueExpr::ConstExpr(ConstExpr::IntegerExpr(integer_expr)))
        );
        self.exprs.push(expr);
    }

    pub fn emit_binary(&mut self, op: BinaryOp) {
        let rhs = self.exprs.pop().expect("TODO: Error handling");
        let lhs = self.exprs.pop().expect("TODO: Error handling");

        let expr = Expr::ExprWithoutBlock(
            ExprWithoutBlock::ValueExpr(
                ValueExpr::BinaryExpr(
                    BinaryExpr::new(
                        self.ast_arena.alloc_expr(lhs),
                        op,
                        self.ast_arena.alloc_expr(rhs)
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
}
