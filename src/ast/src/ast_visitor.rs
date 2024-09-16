/*

This specific visitor pattern, specifically implemented in the Visitor trait,
is inspired by how the Rust compiler visits its Ast:
https://github.com/rust-lang/rust/blob/master/compiler/rustc_ast/src/visit.rs

*/

use crate::{
    ast_query_system::AstQueryEntry,
    ast_state::{ AstState, AstState0, AstState1, AstState2, AstState3 },
    AssignStmt,
    Ast,
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
    IfFalseBranchExpr,
    IntegerExpr,
    ItemStmt,
    Pat,
    PlaceExpr,
    Stmt,
    ValueExpr,
};
use ir_defs::{ DefId, DefKind, NameBinding, NodeId };
use ty::{ PrimTy, Ty };

/// Visits the entire Ast from left to right
///
/// When the visitor is done, the ast will transition into the next state
#[derive(Debug)]
pub struct AstVisitor<'ast, 'b, T, E> where T: AstState, E: AstVisitEmitter<'ast, T> {
    pub ast: Ast<'ast, T>,
    pub ast_visit_emitter: &'b mut E,
}

impl<'ast, 'b, E> AstVisitor<'ast, 'b, AstState0, E> where E: AstVisitEmitter<'ast, AstState0> {
    pub fn visit(mut self) -> Ast<'ast, AstState1> {
        self.visit_stmts(self.ast.main_scope.stmts);
        self.ast.query_system.assert_nodes_amount();
        self.ast.next_state()
    }

    pub fn insert_query_entry(&mut self, node_id: NodeId, ast_query_entry: AstQueryEntry<'ast>) {
        self.ast.query_system.insert_entry(node_id, ast_query_entry)
    }
}

impl<'ast, 'b, E> AstVisitor<'ast, 'b, AstState1, E> where E: AstVisitEmitter<'ast, AstState1> {
    pub fn visit(mut self) -> Ast<'ast, AstState2> {
        self.visit_stmts(self.ast.main_scope.stmts);
        self.ast.next_state()
    }
}

impl<'ast, 'b, E> AstVisitor<'ast, 'b, AstState2, E> where E: AstVisitEmitter<'ast, AstState2> {
    pub fn visit(mut self) -> Ast<'ast, AstState3> {
        self.visit_stmts(self.ast.main_scope.stmts);
        self.ast.next_state()
    }
}

impl<'ast, 'b, T, E> AstVisitor<'ast, 'b, T, E> where T: AstState, E: AstVisitEmitter<'ast, T> {
    pub fn new(ast: Ast<'ast, T>, ast_visit_emitter: &'b mut E) -> Self {
        Self { ast, ast_visit_emitter }
    }
}

