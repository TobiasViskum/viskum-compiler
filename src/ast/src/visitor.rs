/*

This specific visitor pattern, specifically implemented in the Visitor trait,
is inspired by how the Rust compiler visits its Ast:
https://github.com/rust-lang/rust/blob/master/compiler/rustc_ast/src/visit.rs

*/

use crate::{
    AssignStmt,
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
    IdentPat,
    IfExpr,
    IfFalseBranchExpr,
    IntegerExpr,
    ItemStmt,
    Pat,
    PatKind,
    PlaceExpr,
    PlaceKind,
    Stmt,
    ValueExpr,
};

/// A default implementation exists for all ast nodes, meaning
/// the whole ast will be visited from left to right by default
///
/// Each implementation can be overwritten for each state the Ast can be in
/// (e.g. Ast\<AstUnresolved\> or Ast\<AstResolved\>)
pub trait Visitor<'ast>: Sized {
    type Result: Sized;

    fn default_result() -> Self::Result;

    /* Root nodes, uses a default implementation */
    #[allow(unused_variables)]
    fn visit_ident_pat(&mut self, ident_pat: &'ast IdentPat) -> Self::Result {
        Self::default_result()
    }
    #[allow(unused_variables)]
    fn visit_interger_expr(&mut self, interger_expr: &'ast IntegerExpr) -> Self::Result {
        Self::default_result()
    }
    #[allow(unused_variables)]
    fn visit_bool_expr(&mut self, bool_expr: &'ast BoolExpr) -> Self::Result {
        Self::default_result()
    }
    #[allow(unused_variables)]
    fn visit_ident_expr(&mut self, ident_expr: &'ast IdentExpr) -> Self::Result {
        Self::default_result()
    }

    /* Walkable nodes */
    fn visit_def_stmt(&mut self, def_stmt: &'ast DefineStmt<'ast>) -> Self::Result {
        walk_def_stmt(self, def_stmt)
    }

    fn visit_const_expr(&mut self, const_expr: &'ast ConstExpr) -> Self::Result {
        walk_const_expr(self, const_expr)
    }

    fn visit_pat(&mut self, pat: &'ast Pat) -> Self::Result {
        walk_pat(self, pat)
    }

    fn visit_expr(&mut self, expr: &'ast Expr<'ast>) -> Self::Result {
        walk_expr(self, expr)
    }

    fn visit_expr_with_block(&mut self, expr: &'ast ExprWithBlock<'ast>) -> Self::Result {
        walk_expr_with_block(self, expr)
    }

    fn visit_expr_without_block(&mut self, expr: &'ast ExprWithoutBlock<'ast>) -> Self::Result {
        walk_expr_without_block(self, expr)
    }

    fn visit_block_expr(&mut self, expr: &'ast BlockExpr<'ast>) -> Self::Result {
        self.visit_stmts(expr.stmts)
    }

    fn visit_if_expr(&mut self, expr: &'ast IfExpr<'ast>) -> Self::Result {
        walk_if_expr(self, expr)
    }

    fn visit_if_false_branch_expr(&mut self, expr: &'ast IfFalseBranchExpr<'ast>) -> Self::Result {
        walk_if_false_branch_expr(self, expr)
    }

    fn visit_stmts(&mut self, stmts: &'ast [&'ast Stmt<'ast>]) -> Self::Result {
        walk_stmts(self, stmts)
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt<'ast>) -> Self::Result {
        walk_stmt(self, stmt)
    }

    fn visit_place_expr(&mut self, place_expr: &'ast PlaceExpr) -> Self::Result {
        walk_place_expr(self, place_expr)
    }

    fn visit_value_expr(&mut self, value_expr: &'ast ValueExpr) -> Self::Result {
        walk_value_expr(self, value_expr)
    }

    fn visit_binary_expr(&mut self, binary_expr: &'ast BinaryExpr<'ast>) -> Self::Result {
        walk_binary_expr(self, binary_expr)
    }

    fn visit_group_expr(&mut self, group_expr: &'ast GroupExpr<'ast>) -> Self::Result {
        walk_group_expr(self, group_expr)
    }

    fn visit_assign_stmt(&mut self, assign_stmt: &'ast AssignStmt<'ast>) -> Self::Result {
        walk_assign_stmt(self, assign_stmt)
    }

    fn visit_item(&mut self, item: &'ast ItemStmt<'ast>) -> Self::Result {
        todo!()
    }
}

pub fn walk_def_stmt<'a, V>(visitor: &mut V, def_stmt: &'a DefineStmt<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_pat(&def_stmt.setter_expr);
    visitor.visit_expr(&def_stmt.value_expr)
}

pub fn walk_assign_stmt<'a, V>(visitor: &mut V, assign_stmt: &'a AssignStmt<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_place_expr(&assign_stmt.setter_expr);
    visitor.visit_expr(&assign_stmt.value_expr)
}

