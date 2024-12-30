/*

This specific visitor pattern, specifically implemented in the Visitor trait,
is inspired by how the Rust compiler visits its Ast:
https://github.com/rust-lang/rust/blob/master/compiler/rustc_ast/src/visit.rs

*/

use crate::{
    AsigneeExpr,
    AssignStmt,
    Ast,
    AstState,
    BinaryExpr,
    BlockExpr,
    BoolExpr,
    BreakExpr,
    CallExpr,
    CompDeclItem,
    CompFnDeclItem,
    CondKind,
    ConstExpr,
    ContinueExpr,
    DefineStmt,
    EnumItem,
    Expr,
    ExprWithBlock,
    ExprWithoutBlock,
    FieldExpr,
    FnItem,
    GroupExpr,
    IdentNode,
    IfExpr,
    IfFalseBranchExpr,
    ImplItem,
    ImportItem,
    IndexExpr,
    IntegerExpr,
    ItemStmt,
    LoopExpr,
    NullExpr,
    Pat,
    Path,
    PathEverything,
    PathField,
    PkgIdentNode,
    PlaceExpr,
    ReturnExpr,
    Stmt,
    StringExpr,
    StructExpr,
    StructItem,
    TupleExpr,
    TupleFieldExpr,
    TupleStructPat,
    TypedefItem,
    ValueExpr,
};

pub trait VisitAst<'ast, T> where T: AstState {
    /// Only shared with the next pass
    type LocalVisitResult;

    /// Shared with the global resolver
    type GlobalVisitResult;

    fn visit<N>(self) -> (Ast<'ast, N>, Self::GlobalVisitResult, Self::LocalVisitResult)
        where T: AstState<NextState = N>, N: AstState;
}