impl<'ast, 'b, T, E> AstVisitEmitter<'ast, T>
    for AstVisitor<'ast, 'b, T, E>
    where T: AstState, E: AstVisitEmitter<'ast, T>
{
    /* Used during the first pass (name resolution) */
    fn start_scope(&mut self) where T: AstState<ThisState = AstState1> {
        self.ast_visit_emitter.start_scope()
    }
    fn end_scope(&mut self) where T: AstState<ThisState = AstState1> {
        self.ast_visit_emitter.end_scope()
    }
    fn define_var(&mut self, ident_pat: &'ast IdentPat) where T: AstState<ThisState = AstState1> {
        self.ast_visit_emitter.define_var(ident_pat)
    }
    fn lookup_var(&mut self, ident_expr: &'ast IdentExpr) where T: AstState<ThisState = AstState1> {
        self.ast_visit_emitter.lookup_var(ident_expr)
    }

    /* Used during the second pass (type checking) */
    fn set_type_to_node_id(&mut self, node_id: NodeId, ty: &'ast Ty)
        where T: AstState<ThisState = AstState2>
    {
        self.ast_visit_emitter.set_type_to_node_id(node_id, ty)
    }
    fn intern_type(&mut self, ty: Ty) -> &'ast Ty where T: AstState<ThisState = AstState2> {
        self.ast_visit_emitter.intern_type(ty)
    }
    fn get_def_id_from_node_id(&self, node_id: NodeId) -> DefId
        where T: AstState<ThisState = AstState2>
    {
        self.ast_visit_emitter.get_def_id_from_node_id(node_id)
    }
    fn get_namebinding_and_ty_from_def_id(&self, def_id: DefId) -> (NameBinding, &'ast Ty)
        where T: AstState<ThisState = AstState2>
    {
        self.ast_visit_emitter.get_namebinding_and_ty_from_def_id(def_id)
    }
    fn set_namebinding_and_ty_to_def_id(
        &mut self,
        def_id: DefId,
        name_binding: NameBinding,
        ty: &'ast Ty
    )
        where T: AstState<ThisState = AstState2>
    {
        self.ast_visit_emitter.set_namebinding_and_ty_to_def_id(def_id, name_binding, ty)
    }
}

/// This can call functions on the Resolver struct in the resolver crate,
/// which also implements this trait
///
/// The reason it's not using the Resolver directly is to avoid cyclic references
pub trait AstVisitEmitter<'a, T>: Sized where T: AstState {
    /* Methods for the first pass (name resolution) */
    fn start_scope(&mut self) where T: AstState<ThisState = AstState1>;
    fn end_scope(&mut self) where T: AstState<ThisState = AstState1>;
    fn define_var(&mut self, ident_pat: &'a IdentPat) where T: AstState<ThisState = AstState1>;
    fn lookup_var(&mut self, ident_expr: &'a IdentExpr) where T: AstState<ThisState = AstState1>;

    /* Methods for the second pass (type checking) */
    fn intern_type(&mut self, ty: Ty) -> &'a Ty where T: AstState<ThisState = AstState2>;
    fn set_type_to_node_id(&mut self, node_id: NodeId, ty: &'a Ty)
        where T: AstState<ThisState = AstState2>;
    fn get_def_id_from_node_id(&self, node_id: NodeId) -> DefId
        where T: AstState<ThisState = AstState2>;
    fn set_namebinding_and_ty_to_def_id(
        &mut self,
        def_id: DefId,
        name_binding: NameBinding,
        ty: &'a Ty
    )
        where T: AstState<ThisState = AstState2>;
    fn get_namebinding_and_ty_from_def_id(&self, def_id: DefId) -> (NameBinding, &'a Ty)
        where T: AstState<ThisState = AstState2>;
}

/// Implements the visitor trait for
impl<'ast, 'b, E> Visitor<'ast>
    for AstVisitor<'ast, 'b, AstState0, E>
    where E: AstVisitEmitter<'ast, AstState0>
{
    type Result = ();

    fn default_result() -> Self::Result {
        ()
    }

    fn visit_interger_expr(&mut self, interger_expr: &'ast IntegerExpr) -> Self::Result {
        self.insert_query_entry(
            interger_expr.ast_node_id,
            AstQueryEntry::IntegerExpr(interger_expr)
        )
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast DefineStmt<'ast>) -> Self::Result {
        self.insert_query_entry(def_stmt.ast_node_id, AstQueryEntry::DefineStmt(def_stmt));
        walk_def_stmt(self, def_stmt)
    }

    fn visit_assign_stmt(&mut self, assign_stmt: &'ast AssignStmt<'ast>) -> Self::Result {
        self.insert_query_entry(assign_stmt.ast_node_id, AstQueryEntry::AssignStmt(assign_stmt));
        todo!("Walk statement here")
    }

    fn visit_block_expr(&mut self, expr: &'ast BlockExpr<'ast>) -> Self::Result {
        self.insert_query_entry(expr.ast_node_id, AstQueryEntry::BlockExpr(expr));
        walk_stmts(self, expr.stmts)
    }

    fn visit_if_expr(&mut self, if_expr: &'ast IfExpr<'ast>) -> Self::Result {
        self.insert_query_entry(if_expr.ast_node_id, AstQueryEntry::IfExpr(if_expr));
        walk_if_expr(self, if_expr)
    }

    fn visit_ident_expr(&mut self, ident_expr: &'ast IdentExpr) -> Self::Result {
        self.insert_query_entry(ident_expr.ast_node_id, AstQueryEntry::IdentExpr(ident_expr))
    }

    fn visit_ident_pat(&mut self, ident_pat: &'ast IdentPat) -> Self::Result {
        self.insert_query_entry(ident_pat.ast_node_id, AstQueryEntry::IdentPat(ident_pat))
    }

    fn visit_binary_expr(&mut self, binary_expr: &'ast BinaryExpr<'ast>) -> Self::Result {
        self.insert_query_entry(binary_expr.ast_node_id, AstQueryEntry::BinarExpr(binary_expr));
        walk_binary_expr(self, binary_expr)
    }

    fn visit_group_expr(&mut self, group_expr: &'ast GroupExpr<'ast>) -> Self::Result {
        self.insert_query_entry(group_expr.ast_node_id, AstQueryEntry::GroupExpr(group_expr));
        walk_group_expr(self, group_expr)
    }
}

/// Implements the Visitor trait for the first pass (name resolution)
impl<'ast, 'b, E> Visitor<'ast>
    for AstVisitor<'ast, 'b, AstState1, E>
    where E: AstVisitEmitter<'ast, AstState1>
{
    type Result = ();

    fn default_result() -> Self::Result {
        ()
    }

    fn visit_ident_pat(&mut self, ident_pat: &'ast IdentPat) -> Self::Result {
        self.define_var(ident_pat)
    }

    fn visit_ident_expr(&mut self, ident_expr: &'ast IdentExpr) -> Self::Result {
        self.lookup_var(ident_expr)
    }

    fn visit_block_expr(&mut self, expr: &'ast BlockExpr<'ast>) -> Self::Result {
        self.start_scope();
        self.visit_stmts(expr.stmts);
        self.end_scope();
    }
}

/// Implements the Visitor trait for the second pass (type checking)
impl<'ast, 'b, E> Visitor<'ast>
    for AstVisitor<'ast, 'b, AstState2, E>
    where E: AstVisitEmitter<'ast, AstState2>
{
    type Result = &'ast Ty;

    fn default_result() -> Self::Result {
        &Ty::PrimTy(PrimTy::Void)
    }

    fn visit_interger_expr(&mut self, interger_expr: &'ast IntegerExpr) -> Self::Result {
        let interned_type = self.intern_type(Ty::PrimTy(PrimTy::Int));
        self.set_type_to_node_id(interger_expr.ast_node_id, interned_type);
        interned_type
    }

    fn visit_ident_expr(&mut self, ident_expr: &'ast IdentExpr) -> Self::Result {
        let def_id = self.ast_visit_emitter.get_def_id_from_node_id(ident_expr.ast_node_id);
        let (_, def_type) = self.ast_visit_emitter.get_namebinding_and_ty_from_def_id(def_id);
        def_type
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast DefineStmt<'ast>) -> Self::Result {
        // When tuples are implemented, compare if tuple on lhs, is same as tuple type on rhs
        let def_type = walk_expr(self, &def_stmt.value_expr);

        match &def_stmt.setter_expr {
            Pat::IdentPat(ident_pat) => {
                let def_id = self.ast_visit_emitter.get_def_id_from_node_id(ident_pat.ast_node_id);
                self.ast_visit_emitter.set_namebinding_and_ty_to_def_id(
                    def_id,
                    NameBinding::from(DefKind::Variable),
                    def_type
                );
            }
        }

        // Returns void, since definitions cannot return a value
        self.intern_type(Ty::PrimTy(PrimTy::Void))
    }

    fn visit_if_expr(&mut self, expr: &'ast IfExpr<'ast>) -> Self::Result {
        let true_type = walk_stmts(self, expr.true_block.stmts);
        let false_type = expr.false_block
            .as_ref()
            .map(|expr| walk_if_false_branch_expr(self, expr));

        if let Some(false_type) = false_type {
            if true_type == false_type { true_type } else { self.intern_type(Ty::Unkown) }
        } else {
            true_type
        }
    }

    fn visit_binary_expr(&mut self, binary_expr: &'ast BinaryExpr<'ast>) -> Self::Result {
        let lhs_type = walk_expr(self, binary_expr.lhs);
        let rhs_type = walk_expr(self, binary_expr.rhs);

        let result_type = match (lhs_type, rhs_type) {
            (Ty::PrimTy(PrimTy::Int), Ty::PrimTy(PrimTy::Int)) => { &Ty::PrimTy(PrimTy::Int) }
            _ => {
                println!(
                    "Report error in binary expr: {} {} {}",
                    lhs_type,
                    binary_expr.op,
                    rhs_type
                );
                self.intern_type(Ty::Unkown)
            }
        };

        self.ast_visit_emitter.set_type_to_node_id(binary_expr.ast_node_id, result_type);

        result_type
    }
}

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

    fn visit_assign_stmt(&mut self, stmt: &'ast AssignStmt<'ast>) -> Self::Result {
        todo!()
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

pub fn walk_stmt<'a, V>(visitor: &mut V, stmt: &'a Stmt<'a>) -> V::Result where V: Visitor<'a> {
    match stmt {
        Stmt::AssignStmt(stmt) => visitor.visit_assign_stmt(stmt),
        Stmt::DefineStmt(stmt) => visitor.visit_def_stmt(stmt),
        Stmt::ItemStmt(item) => visitor.visit_item(item),
        Stmt::ExprStmt(expr) => visitor.visit_expr(expr),
    }
}

pub fn walk_expr<'a, V>(visitor: &mut V, expr: &'a Expr<'a>) -> V::Result where V: Visitor<'a> {
    match expr {
        Expr::ExprWithBlock(expr) => visitor.visit_expr_with_block(expr),
        Expr::ExprWithoutBlock(expr) => visitor.visit_expr_without_block(expr),
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
    match place_expr {
        PlaceExpr::IdentExpr(ident_expr) => visitor.visit_ident_expr(ident_expr),
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
        IfFalseBranchExpr::ElseExpr(expr) => visitor.visit_block_expr(expr),
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
    }
}

pub fn walk_pat<'a, V>(visitor: &mut V, pat: &'a Pat) -> V::Result where V: Visitor<'a> {
    match pat {
        Pat::IdentPat(pat) => visitor.visit_ident_pat(pat),
    }
}
