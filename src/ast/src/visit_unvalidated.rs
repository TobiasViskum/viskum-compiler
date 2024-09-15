use crate::{
    ast_state::{ AstState, AstUnvalidated },
    ast_visitor::{ AstVisitor, OnAstVisit },
    BinaryExpr,
    BlockExpr,
    ConstExpr,
    DefineStmt,
    Expr,
    ExprWithBlock,
    ExprWithoutBlock,
    GroupExpr,
    IdentExpr,
    IdentPat,
    IfExpr,
    Pat,
    PlaceExpr,
    Stmt,
    ValueExpr,
};

#[derive(Debug, Clone, Copy)]
pub enum AstVisitEvent<'ast, T> where T: AstState {
    ScopeChange(ScopeChange),
    AstNode(AstNodeKind<'ast, T>),
}

#[derive(Debug, Clone, Copy)]
pub enum AstNodeKind<'ast, T> where T: AstState {
    Define(&'ast IdentPat<T>),
    Lookup(&'ast IdentExpr<T>),
}

#[derive(Debug, Clone, Copy)]
pub enum ScopeChange {
    Increment,
    Decrement,
}

impl<'ast> OnAstVisit<'ast, AstUnvalidated> for AstVisitor<'ast, AstUnvalidated> {
    type VisitEvent = AstVisitEvent<'ast, AstUnvalidated>;
    type VisitReturn = ();

    fn on_visit_binary_expr(
        &mut self,
        expr: &'ast BinaryExpr<'ast, AstUnvalidated>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}

    fn on_visit_block(
        &mut self,
        block_expr: &'ast BlockExpr<'ast, AstUnvalidated>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {
        f(AstVisitEvent::<AstUnvalidated>::ScopeChange(ScopeChange::Increment))
    }

    fn on_leave_block(
        &mut self,
        block_expr: &'ast BlockExpr<'ast, AstUnvalidated>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {
        f(AstVisitEvent::<AstUnvalidated>::ScopeChange(ScopeChange::Decrement))
    }

    fn on_visit_const_expr(&mut self, expr: &'ast ConstExpr, f: &mut dyn FnMut(Self::VisitEvent)) {}

    fn on_visit_def_stmt(
        &mut self,
        def_stmt: &'ast DefineStmt<'ast, AstUnvalidated>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}

    fn on_visit_expr(
        &mut self,
        expr: &'ast Expr<'ast, AstUnvalidated>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_expr_with_block(
        &mut self,
        expr: &'ast ExprWithBlock<'ast, AstUnvalidated>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_expr_without_block(
        &mut self,
        expr: &'ast ExprWithoutBlock<'ast, AstUnvalidated>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_group_expr(
        &mut self,
        expr: &'ast GroupExpr<'ast, AstUnvalidated>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_ident_expr(
        &mut self,
        expr: &'ast IdentExpr<AstUnvalidated>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {
        f(AstVisitEvent::AstNode(AstNodeKind::Lookup(expr)))
    }
    fn on_visit_ident_pat(
        &mut self,
        pat: &'ast IdentPat<AstUnvalidated>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {
        f(AstVisitEvent::AstNode(AstNodeKind::Define(pat)))
    }
    fn on_visit_if_expr(
        &mut self,
        expr: &'ast IfExpr<'ast, AstUnvalidated>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_pat(
        &mut self,
        pat: &'ast Pat<AstUnvalidated>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_place_expr(
        &mut self,
        expr: &'ast PlaceExpr<AstUnvalidated>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_stmt(
        &mut self,
        stmt: &'ast Stmt<'ast, AstUnvalidated>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_stmts(
        &mut self,
        stmts: &'ast [&'ast Stmt<'ast, AstUnvalidated>],
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
    fn on_visit_value_expr(
        &mut self,
        expr: &'ast ValueExpr<'ast, AstUnvalidated>,
        f: &mut dyn FnMut(Self::VisitEvent)
    ) {}
}