/// A default implementation exists for all ast nodes, meaning
/// the whole ast will be visited from left to right by default
///
/// Each implementation can be overwritten for each state the Ast can be in
/// (e.g. Ast\<'ast, AstUnresolved\> or Ast\<'ast, AstResolved\>)
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
    fn visit_string_expr(&mut self, string_expr: &'ast StringExpr) -> Self::Result {
        Self::default_result()
    }
    #[allow(unused_variables)]
    fn visit_null_expr(&mut self, null_expr: &'ast NullExpr) -> Self::Result {
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

    #[allow(unused_variables)]
    fn visit_path_segment(&mut self, path_segment: &'ast IdentNode) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_path_pkg(&mut self, pkg_ident_expr: &'ast PkgIdentNode) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_path_everything(&mut self, path_everything: &'ast PathEverything) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_pkg_ident_expr(&mut self, pkg_ident_expr: &'ast PkgIdentNode) -> Self::Result {
        Self::default_result()
    }

    /* Traversel of enums (not nodes) */
    fn visit_path_field(&mut self, path_field: &'ast PathField<'ast>) -> Self::Result {
        walk_path_field(self, path_field)
    }

    fn visit_tuple_struct_pat(&mut self, tuple_pat: &'ast TupleStructPat<'ast>) -> Self::Result {
        walk_tuple_struct_pat(self, tuple_pat)
    }

    fn visit_path(&mut self, path: Path<'ast>) -> Self::Result {
        walk_path(self, path)
    }

    fn visit_const_expr(&mut self, const_expr: ConstExpr<'ast>) -> Self::Result {
        walk_const_expr(self, const_expr)
    }

    fn visit_pat(&mut self, pat: Pat<'ast>) -> Self::Result {
        walk_pat(self, pat)
    }

    fn visit_return_expr(&mut self, return_expr: &'ast ReturnExpr) -> Self::Result {
        walk_return_expr(self, return_expr)
    }

    fn visit_expr(&mut self, expr: Expr<'ast>) -> Self::Result {
        walk_expr(self, expr)
    }

    fn visit_cond_kind(&mut self, cond_kind: CondKind<'ast>) -> Self::Result {
        match cond_kind {
            CondKind::CondExpr(expr) => self.visit_expr(expr),
            CondKind::CondPat(pat, rhs_expr) => {
                self.visit_pat(pat);
                self.visit_expr(rhs_expr)
            }
        }
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

    fn visit_asignee_expr(&mut self, assignee_expr: AsigneeExpr<'ast>) -> Self::Result {
        walk_asignee_expr(self, assignee_expr)
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
            ItemStmt::ImportItem(import_item) => self.visit_import_item(import_item),
            ItemStmt::FnItem(fn_item) => self.visit_fn_item(fn_item),
            ItemStmt::StructItem(struct_item) => self.visit_struct_item(struct_item),
            ItemStmt::TypedefItem(typedef_item) => self.visit_typedef_item(typedef_item),
            ItemStmt::EnumItem(enum_item) => self.visit_enum_item(enum_item),
            ItemStmt::CompDeclItem(comp_decl_item) => self.visit_comp_decl_item(comp_decl_item),
            ItemStmt::ImplItem(impl_item) => self.visit_impl_item(impl_item),
        }
    }

    fn visit_import_item(&mut self, import_item: &'ast ImportItem<'ast>) -> Self::Result {
        walk_import_item(self, import_item)
    }

    fn visit_comp_decl_item(&mut self, comp_decl_item: CompDeclItem<'ast>) -> Self::Result {
        walk_comp_decl_item(self, comp_decl_item)
    }

    fn visit_impl_item(&mut self, impl_item: &'ast ImplItem<'ast>) -> Self::Result {
        walk_impl_item(self, impl_item)
    }

    fn visit_comp_fn_decl_item(
        &mut self,
        comp_fn_decl_item: &'ast CompFnDeclItem<'ast>
    ) -> Self::Result {
        Self::default_result()
    }

    fn visit_typedef_item(&mut self, typedef_item: &'ast TypedefItem<'ast>) -> Self::Result {
        Self::default_result()
    }

    fn visit_enum_item(&mut self, enum_item: &'ast EnumItem<'ast>) -> Self::Result {
        Self::default_result()
    }

    fn visit_fn_item(&mut self, fn_item: &'ast FnItem<'ast>) -> Self::Result {
        self.visit_stmts(fn_item.body)
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

    fn visit_index_expr(&mut self, index_expr: &'ast IndexExpr<'ast>) -> Self::Result {
        walk_index_expr(self, index_expr)
    }

    fn visit_struct_expr(&mut self, struct_expr: &'ast StructExpr<'ast>) -> Self::Result {
        walk_struct_expr(self, struct_expr)
    }

    fn visit_call_expr(&mut self, call_expr: &'ast CallExpr<'ast>) -> Self::Result {
        walk_call_expr(self, call_expr)
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
    visitor.visit_asignee_expr(assign_stmt.setter_expr);
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
        ExprWithoutBlock::ReturnExpr(return_expr) => visitor.visit_return_expr(return_expr),
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

pub fn walk_return_expr<'a, V>(visitor: &mut V, return_expr: &'a ReturnExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    return_expr.value.map(|expr| visitor.visit_expr(expr)).unwrap_or(V::default_result())
}

pub fn walk_if_expr<'a, V>(visitor: &mut V, if_expr: &'a IfExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_cond_kind(if_expr.cond_kind);

    if let Some(if_false_branch_expr) = &if_expr.false_block {
        visitor.visit_stmts(if_expr.true_block);
        visitor.visit_if_false_branch_expr(*if_false_branch_expr)
    } else {
        visitor.visit_stmts(if_expr.true_block)
    }
}

pub fn walk_asignee_expr<'a, V>(visitor: &mut V, asignee_expr: AsigneeExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    match asignee_expr {
        AsigneeExpr::PlaceExpr(place_expr) => visitor.visit_place_expr(place_expr),
        AsigneeExpr::CallExpr(call_expr) => visitor.visit_call_expr(call_expr),
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
        PlaceExpr::IndexExpr(index_expr) => visitor.visit_index_expr(index_expr),
        PlaceExpr::PkgIdentExpr(pkg_ident_expr) => visitor.visit_pkg_ident_expr(pkg_ident_expr),
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

pub fn walk_index_expr<'a, V>(visitor: &mut V, index_expr: &'a IndexExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_expr(index_expr.lhs);
    visitor.visit_expr(index_expr.value_expr)
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
        ValueExpr::CallExpr(call_expr) => visitor.visit_call_expr(call_expr),
    }
}

