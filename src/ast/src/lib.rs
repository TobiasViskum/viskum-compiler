/*

IMPLEMENTATION DETAILS:

Each node in the Ast doesn't have ANY methods at all (except for `new` method),
since the nodes are just for holding relavant data.
Therefore all fields on each node is also public.

That means when making an AstVisitor or AstPrettifier,
those structs doesn't rely on logic from the Ast nodes,
but rather keep their logic separated.

You might see in some of the enums in this file, that
some have references to their variants and some don't. That's because, 
they only holds references to "real" nodes, which is a node that has a NodeId

A description of the overall structure of the Ast (inspired by the way Rust structure its HIR)

Stmt(
    DefineStmt,
    AssignStmt,
    ItemStmt(
        FunctionStmt
    ),
    ExprStmt(
        Expr(
            ExprWithBlock(
                BlockExpr,
                IfExpr
            ),
            ExprWithoutBlock(
                PlaceExpr(
                    IdentExpr
                ),
                ValueExpr(
                    BinaryExpr,
                    GroupExpr,
                    ConstExpr(
                        IntegerExpr
                    )
                )
            )
        )
    )
)

*/

mod passes;
mod visitor;
mod ast_arena;
mod ast_prettifier;
mod ast_query_system;
mod ast_state;
mod ast_visitor;

pub use ast_state::*;
pub use ast_arena::AstArena;
pub use ast_prettifier::AstPrettifier;
pub use ast_query_system::{ AstQueryEntry, AstQuerySystem };
pub use ast_visitor::{ AstVisitEmitter, AstVisitor };
pub use visitor::*;

use std::{ cell::Cell, marker::PhantomData };
use ir_defs::{ NodeId, ResultLoc };
use op::BinaryOp;
use span::Span;
use derive_new::new;

type Stmts<'ast> = &'ast [&'ast Stmt<'ast>];

#[derive(Debug, new)]
pub struct Ast<'ast, T> where T: AstState {
    pub main_scope: GlobalScope<'ast>,
    query_system: AstQuerySystem<'ast>,
    _state: PhantomData<T>,
}

impl<'ast, T> Ast<'ast, T> where T: AstState {
    pub(crate) fn next_state<N>(self) -> Ast<'ast, N> where T: AstState<NextState = N>, N: AstState {
        Ast {
            main_scope: self.main_scope,
            query_system: self.query_system,
            _state: PhantomData,
        }
    }
}

impl<'ast> Ast<'ast, AstState0> {
    pub fn get_visitor<'ctx, 'c, E: AstVisitEmitter<'ctx, 'ast, AstState0>>(
        self,
        src: &'c str,
        ast_visit_emitter: &'c mut E
    )
        -> AstVisitor<'ctx, 'ast, 'c, AstState0, E>
        where 'ctx: 'ast, 'ast: 'c
    {
        AstVisitor::new(self, src, ast_visit_emitter)
    }

    pub fn insert_query_entry(&mut self, node_id: NodeId, ast_query_entry: AstQueryEntry<'ast>) {
        self.query_system.insert_entry(node_id, ast_query_entry)
    }
}
impl<'ast> Ast<'ast, AstState1> {
    pub fn get_visitor<'ctx, 'c, E: AstVisitEmitter<'ctx, 'ast, AstState1>>(
        self,
        src: &'c str,
        ast_visit_emitter: &'c mut E
    )
        -> AstVisitor<'ctx, 'ast, 'c, AstState1, E>
        where 'ctx: 'ast, 'ast: 'c
    {
        AstVisitor::new(self, src, ast_visit_emitter)
    }
}

impl<'ast> Ast<'ast, AstState2> {
    pub fn get_visitor<'ctx, 'c, E: AstVisitEmitter<'ctx, 'ast, AstState2>>(
        self,
        src: &'c str,
        ast_visit_emitter: &'c mut E
    )
        -> AstVisitor<'ctx, 'ast, 'c, AstState2, E>
        where 'ctx: 'ast, 'ast: 'c
    {
        AstVisitor::new(self, src, ast_visit_emitter)
    }
}

impl<'ast> Ast<'ast, AstState3> {
    pub fn query_all(&self, mut f: impl FnMut(&NodeId, &AstQueryEntry)) {
        self.query_system.query_all(|node_id, query_entry| f(node_id, query_entry));
    }
}

#[derive(Debug, new)]
pub struct GlobalScope<'ast> {
    pub stmts: Stmts<'ast>,
}

#[derive(Debug, Clone, Copy)]
pub enum Stmt<'ast> {
    ItemStmt(ItemStmt<'ast>),
    DefineStmt(&'ast DefineStmt<'ast>),
    AssignStmt(&'ast AssignStmt<'ast>),
    ExprStmt(&'ast Expr<'ast>),
}

