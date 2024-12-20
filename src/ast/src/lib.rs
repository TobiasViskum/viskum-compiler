/*

IMPLEMENTATION DETAILS:

Each node in the Ast doesn't have ANY methods at all (except for a `new` method),
since the nodes are just for holding relavant data.
Therefore all fields on each node is also public.

That means when making an AstVisitor or AstPrettifier,
those structs doesn't rely on logic from the Ast nodes,
but rather keep their logic separated.

You might see in some of the enums in this file, that
some have references to their variants and some don't. That's because, 
they only holds references to "real" nodes, which is a node that has a NodeId

A description of the overall structure of the Ast (inspired by the way Rust structure its HIR).
A `&` means that it's a reference to the node (allocated in an arena)

Stmt(
    &DefineStmt,
    &AssignStmt,
    ItemStmt(
        &FunctionStmt
    ),
    ExprStmt(
        Expr(
            ExprWithBlock(
                &BlockExpr,
                &IfExpr,
                &LoopExpr
            ),
            ExprWithoutBlock(
                PlaceExpr(
                    &IdentExpr,
                    &TupleFieldExpr
                ),
                ValueExpr(
                    &BinaryExpr,
                    &GroupExpr,
                    &TupleExpr,
                    ConstExpr(
                        &IntegerExpr
                    )
                )
            )
        )
    )
)

*/

mod typechecker;
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
use ir::{ Mutability, NodeId, Symbol };
use op::BinaryOp;
use span::Span;
use derive_new::new;

type Stmts<'ast> = &'ast [Stmt<'ast>];

#[derive(Debug, new)]
pub struct Ast<'ast, T> where T: AstState {
    pub main_scope: GlobalScope<'ast>,
    pub fn_count: usize,
    query_system: AstQuerySystem<'ast>,
    _state: PhantomData<T>,
}

impl<'ast, T> Ast<'ast, T> where T: AstState {
    pub fn next_state<N>(self) -> Ast<'ast, N> where T: AstState<NextState = N>, N: AstState {
        Ast {
            main_scope: self.main_scope,
            query_system: self.query_system,
            fn_count: self.fn_count,
            _state: PhantomData,
        }
    }

    pub fn expected_node_count(&self) -> usize {
        self.query_system.expected_nodes_count
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    C,
    Normal,
}

#[derive(Debug, new)]
pub struct GlobalScope<'ast> {
    pub stmts: Stmts<'ast>,
}

#[derive(Debug, Clone, Copy)]
pub enum Typing<'ast> {
    Ident(Span),
    Tuple(&'ast [Typing<'ast>]),
    Ptr(&'ast Typing<'ast>, Mutability),
    ManyPtr(&'ast Typing<'ast>),
    VariadicArgs,
    SelfType,
    Fn(&'ast [Typing<'ast>], Option<&'ast Typing<'ast>>),
}

#[derive(Debug, Clone, Copy)]
pub enum Stmt<'ast> {
    ItemStmt(ItemStmt<'ast>),
    DefineStmt(&'ast DefineStmt<'ast>),
    AssignStmt(&'ast AssignStmt<'ast>),
    ExprStmt(Expr<'ast>),
}

#[derive(Debug, Clone, Copy)]
pub enum ItemStmt<'ast> {
    FnItem(&'ast FnItem<'ast>),
    StructItem(&'ast StructItem<'ast>),
    TypedefItem(&'ast TypedefItem<'ast>),
    EnumItem(&'ast EnumItem<'ast>),
    ImplItem(&'ast ImplItem<'ast>),
    CompDeclItem(CompDeclItem<'ast>),
}

#[derive(Debug, new)]
pub struct ImplItem<'ast> {
    pub ident_node: &'ast IdentNode,
    pub impl_fns: &'ast [&'ast FnItem<'ast>],
    pub ast_node_id: NodeId,
}

