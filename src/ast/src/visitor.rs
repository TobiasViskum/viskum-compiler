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
    BreakExpr,
    ConstExpr,
    ContinueExpr,
    DefineStmt,
    Expr,
    ExprWithBlock,
    ExprWithoutBlock,
    FieldExpr,
    FnItem,
    GroupExpr,
    IdentNode,
    IfExpr,
    IfFalseBranchExpr,
    IntegerExpr,
    ItemStmt,
    LoopExpr,
    Pat,
    PlaceExpr,
    Stmt,
    StructExpr,
    StructItem,
    TupleExpr,
    TupleFieldExpr,
    TypedefItem,
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
    fn visit_ident_pat(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
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
    fn visit_ident_expr(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_continue_expr(&mut self, continue_expr: &'ast ContinueExpr) -> Self::Result {
        Self::default_result()
    }

    /* Traversel of enums (not nodes) */
    fn visit_const_expr(&mut self, const_expr: ConstExpr<'ast>) -> Self::Result {
        walk_const_expr(self, const_expr)
    }

    fn visit_pat(&mut self, pat: Pat<'ast>) -> Self::Result {
        walk_pat(self, pat)
    }

    fn visit_expr(&mut self, expr: Expr<'ast>) -> Self::Result {
        walk_expr(self, expr)
    }

    fn visit_expr_with_block(&mut self, expr: ExprWithBlock<'ast>) -> Self::Result {
        walk_expr_with_block(self, expr)
    }

    fn visit_expr_without_block(&mut self, expr: ExprWithoutBlock<'ast>) -> Self::Result {
        walk_expr_without_block(self, expr)
    }

    fn visit_place_expr(&mut self, place_expr: PlaceExpr<'ast>) -> Self::Result {
        walk_place_expr(self, place_expr)
    }

    fn visit_value_expr(&mut self, value_expr: ValueExpr<'ast>) -> Self::Result {
        walk_value_expr(self, value_expr)
    }

    fn visit_stmt(&mut self, stmt: Stmt<'ast>) -> Self::Result {
        walk_stmt(self, stmt)
    }

    fn visit_stmts(&mut self, stmts: &'ast [Stmt<'ast>]) -> Self::Result {
        walk_stmts(self, stmts)
    }

    fn visit_item(&mut self, item: ItemStmt<'ast>) -> Self::Result {
        match item {
            ItemStmt::FnItem(fn_item) => self.visit_fn_item(fn_item),
            ItemStmt::StructItem(struct_item) => self.visit_struct_item(struct_item),
            ItemStmt::TypedefItem(typedef_item) => self.visit_typedef_item(typedef_item),
        }
    }

    fn visit_typedef_item(&mut self, typedef_item: &'ast TypedefItem<'ast>) -> Self::Result {
        Self::default_result()
    }

    fn visit_fn_item(&mut self, fn_item: &'ast FnItem<'ast>) -> Self::Result {
        self.visit_block_expr(fn_item.body)
    }

    fn visit_struct_item(&mut self, struct_item: &'ast StructItem<'ast>) -> Self::Result {
        Self::default_result()
    }

    fn visit_if_false_branch_expr(&mut self, expr: IfFalseBranchExpr<'ast>) -> Self::Result {
        walk_if_false_branch_expr(self, expr)
    }

    /* Walkable nodes */
    fn visit_def_stmt(&mut self, def_stmt: &'ast DefineStmt<'ast>) -> Self::Result {
        walk_def_stmt(self, def_stmt)
    }

    fn visit_tuple_field_expr(
        &mut self,
        tuple_field_expr: &'ast TupleFieldExpr<'ast>
    ) -> Self::Result {
        walk_tuple_field_expr(self, tuple_field_expr)
    }

    fn visit_field_expr(&mut self, field_expr: &'ast FieldExpr<'ast>) -> Self::Result {
        walk_field_expr(self, field_expr)
    }

    fn visit_struct_expr(&mut self, struct_expr: &'ast StructExpr<'ast>) -> Self::Result {
        walk_struct_expr(self, struct_expr)
    }

    fn visit_block_expr(&mut self, expr: &'ast BlockExpr<'ast>) -> Self::Result {
        self.visit_stmts(expr.stmts)
    }

    fn visit_loop_expr(&mut self, loop_expr: &'ast LoopExpr<'ast>) -> Self::Result {
        walk_loop_expr(self, loop_expr)
    }

    fn visit_break_expr(&mut self, break_expr: &'ast BreakExpr<'ast>) -> Self::Result {
        walk_break_expr(self, break_expr)
    }

    fn visit_if_expr(&mut self, expr: &'ast IfExpr<'ast>) -> Self::Result {
        walk_if_expr(self, expr)
    }

    fn visit_tuple_expr(&mut self, tuple_expr: &'ast TupleExpr<'ast>) -> Self::Result {
        walk_tuple_expr(self, tuple_expr)
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
}

pub fn walk_def_stmt<'a, V>(visitor: &mut V, def_stmt: &'a DefineStmt<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_pat(def_stmt.setter_expr);
    visitor.visit_expr(def_stmt.value_expr)
}