#[derive(Debug, Clone, Copy)]
pub enum ItemStmt<'ast> {
    FunctionStmt(&'ast FunctionStmt<'ast>),
}

#[derive(Debug, new)]
pub struct FunctionStmt<'ast> {
    pub ident_expr: IdentExpr,
    pub body: &'ast Expr<'ast>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct DefineStmt<'ast> {
    #[new(value = "None.into()")]
    pub mut_span: Cell<Option<Span>>,
    pub setter_expr: &'ast Pat<'ast>,
    pub value_expr: &'ast Expr<'ast>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct AssignStmt<'ast> {
    pub setter_expr: &'ast PlaceExpr<'ast>,
    pub value_expr: &'ast Expr<'ast>,
    pub ast_node_id: NodeId,
    pub span: Span,
}

#[derive(Debug, new)]
pub struct Pat<'ast> {
    pub kind: PatKind<'ast>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, Clone, Copy)]
pub enum PatKind<'ast> {
    IdentPat(&'ast IdentPat),
}

/// This is a unique node, because it makes access of node_id eaiser
#[derive(Debug, new)]
pub struct Expr<'ast> {
    pub kind: ExprKind<'ast>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, Clone, Copy)]
pub enum ExprKind<'ast> {
    ExprWithBlock(ExprWithBlock<'ast>),
    ExprWithoutBlock(ExprWithoutBlock<'ast>),
}

#[derive(Debug, Clone, Copy)]
pub enum ExprWithBlock<'ast> {
    BlockExpr(&'ast BlockExpr<'ast>),
    IfExpr(&'ast IfExpr<'ast>),
    LoopExpr(&'ast LoopExpr<'ast>),
}

#[derive(Debug, new)]
pub struct LoopExpr<'ast> {
    pub body: &'ast BlockExpr<'ast>,
    pub ast_node_id: NodeId,
    pub result_loc: ResultLoc,
}

#[derive(Debug, new)]
pub struct BlockExpr<'ast> {
    pub stmts: Stmts<'ast>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct IfExpr<'ast> {
    pub condition: &'ast Expr<'ast>,
    pub true_block: &'ast BlockExpr<'ast>,
    pub false_block: Option<IfFalseBranchExpr<'ast>>,
    pub ast_node_id: NodeId,
    pub span: Span,
    /// Makes sure if and elif result goes into same place
    pub result_loc: ResultLoc,
}

#[derive(Debug, Clone, Copy)]
pub enum IfFalseBranchExpr<'ast> {
    ElseExpr(&'ast BlockExpr<'ast>),
    ElifExpr(&'ast IfExpr<'ast>),
}

#[derive(Debug, Clone, Copy)]
pub enum ExprWithoutBlock<'ast> {
    PlaceExpr(&'ast PlaceExpr<'ast>),
    ValueExpr(ValueExpr<'ast>),
    BreakExpr(&'ast BreakExpr<'ast>),
    ContinueExpr(&'ast ContinueExpr),
}

#[derive(Debug, new)]
pub struct BreakExpr<'ast> {
    pub value: Option<&'ast Expr<'ast>>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct ContinueExpr {
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct PlaceExpr<'ast> {
    pub kind: PlaceKind<'ast>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, Clone, Copy)]
pub enum PlaceKind<'ast> {
    IdentExpr(&'ast IdentExpr),
}

#[derive(Debug, new)]
pub struct IdentPat {
    pub span: Span,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct IdentExpr {
    pub span: Span,
    pub ast_node_id: NodeId,
}

impl IdentExpr {
    pub fn get_as_pat(&self) -> IdentPat {
        IdentPat::new(self.span, self.ast_node_id)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ValueExpr<'ast> {
    BinaryExpr(&'ast BinaryExpr<'ast>),
    GroupExpr(&'ast GroupExpr<'ast>),
    TupleExpr(&'ast TupleExpr<'ast>),
    ConstExpr(ConstExpr<'ast>),
}

#[derive(Debug, new)]
pub struct TupleExpr<'ast> {
    pub fields: &'ast [&'ast Expr<'ast>],
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct GroupExpr<'ast> {
    pub expr: &'ast Expr<'ast>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct BinaryExpr<'ast> {
    pub lhs: &'ast Expr<'ast>,
    pub op: BinaryOp,
    pub rhs: &'ast Expr<'ast>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, Clone, Copy)]
pub enum ConstExpr<'ast> {
    IntegerExpr(&'ast IntegerExpr),
    BoolExpr(&'ast BoolExpr),
}

#[derive(Debug, new)]
pub struct BoolExpr {
    pub val: bool,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct IntegerExpr {
    pub val: i64,
    pub ast_node_id: NodeId,
}

#[derive(Debug)]
pub struct FloatExpr {
    pub val: f64,
    pub ast_node_id: NodeId,
}