#[derive(Debug, Clone, Copy)]
pub enum CompDeclItem<'ast> {
    CompFnDeclItem(&'ast CompFnDeclItem<'ast>),
}

#[derive(Debug, new)]
pub struct CompFnDeclItem<'ast> {
    pub ident_node: &'ast IdentNode,
    pub args: &'ast [&'ast Field<'ast>],
    pub return_ty: Option<Typing<'ast>>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct TypedefItem<'ast> {
    pub ident_node: &'ast IdentNode,
    pub type_expr: Typing<'ast>,
    pub item_type: ItemType,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct FnItem<'ast> {
    pub ident_node: &'ast IdentNode,
    pub body: Stmts<'ast>,
    pub args: &'ast [ArgKind<'ast>],
    pub return_ty: Option<Typing<'ast>>,
    pub item_type: ItemType,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct StructItem<'ast> {
    pub ident_node: &'ast IdentNode,
    pub field_declarations: &'ast [&'ast Field<'ast>],
    pub item_type: ItemType,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct EnumItem<'ast> {
    pub ident_node: &'ast IdentNode,
    pub variants: &'ast [EnumVariant<'ast>],
    pub item_type: ItemType,
    pub ast_node_id: NodeId,
}
#[derive(Debug, new)]
pub struct EnumVariant<'ast> {
    pub ident_node: &'ast IdentNode,
    pub enum_data: Option<&'ast [Typing<'ast>]>,
}

#[derive(Debug, new)]
pub struct Field<'ast> {
    pub ident: &'ast IdentNode,
    pub type_expr: Typing<'ast>,
}

type Arg<'ast> = &'ast Field<'ast>;

#[derive(Debug, Clone, Copy)]
pub enum ArgKind<'ast> {
    /// `self`
    NormalSelf(&'ast IdentNode),
    /// `mut self`
    MutSelf(&'ast IdentNode),
    /// `*self`
    PtrSelf(&'ast IdentNode),
    /// `*mut self`
    MutPtrSelf(&'ast IdentNode),
    /// Any other argument
    Arg(Arg<'ast>),
}

#[derive(Debug, new)]
pub struct StructExpr<'ast> {
    pub ident_node: &'ast IdentNode,
    pub field_initializations: &'ast [&'ast FieldInitialization<'ast>],
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct FieldInitialization<'ast> {
    pub ident: &'ast IdentNode,
    pub value: Expr<'ast>,
}

#[derive(Debug, new)]
pub struct DefineStmt<'ast> {
    #[new(value = "None.into()")]
    pub mut_span: Cell<Option<Span>>,
    pub setter_expr: Pat<'ast>,
    pub value_expr: Expr<'ast>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, Clone, Copy)]
pub enum AsigneeExpr<'ast> {
    PlaceExpr(PlaceExpr<'ast>),
    CallExpr(&'ast CallExpr<'ast>),
}

#[derive(Debug, new)]
pub struct AssignStmt<'ast> {
    pub setter_expr: AsigneeExpr<'ast>,
    pub value_expr: Expr<'ast>,
    pub ast_node_id: NodeId,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum Pat<'ast> {
    /// E.g. x in `x := 5`
    IdentPat(&'ast IdentNode),
    // E.g.  in `Option.Some(x) := val`
    TupleStructPat(&'ast TupleStructPat<'ast>),
}

#[derive(Debug, new)]
pub struct TupleStructPat<'ast> {
    pub path: Path<'ast>,
    pub fields: &'ast [Pat<'ast>],
    pub ast_node_id: NodeId,
}

#[derive(Debug, Clone, Copy)]
pub enum Path<'ast> {
    PathSegment(&'ast IdentNode),
    PathField(&'ast PathField<'ast>),
}

#[derive(Debug, Clone, Copy, new)]
pub struct PathField<'ast> {
    pub lhs: Path<'ast>,
    pub rhs: &'ast IdentNode,
    pub ast_node_id: NodeId,
}