pub fn walk_stmt<'a, V>(visitor: &mut V, stmt: &'a Stmt<'a>) -> V::Result where V: Visitor<'a> {
    match stmt {
        Stmt::AssignStmt(stmt) => visitor.visit_assign_stmt(stmt),
        Stmt::DefineStmt(stmt) => visitor.visit_def_stmt(stmt),
        Stmt::ItemStmt(item) => visitor.visit_item(item),
        Stmt::ExprStmt(expr) => visitor.visit_expr(expr),
    }
}

pub fn walk_expr<'a, V>(visitor: &mut V, expr: &'a Expr<'a>) -> V::Result where V: Visitor<'a> {
    match &expr.kind {
        ExprKind::ExprWithBlock(expr) => visitor.visit_expr_with_block(expr),
        ExprKind::ExprWithoutBlock(expr) => visitor.visit_expr_without_block(expr),
    }
}

pub fn walk_expr_with_block<'a, V>(
    visitor: &mut V,
    expr_with_block: &'a ExprWithBlock<'a>
) -> V::Result
    where V: Visitor<'a>
{
    match expr_with_block {
        ExprWithBlock::BlockExpr(expr) => visitor.visit_block_expr(expr),
        ExprWithBlock::IfExpr(expr) => visitor.visit_if_expr(expr),
    }
}

pub fn walk_expr_without_block<'a, V>(
    visitor: &mut V,
    expr_without_block: &'a ExprWithoutBlock<'a>
) -> V::Result
    where V: Visitor<'a>
{
    match expr_without_block {
        ExprWithoutBlock::PlaceExpr(expr) => visitor.visit_place_expr(expr),
        ExprWithoutBlock::ValueExpr(expr) => visitor.visit_value_expr(expr),
    }
}

pub fn walk_if_expr<'a, V>(visitor: &mut V, if_expr: &'a IfExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_expr(if_expr.condition);
    if let Some(if_false_branch_expr) = &if_expr.false_block {
        visitor.visit_block_expr(if_expr.true_block);
        visitor.visit_if_false_branch_expr(if_false_branch_expr)
    } else {
        visitor.visit_block_expr(if_expr.true_block)
    }
}

pub fn walk_place_expr<'a, V>(visitor: &mut V, place_expr: &'a PlaceExpr) -> V::Result
    where V: Visitor<'a>
{
    match &place_expr.kind {
        PlaceKind::IdentExpr(ident_expr) => visitor.visit_ident_expr(ident_expr),
    }
}

pub fn walk_value_expr<'a, V>(visitor: &mut V, value_expr: &'a ValueExpr) -> V::Result
    where V: Visitor<'a>
{
    match value_expr {
        ValueExpr::BinaryExpr(binary_expr) => visitor.visit_binary_expr(binary_expr),
        ValueExpr::GroupExpr(group_expr) => visitor.visit_group_expr(group_expr),
        ValueExpr::ConstExpr(const_expr) => visitor.visit_const_expr(const_expr),
    }
}

pub fn walk_binary_expr<'a, V>(visitor: &mut V, binary_expr: &'a BinaryExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_expr(binary_expr.lhs);
    visitor.visit_expr(binary_expr.rhs)
}

pub fn walk_group_expr<'a, V>(visitor: &mut V, group_expr: &'a GroupExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_expr(group_expr.expr)
}

pub fn walk_if_false_branch_expr<'a, V>(
    visitor: &mut V,
    if_false_branch_expr: &'a IfFalseBranchExpr<'a>
) -> V::Result
    where V: Visitor<'a>
{
    match if_false_branch_expr {
        IfFalseBranchExpr::ElifExpr(expr) => visitor.visit_if_expr(expr),
        IfFalseBranchExpr::ElseExpr(block_expr) => visitor.visit_block_expr(block_expr),
    }
}

pub fn walk_stmts<'a, V>(visitor: &mut V, stmts: &'a [&'a Stmt<'a>]) -> V::Result
    where V: Visitor<'a>
{
    let mut result: Option<V::Result> = None;

    for stmt in stmts.iter() {
        result = Some(visitor.visit_stmt(stmt));
    }

    if let Some(result) = result {
        result
    } else {
        V::default_result()
    }
}

pub fn walk_const_expr<'a, V>(visitor: &mut V, const_expr: &'a ConstExpr) -> V::Result
    where V: Visitor<'a>
{
    match const_expr {
        ConstExpr::IntegerExpr(expr) => visitor.visit_interger_expr(expr),
        ConstExpr::BoolExpr(expr) => visitor.visit_bool_expr(expr),
    }
}

pub fn walk_pat<'a, V>(visitor: &mut V, pat: &'a Pat) -> V::Result where V: Visitor<'a> {
    match &pat.kind {
        PatKind::IdentPat(pat) => visitor.visit_ident_pat(pat),
    }
}
