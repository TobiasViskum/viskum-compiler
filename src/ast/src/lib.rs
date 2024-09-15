mod ast_arena;
// mod visit_unvalidated;
// mod visit_resolved;
pub mod ast_state;
pub mod ast_visitor;

use std::marker::PhantomData;

use ast_state::{ AstState, AstState1, AstState2, AstUnvalidated };
use ast_visitor::{ AstVisitEmitter, AstVisitor, Visitor };
// pub use visit_unvalidated::{ AstVisitEvent, ScopeChange, AstNodeKind };
use core_traits::Dissasemble;
use op::BinaryOp;
use span::Span;

pub use ast_arena::AstArena;
use ty::NodeId;

const AST_DISSASEMBLE_INDENT: usize = 4;

pub trait AstDissasemble {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String;
}

#[derive(Debug)]
pub struct Ast<'ast, T> where T: AstState {
    main_scope: GlobalScope<'ast>,
    _state: PhantomData<T>,
}

impl<'ast, T> Ast<'ast, T> where T: AstState {
    pub fn new(main_scope: GlobalScope<'ast>) -> Self {
        Self {
            main_scope,
            _state: PhantomData,
        }
    }

    pub fn dissasemble(&self, src: &str) -> String {
        self.main_scope.ast_dissasemble(0, src)
    }

    pub fn next_state<N>(self) -> Ast<'ast, N> where T: AstState<NextState = N>, N: AstState {
        Ast {
            main_scope: self.main_scope,
            _state: PhantomData,
        }
    }
}