#[derive(Debug, Clone, Copy)]
pub enum Expr<'ast> {
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
}

#[derive(Debug, new)]
pub struct BlockExpr<'ast> {
    pub stmts: Stmts<'ast>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new, Clone, Copy)]
pub enum CondKind<'ast> {
    CondExpr(Expr<'ast>),
    CondPat(Pat<'ast>, Expr<'ast>),
}

#[derive(Debug, new)]
pub struct IfExpr<'ast> {
    pub cond_kind: CondKind<'ast>,
    pub true_block: Stmts<'ast>,
    pub false_block: Option<IfFalseBranchExpr<'ast>>,
    pub ast_node_id: NodeId,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum IfFalseBranchExpr<'ast> {
    ElseExpr(&'ast BlockExpr<'ast>),
    ElifExpr(&'ast IfExpr<'ast>),
}

#[derive(Debug, Clone, Copy)]
pub enum ExprWithoutBlock<'ast> {
    PlaceExpr(PlaceExpr<'ast>),
    ValueExpr(ValueExpr<'ast>),
    BreakExpr(&'ast BreakExpr<'ast>),
    ContinueExpr(&'ast ContinueExpr),
    ReturnExpr(&'ast ReturnExpr<'ast>),
}

#[derive(Debug, new)]
pub struct BreakExpr<'ast> {
    pub value: Option<Expr<'ast>>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct ReturnExpr<'ast> {
    pub value: Option<Expr<'ast>>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct ContinueExpr {
    pub ast_node_id: NodeId,
}

#[derive(Debug, Clone, Copy)]
pub enum PlaceExpr<'ast> {
    IdentExpr(&'ast IdentNode),
    TupleFieldExpr(&'ast TupleFieldExpr<'ast>),
    FieldExpr(&'ast FieldExpr<'ast>),
    IndexExpr(&'ast IndexExpr<'ast>),
}

#[derive(Debug, new)]
pub struct IndexExpr<'ast> {
    pub lhs: Expr<'ast>,
    pub value_expr: Expr<'ast>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct FieldExpr<'ast> {
    pub lhs: Expr<'ast>,
    pub rhs: &'ast IdentNode,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct TupleFieldExpr<'ast> {
    pub lhs: Expr<'ast>,
    pub rhs: &'ast IntegerExpr,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct IdentNode {
    pub span: Span,
    pub ast_node_id: NodeId,
}

impl IdentNode {
    pub fn get_copy(&self) -> Self {
        Self::new(self.span, self.ast_node_id)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ValueExpr<'ast> {
    BinaryExpr(&'ast BinaryExpr<'ast>),
    GroupExpr(&'ast GroupExpr<'ast>),
    TupleExpr(&'ast TupleExpr<'ast>),
    StructExpr(&'ast StructExpr<'ast>),
    ConstExpr(ConstExpr<'ast>),
    CallExpr(&'ast CallExpr<'ast>),
}

#[derive(Debug, new)]
pub struct CallExpr<'ast> {
    pub callee: Expr<'ast>,
    pub args: &'ast [Expr<'ast>],
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct TupleExpr<'ast> {
    pub fields: &'ast [Expr<'ast>],
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct GroupExpr<'ast> {
    pub expr: Expr<'ast>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, new)]
pub struct BinaryExpr<'ast> {
    pub lhs: Expr<'ast>,
    pub op: BinaryOp,
    pub rhs: Expr<'ast>,
    pub ast_node_id: NodeId,
}

#[derive(Debug, Clone, Copy)]
pub enum ConstExpr<'ast> {
    IntegerExpr(&'ast IntegerExpr),
    BoolExpr(&'ast BoolExpr),
    NullExpr(&'ast NullExpr),
    StringExpr(&'ast StringExpr),
}

#[derive(Debug, new)]
pub struct NullExpr {
    pub span: Span,
    pub ast_node_id: NodeId,
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

#[derive(Debug, new)]
pub struct StringExpr {
    pub val: Symbol,
    pub len: usize,
    pub ast_node_id: NodeId,
}

#[derive(Debug)]
pub struct FloatExpr {
    pub val: f64,
    pub ast_node_id: NodeId,
}

pub fn get_ident_node_from_arg_kind<'ast>(arg_kind: ArgKind<'ast>) -> &'ast IdentNode {
    match arg_kind {
        | ArgKind::NormalSelf(ident_node)
        | ArgKind::MutSelf(ident_node)
        | ArgKind::PtrSelf(ident_node)
        | ArgKind::MutPtrSelf(ident_node) => ident_node,
        ArgKind::Arg(field) => field.ident,
    }
}

pub fn get_node_id_from_expr(expr: Expr) -> NodeId {
    match expr {
        Expr::ExprWithBlock(expr_with_block) => {
            match expr_with_block {
                ExprWithBlock::BlockExpr(block_expr) => block_expr.ast_node_id,
                ExprWithBlock::IfExpr(if_expr) => if_expr.ast_node_id,
                ExprWithBlock::LoopExpr(loop_expr) => loop_expr.ast_node_id,
            }
        }
        Expr::ExprWithoutBlock(expr_without_block) => {
            match expr_without_block {
                ExprWithoutBlock::BreakExpr(break_expr) => break_expr.ast_node_id,
                ExprWithoutBlock::ContinueExpr(continue_expr) => continue_expr.ast_node_id,
                ExprWithoutBlock::ReturnExpr(return_expr) => return_expr.ast_node_id,
                ExprWithoutBlock::PlaceExpr(place_expr) => get_node_id_from_place_expr(place_expr),
                ExprWithoutBlock::ValueExpr(value_expr) => get_node_id_from_value_expr(value_expr),
            }
        }
    }
}

pub fn get_node_id_from_place_expr(place_expr: PlaceExpr) -> NodeId {
    match place_expr {
        PlaceExpr::IdentExpr(ident_expr) => ident_expr.ast_node_id,
        PlaceExpr::TupleFieldExpr(tuple_field_expr) => tuple_field_expr.ast_node_id,
        PlaceExpr::FieldExpr(field_expr) => field_expr.ast_node_id,
        PlaceExpr::IndexExpr(index_expr) => index_expr.ast_node_id,
    }
}

pub fn get_node_id_from_value_expr(value_expr: ValueExpr) -> NodeId {
    match value_expr {
        ValueExpr::BinaryExpr(binary_expr) => binary_expr.ast_node_id,
        ValueExpr::GroupExpr(group_expr) => group_expr.ast_node_id,
        ValueExpr::TupleExpr(tuple_expr) => tuple_expr.ast_node_id,
        ValueExpr::StructExpr(struct_expr) => struct_expr.ast_node_id,
        ValueExpr::CallExpr(call_expr) => call_expr.ast_node_id,
        ValueExpr::ConstExpr(const_expr) => {
            match const_expr {
                ConstExpr::BoolExpr(bool_expr) => bool_expr.ast_node_id,
                ConstExpr::IntegerExpr(integer_expr) => integer_expr.ast_node_id,
                ConstExpr::NullExpr(null_expr) => null_expr.ast_node_id,
                ConstExpr::StringExpr(string_expr) => string_expr.ast_node_id,
            }
        }
    }
}

pub fn get_node_id_from_pattern(pat: Pat) -> NodeId {
    match pat {
        Pat::IdentPat(ident_pat) => ident_pat.ast_node_id,
        Pat::TupleStructPat(tuple_pat) => tuple_pat.ast_node_id,
    }
}

pub fn is_stmt_adt(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::ItemStmt(
            ItemStmt::StructItem(_) | ItemStmt::EnumItem(_) | ItemStmt::TypedefItem(_),
        ) => true,
        _ => false,
    }
}
