use crate::{
    ast_state::{ AstState, AstState1, AstState2, AstState3 },
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
use ty::{ NodeId, PrimTy, Ty };

/// Visits the entire Ast from left to right
///
/// When the visitor is done, the ast will transition into the next ast-state
#[derive(Debug)]
pub struct AstVisitor<'ast, 'b, T, E> where T: AstState, E: AstVisitEmitter<'ast, T> {
    pub ast: Ast<'ast, T>,
    pub ast_visit_emitter: &'b mut E,
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
    fn set_type_to_node(&mut self, node_id: NodeId, ty: Ty) -> &'ast Ty
        where T: AstState<ThisState = AstState2>
    {
        self.ast_visit_emitter.set_type_to_node(node_id, ty)
    }
    fn get_ty_from_node_id(&self, node_id: NodeId) -> Option<&'ast Ty>
        where T: AstState<ThisState = AstState2>
    {
        self.ast_visit_emitter.get_ty_from_node_id(node_id)
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
    fn set_type_to_node(&mut self, node_id: NodeId, ty: Ty) -> &'a Ty
        where T: AstState<ThisState = AstState2>;

    fn get_ty_from_node_id(&self, node_id: NodeId) -> Option<&'a Ty>
        where T: AstState<ThisState = AstState2>;
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
        walk_stmts(self, expr.stmts);
        self.end_scope();
    }
}

impl<'ast, 'b, E> Visitor<'ast>
    for AstVisitor<'ast, 'b, AstState2, E>
    where E: AstVisitEmitter<'ast, AstState2>
{
    type Result = &'ast Ty;

    fn default_result() -> Self::Result {
        &Ty::PrimTy(PrimTy::Void)
    }

    fn visit_interger_expr(&mut self, interger_expr: &'ast IntegerExpr) -> Self::Result {
        self.set_type_to_node(interger_expr.ast_node_id, Ty::PrimTy(PrimTy::Int))
    }

    fn visit_ident_expr(&mut self, ident_expr: &'ast IdentExpr) -> Self::Result {
        self.get_ty_from_node_id(ident_expr.ast_node_id).unwrap_or(&Ty::Unkown)
    }
}

/// A default implementation exists for all ast nodes, meaning
/// the whole ast will be visited if no implementations are overwritten
///
/// A different implementation can be implemented for each state the Ast is in
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

macro_rules! _walk {
    (
        $method_name:ident < $lifetime_a:lifetime,
        $visitor_generic:ident > (
            $visitor:ident: &mut $visitor_type:ident,
            $walk_name:ident: $walk_type:ty
        ) $code:block
    ) => {
        pub fn $method_name<$lifetime_a, 'b, T, E, $visitor_generic>($visitor: &mut $visitor_type, $walk_name: $walk_type) -> $visitor_generic::Result
            where T: AstState, E: AstVisitEmitter<$lifetime_a, T>, $visitor_generic: Visitor<$lifetime_a, 'b, T, E>
        {
            $code
        }
    };
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