pub fn walk_assign_stmt<'a, V>(visitor: &mut V, assign_stmt: &'a AssignStmt<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_place_expr(assign_stmt.setter_expr);
    visitor.visit_expr(assign_stmt.value_expr)
}

pub fn walk_stmt<'a, V>(visitor: &mut V, stmt: Stmt<'a>) -> V::Result where V: Visitor<'a> {
    match stmt {
        Stmt::AssignStmt(stmt) => visitor.visit_assign_stmt(stmt),
        Stmt::DefineStmt(stmt) => visitor.visit_def_stmt(stmt),
        Stmt::ItemStmt(item) => visitor.visit_item(item),
        Stmt::ExprStmt(expr) => visitor.visit_expr(expr),
    }
}

pub fn walk_expr<'a, V>(visitor: &mut V, expr: Expr<'a>) -> V::Result where V: Visitor<'a> {
    match expr {
        Expr::ExprWithBlock(expr) => visitor.visit_expr_with_block(expr),
        Expr::ExprWithoutBlock(expr) => visitor.visit_expr_without_block(expr),
    }
}

pub fn walk_expr_with_block<'a, V>(visitor: &mut V, expr_with_block: ExprWithBlock<'a>) -> V::Result
    where V: Visitor<'a>
{
    match expr_with_block {
        ExprWithBlock::BlockExpr(expr) => visitor.visit_block_expr(expr),
        ExprWithBlock::IfExpr(expr) => visitor.visit_if_expr(expr),
        ExprWithBlock::LoopExpr(loop_expr) => visitor.visit_loop_expr(loop_expr),
    }
}

pub fn walk_expr_without_block<'a, V>(
    visitor: &mut V,
    expr_without_block: ExprWithoutBlock<'a>
) -> V::Result
    where V: Visitor<'a>
{
    match expr_without_block {
        ExprWithoutBlock::PlaceExpr(expr) => visitor.visit_place_expr(expr),
        ExprWithoutBlock::ValueExpr(expr) => visitor.visit_value_expr(expr),
        ExprWithoutBlock::BreakExpr(break_expr) => visitor.visit_break_expr(break_expr),
        ExprWithoutBlock::ContinueExpr(continue_expr) => visitor.visit_continue_expr(continue_expr),
        ExprWithoutBlock::ReturnExpr(return_expr) => todo!(),
    }
}

pub fn walk_loop_expr<'a, V>(visitor: &mut V, loop_expr: &'a LoopExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_block_expr(loop_expr.body)
}

pub fn walk_break_expr<'a, V>(visitor: &mut V, break_expr: &'a BreakExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    break_expr.value.map(|expr| visitor.visit_expr(expr)).unwrap_or(V::default_result())
}