impl<'ast> Ast<'ast, AstState1> {
    pub fn get_visitor<'b, E: AstVisitEmitter<'ast, AstState1>>(
        self,
        ast_visit_emitter: &'b mut E
    ) -> AstVisitor<'ast, 'b, AstState1, E>
        where 'ast: 'b
    {
        AstVisitor::new(self, ast_visit_emitter)
    }
}
impl<'ast> Ast<'ast, AstState2> {
    pub fn get_visitor<'b, E>(
        self,
        ast_visit_emitter: &'b mut E
    )
        -> AstVisitor<'ast, 'b, AstState2, E>
        where 'ast: 'b, E: AstVisitEmitter<'ast, AstState2>
    {
        AstVisitor::new(self, ast_visit_emitter)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Stmt<'ast> {
    ItemStmt(ItemStmt<'ast>),
    DefineStmt(DefineStmt<'ast>),
    AssignStmt(AssignStmt<'ast>),
    ExprStmt(Expr<'ast>),
}

impl<'ast> AstDissasemble for Stmt<'ast> {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::AssignStmt(stmt) => stmt.ast_dissasemble(scope_depth, src),
            Self::DefineStmt(stmt) => stmt.ast_dissasemble(scope_depth, src),
            Self::ExprStmt(stmt) => stmt.ast_dissasemble(scope_depth, src),
            Self::ItemStmt(stmt) => stmt.ast_dissasemble(scope_depth, src),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ItemStmt<'ast> {
    FunctionStmt(FunctionStmt<'ast>),
}

impl<'ast> AstDissasemble for ItemStmt<'ast> {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::FunctionStmt(stmt) => stmt.ast_dissasemble(scope_depth, src),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FunctionStmt<'ast> {
    pub ident_expr: IdentExpr,
    pub body: Expr<'ast>,
    pub ast_node_id: NodeId,
}
impl<'ast> FunctionStmt<'ast> {
    pub fn new(ident_expr: IdentExpr, body: Expr<'ast>, ast_node_id: NodeId) -> Self {
        Self { ident_expr, body, ast_node_id }
    }
}

impl<'ast> AstDissasemble for FunctionStmt<'ast> {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        format!(
            "def {}()\n{}end\n",
            self.ident_expr.ast_dissasemble(scope_depth, src),
            self.body.ast_dissasemble(scope_depth, src)
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DefineStmt<'ast> {
    pub setter_expr: Pat,
    pub value_expr: Expr<'ast>,
    pub ast_node_id: NodeId,
}

impl<'ast> DefineStmt<'ast> {
    pub fn new(setter_expr: Pat, value_expr: Expr<'ast>, ast_node_id: NodeId) -> Self {
        Self {
            setter_expr,
            value_expr,
            ast_node_id,
        }
    }
}

impl<'ast> AstDissasemble for DefineStmt<'ast> {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        format!(
            "{} := {}",
            self.setter_expr.ast_dissasemble(scope_depth, src),
            self.value_expr.ast_dissasemble(scope_depth, src)
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AssignStmt<'ast> {
    pub setter_expr: PlaceExpr,
    pub value_expr: Expr<'ast>,
    pub ast_node_id: NodeId,
}

impl<'ast> AssignStmt<'ast> {
    pub fn new(setter_expr: PlaceExpr, value_expr: Expr<'ast>, ast_node_id: NodeId) -> Self {
        Self {
            setter_expr,
            value_expr,
            ast_node_id,
        }
    }
}

impl<'ast> AstDissasemble for AssignStmt<'ast> {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        format!(
            "{} = {}",
            self.setter_expr.ast_dissasemble(scope_depth, src),
            self.value_expr.ast_dissasemble(scope_depth, src)
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Pat {
    IdentPat(IdentPat),
}

impl AstDissasemble for Pat {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::IdentPat(expr) => expr.ast_dissasemble(scope_depth, src),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Expr<'ast> {
    ExprWithBlock(ExprWithBlock<'ast>),
    ExprWithoutBlock(ExprWithoutBlock<'ast>),
}

impl<'ast> AstDissasemble for Expr<'ast> {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::ExprWithBlock(expr) => expr.ast_dissasemble(scope_depth, src),
            Self::ExprWithoutBlock(expr) => expr.ast_dissasemble(scope_depth, src),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExprWithBlock<'ast> {
    BlockExpr(&'ast BlockExpr<'ast>),
    IfExpr(&'ast IfExpr<'ast>),
}

impl<'ast> AstDissasemble for ExprWithBlock<'ast> {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::BlockExpr(expr) => expr.ast_dissasemble(scope_depth, src),
            Self::IfExpr(expr) => expr.ast_dissasemble(scope_depth, src),
        }
    }
}

type Stmts<'ast> = &'ast [&'ast Stmt<'ast>];

#[derive(Debug)]
pub struct GlobalScope<'ast> {
    pub stmts: Stmts<'ast>,
    pub ast_node_id: NodeId,
}

impl<'ast> GlobalScope<'ast> {
    pub fn new(stmts: Stmts<'ast>, ast_node_id: NodeId) -> Self {
        Self { stmts, ast_node_id }
    }
}
impl<'ast> AstDissasemble for GlobalScope<'ast> {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        let mut string_builder = String::new();
        for stmt in self.stmts.iter() {
            string_builder += format!(
                "{}{}\n",
                " ".repeat(AST_DISSASEMBLE_INDENT * scope_depth),
                stmt.ast_dissasemble(scope_depth, src)
            ).as_str();
        }

        string_builder
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BlockExpr<'ast> {
    pub stmts: Stmts<'ast>,
    pub ast_node_id: NodeId,
}

impl<'ast> BlockExpr<'ast> {
    pub fn new(stmts: Stmts<'ast>, ast_node_id: NodeId) -> Self {
        Self { stmts, ast_node_id }
    }
}

impl<'ast> AstDissasemble for BlockExpr<'ast> {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        let mut string_builder = format!(
            "{}do\n",
            " ".repeat(scope_depth * AST_DISSASEMBLE_INDENT)
        );
        for stmt in self.stmts.iter() {
            string_builder += format!(
                "{}{}\n",
                " ".repeat(AST_DISSASEMBLE_INDENT * (scope_depth + 1)),
                stmt.ast_dissasemble(scope_depth + 1, src)
            ).as_str();
        }
        string_builder += format!(
            "{}end ",
            " ".repeat(AST_DISSASEMBLE_INDENT * scope_depth)
        ).as_str();
        string_builder
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IfExpr<'ast> {
    pub condition: &'ast Expr<'ast>,
    pub true_block: &'ast BlockExpr<'ast>,
    pub false_block: Option<IfFalseBranchExpr<'ast>>,
    pub ast_node_id: NodeId,
}

impl<'ast> IfExpr<'ast> {
    pub fn new(
        condition: &'ast Expr<'ast>,
        true_block: &'ast BlockExpr<'ast>,
        false_block: Option<IfFalseBranchExpr<'ast>>,
        ast_node_id: NodeId
    ) -> Self {
        Self {
            condition,
            true_block,
            false_block,
            ast_node_id,
        }
    }
}

impl<'ast> AstDissasemble for IfExpr<'ast> {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        let mut string_builder = format!(
            "if {} \n",
            self.condition.ast_dissasemble(scope_depth, src)
        );
        string_builder += self.true_block.ast_dissasemble(scope_depth, src).as_str();

        match &self.false_block {
            Some(if_false) =>
                match if_false {
                    IfFalseBranchExpr::ElifExpr(expr) => {
                        string_builder += format!(
                            "el{}",
                            expr.ast_dissasemble(scope_depth, src)
                        ).as_str();
                    }
                    IfFalseBranchExpr::ElseExpr(expr) => {
                        string_builder += format!(
                            "else {}",
                            expr.ast_dissasemble(scope_depth, src)
                        ).as_str();
                    }
                }
            _ => {}
        }

        string_builder
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IfFalseBranchExpr<'ast> {
    ElseExpr(&'ast BlockExpr<'ast>),
    ElifExpr(&'ast IfExpr<'ast>),
}

impl<'ast> AstDissasemble for IfFalseBranchExpr<'ast> {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::ElifExpr(expr) => expr.ast_dissasemble(scope_depth, src),
            Self::ElseExpr(expr) => expr.ast_dissasemble(scope_depth, src),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExprWithoutBlock<'ast> {
    PlaceExpr(PlaceExpr),
    ValueExpr(ValueExpr<'ast>),
}

impl<'ast> AstDissasemble for ExprWithoutBlock<'ast> {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::PlaceExpr(expr) => expr.ast_dissasemble(scope_depth, src),
            Self::ValueExpr(expr) => expr.ast_dissasemble(scope_depth, src),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PlaceExpr {
    IdentExpr(IdentExpr),
}

impl AstDissasemble for PlaceExpr {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::IdentExpr(expr) => expr.ast_dissasemble(scope_depth, src),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IdentPat {
    pub span: Span,
    pub ast_node_id: NodeId,
}

impl IdentPat {
    pub fn new(span: Span, ast_node_id: NodeId) -> Self {
        Self {
            span,
            ast_node_id,
        }
    }

    pub fn get_lexeme<'a>(&self, src: &'a str) -> &'a str {
        &src[self.span.get_byte_start()..self.span.get_byte_end()]
    }
}

impl AstDissasemble for IdentPat {
    fn ast_dissasemble(&self, _: usize, src: &str) -> String {
        src[self.span.get_byte_range()].to_string()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IdentExpr {
    pub span: Span,
    pub ast_node_id: NodeId,
}

impl IdentExpr {
    pub fn new(span: Span, ast_node_id: NodeId) -> Self {
        Self {
            span,
            ast_node_id,
        }
    }

    pub fn get_lexeme<'a>(&self, src: &'a str) -> &'a str {
        &src[self.span.get_byte_start()..self.span.get_byte_end()]
    }

    pub fn as_pat(self) -> IdentPat {
        IdentPat::new(self.span, self.ast_node_id)
    }
}

impl AstDissasemble for IdentExpr {
    fn ast_dissasemble(&self, _: usize, src: &str) -> String {
        src[self.span.get_byte_range()].to_string()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ValueExpr<'ast> {
    BinaryExpr(BinaryExpr<'ast>),
    GroupExpr(GroupExpr<'ast>),
    ConstExpr(ConstExpr),
}

impl<'ast> AstDissasemble for ValueExpr<'ast> {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::BinaryExpr(expr) => expr.ast_dissasemble(scope_depth, src),
            Self::GroupExpr(expr) => expr.ast_dissasemble(scope_depth, src),
            Self::ConstExpr(expr) => expr.dissasemble(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GroupExpr<'ast> {
    pub expr: &'ast Expr<'ast>,
    pub ast_node_id: NodeId,
}

impl<'ast> GroupExpr<'ast> {
    pub fn new(expr: &'ast Expr<'ast>, ast_node_id: NodeId) -> Self {
        Self { expr, ast_node_id }
    }
}

impl<'ast> AstDissasemble for GroupExpr<'ast> {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        format!("({})", self.expr.ast_dissasemble(scope_depth, src))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BinaryExpr<'ast> {
    pub lhs: &'ast Expr<'ast>,
    pub op: BinaryOp,
    pub rhs: &'ast Expr<'ast>,
    pub ast_node_id: NodeId,
}

impl<'ast> BinaryExpr<'ast> {
    pub fn new(
        lhs: &'ast Expr<'ast>,
        op: BinaryOp,
        rhs: &'ast Expr<'ast>,
        ast_node_id: NodeId
    ) -> Self {
        Self { lhs, op, rhs, ast_node_id }
    }
}
impl<'ast> AstDissasemble for BinaryExpr<'ast> {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        format!(
            "{} {} {}",
            self.lhs.ast_dissasemble(scope_depth, src),
            self.op.dissasemble(),
            self.rhs.ast_dissasemble(scope_depth, src)
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConstExpr {
    IntegerExpr(IntegerExpr),
}

impl Dissasemble for ConstExpr {
    fn dissasemble(&self) -> String {
        match self {
            Self::IntegerExpr(expr) => expr.dissasemble(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IntegerExpr {
    pub val: i64,
    pub ast_node_id: NodeId,
}

impl IntegerExpr {
    pub fn new(val: i64, ast_node_id: NodeId) -> Self {
        Self { val, ast_node_id }
    }
}

impl Dissasemble for IntegerExpr {
    fn dissasemble(&self) -> String {
        self.val.to_string()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FloatExpr {
    pub val: f64,
    pub ast_node_id: NodeId,
}