pub fn walk_impl_item<'a, V>(visitor: &mut V, impl_item: &'a ImplItem<'a>) -> V::Result
    where V: Visitor<'a>
{
    impl_item.impl_fns.iter().for_each(|item| {
        visitor.visit_fn_item(item);
    });

    V::default_result()
}

pub fn walk_import_item<'a, V>(visitor: &mut V, import_item: &'a ImportItem<'a>) -> V::Result
    where V: Visitor<'a>
{
    for import_item in import_item.import_items_path.iter() {
        visitor.visit_path(*import_item);
    }

    V::default_result()
}

pub fn walk_comp_decl_item<'a, V>(visitor: &mut V, comp_decl_item: CompDeclItem<'a>) -> V::Result
    where V: Visitor<'a>
{
    match comp_decl_item {
        CompDeclItem::CompFnDeclItem(comp_fn_decl_item) =>
            visitor.visit_comp_fn_decl_item(comp_fn_decl_item),
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

pub fn walk_call_expr<'a, V>(visitor: &mut V, call_expr: &'a CallExpr<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_expr(call_expr.callee);

    call_expr.args.iter().for_each(|arg| {
        visitor.visit_expr(*arg);
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
        IfFalseBranchExpr::ElifExpr(if_expr) => visitor.visit_if_expr(if_expr),
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

pub fn walk_stmts_none_items_but_fns<'a, V>(visitor: &mut V, stmts: &'a [Stmt<'a>]) -> V::Result
    where V: Visitor<'a>
{
    let mut result: Option<V::Result> = None;

    for stmt in stmts.iter() {
        match stmt {
            Stmt::ItemStmt(item) => {
                match item {
                    ItemStmt::FnItem(fn_item) => {
                        result = Some(visitor.visit_fn_item(fn_item));
                    }
                    ItemStmt::ImplItem(impl_item) => {
                        visitor.visit_impl_item(impl_item);
                    }
                    _ => {}
                }
            }

            stmt => {
                result = Some(visitor.visit_stmt(*stmt));
            }
        }
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
        ConstExpr::NullExpr(expr) => visitor.visit_null_expr(expr),
        ConstExpr::StringExpr(string_expr) => visitor.visit_string_expr(string_expr),
    }
}

pub fn walk_pat<'a, V>(visitor: &mut V, pat: Pat<'a>) -> V::Result where V: Visitor<'a> {
    match &pat {
        Pat::IdentPat(pat) => visitor.visit_ident_pat(pat),
        Pat::TupleStructPat(tuple_pat) => visitor.visit_tuple_struct_pat(tuple_pat),
    }
}

pub fn walk_tuple_struct_pat<'a, V>(visitor: &mut V, tuple_pat: &'a TupleStructPat<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_path(tuple_pat.path);

    tuple_pat.fields.iter().for_each(|pat| {
        visitor.visit_pat(*pat);
    });

    V::default_result()
}

pub fn walk_path<'a, V>(visitor: &mut V, path: Path<'a>) -> V::Result where V: Visitor<'a> {
    match path {
        Path::PathField(path_field) => visitor.visit_path_field(path_field),
        Path::PathSegment(path_segment) => visitor.visit_path_segment(path_segment),
        Path::PathPkg(path_pkg) => visitor.visit_path_pkg(path_pkg),
    }
}

pub fn walk_path_field<'a, V>(visitor: &mut V, path_field: &'a PathField<'a>) -> V::Result
    where V: Visitor<'a>
{
    visitor.visit_path(path_field.lhs);
    visitor.visit_path_segment(path_field.rhs)
}
