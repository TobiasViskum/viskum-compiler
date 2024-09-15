use ty::Ty;

use crate::{ ast_state::AstResolved, ast_visitor::{ AstVisitor, OnAstVisit }, * };

impl<'ast> OnAstVisit<'ast, AstResolved> for AstVisitor<'ast, AstResolved> {
    type VisitEvent = usize;
    type VisitReturn = Ty;

    fn on_leave_block(
        &mut self,
        block_expr: &'ast BlockExpr<'ast, AstResolved>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}

    fn on_visit_binary_expr(
        &mut self,
        expr: &'ast BinaryExpr<'ast, AstResolved>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}

    fn on_visit_block(
        &mut self,
        block_expr: &'ast BlockExpr<'ast, AstResolved>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}

    fn on_visit_const_expr(&mut self, expr: &'ast ConstExpr, f: &mut dyn FnMut(Self::VisitEvent)) {}

    fn on_visit_def_stmt(
        &mut self,
        def_stmt: &'ast DefineStmt<'ast, AstResolved>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_expr(
        &mut self,
        expr: &'ast Expr<'ast, AstResolved>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_expr_with_block(
        &mut self,
        expr: &'ast ExprWithBlock<'ast, AstResolved>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_expr_without_block(
        &mut self,
        expr: &'ast ExprWithoutBlock<'ast, AstResolved>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_group_expr(
        &mut self,
        expr: &'ast GroupExpr<'ast, AstResolved>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_ident_expr(
        &mut self,
        expr: &'ast IdentExpr<AstResolved>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_ident_pat(
        &mut self,
        pat: &'ast IdentPat<AstResolved>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_if_expr(
        &mut self,
        expr: &'ast IfExpr<'ast, AstResolved>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_pat(&mut self, pat: &'ast Pat<AstResolved>, f: &mut dyn FnMut(Self::VisitEvent)) {}
    fn on_visit_place_expr(
        &mut self,
        expr: &'ast PlaceExpr<AstResolved>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_stmt(
        &mut self,
        stmt: &'ast Stmt<'ast, AstResolved>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_stmts(
        &mut self,
        stmts: &'ast [&'ast Stmt<'ast, AstResolved>],
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_value_expr(
        &mut self,
        expr: &'ast ValueExpr<'ast, AstResolved>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
}