pub fn walk_if_expr<'a, V>(visitor: &mut V, if_expr: &'a IfExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_expr(if_expr.condition);
    if let Some(if_false_branch_expr) = &if_expr.false_block {
        visitor.visit_block_expr(if_expr.true_block);
        visitor.visit_if_false_branch_expr(*if_false_branch_expr)
    } else {
        visitor.visit_block_expr(if_expr.true_block)
    }
}

pub fn walk_place_expr<'a, V>(visitor: &mut V, place_expr: PlaceExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    match &place_expr {
        PlaceExpr::IdentExpr(ident_expr) => visitor.visit_ident_expr(ident_expr),
        PlaceExpr::TupleFieldExpr(tuple_field_expr) =>
            visitor.visit_tuple_field_expr(tuple_field_expr),
        PlaceExpr::FieldExpr(field_expr) => visitor.visit_field_expr(field_expr),
    }
}

pub fn walk_tuple_field_expr<'a, V>(
    visitor: &mut V,
    tuple_field_expr: &'a TupleFieldExpr<'a>
) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_expr(tuple_field_expr.lhs);
    visitor.visit_interger_expr(tuple_field_expr.rhs)
}

pub fn walk_field_expr<'a, V>(visitor: &mut V, field_expr: &'a FieldExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_expr(field_expr.lhs);
    visitor.visit_ident_expr(field_expr.rhs)
}

pub fn walk_value_expr<'a, V>(visitor: &mut V, value_expr: ValueExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    match value_expr {
        ValueExpr::TupleExpr(tuple_expr) => visitor.visit_tuple_expr(tuple_expr),
        ValueExpr::BinaryExpr(binary_expr) => visitor.visit_binary_expr(binary_expr),
        ValueExpr::GroupExpr(group_expr) => visitor.visit_group_expr(group_expr),
        ValueExpr::ConstExpr(const_expr) => visitor.visit_const_expr(const_expr),
        ValueExpr::StructExpr(struct_expr) => visitor.visit_struct_expr(struct_expr),
    }
}

pub fn walk_struct_expr<'a, V>(visitor: &mut V, struct_expr: &'a StructExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_ident_expr(struct_expr.ident_node);

    struct_expr.field_initializations.iter().for_each(|field| {
        visitor.visit_ident_expr(field.ident);
        visitor.visit_expr(field.value);
    });

    V::default_result()
}

pub fn walk_tuple_expr<'a, V>(visitor: &mut V, tuple_expr: &'a TupleExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    tuple_expr.fields.iter().for_each(|expr| {
        visitor.visit_expr(*expr);
    });

    V::default_result()
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
    if_false_branch_expr: IfFalseBranchExpr<'a>
) -> V::Result
    where V: Visitor<'a>
{
    match if_false_branch_expr {
        IfFalseBranchExpr::ElifExpr(expr) => visitor.visit_if_expr(expr),
        IfFalseBranchExpr::ElseExpr(block_expr) => visitor.visit_block_expr(block_expr),
    }
}

pub fn walk_stmts<'a, V>(visitor: &mut V, stmts: &'a [Stmt<'a>]) -> V::Result where V: Visitor<'a> {
    let mut result: Option<V::Result> = None;

    for stmt in stmts.iter() {
        result = Some(visitor.visit_stmt(*stmt));
    }

    if let Some(result) = result {
        result
    } else {
        V::default_result()
    }
}

pub fn walk_const_expr<'a, V>(visitor: &mut V, const_expr: ConstExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    match const_expr {
        ConstExpr::IntegerExpr(expr) => visitor.visit_interger_expr(expr),
        ConstExpr::BoolExpr(expr) => visitor.visit_bool_expr(expr),
    }
}

pub fn walk_pat<'a, V>(visitor: &mut V, pat: Pat<'a>) -> V::Result where V: Visitor<'a> {
    match &pat {
        Pat::IdentPat(pat) => visitor.visit_ident_pat(pat),
    }
}
