use std::marker::PhantomData;
mod ast_arena;

use ast_state::{ AstState, AstUnvalidated, AstValidated };
use core_traits::Dissasemble;
use op::BinaryOp;
use span::Span;

pub use ast_arena::AstArena;

const AST_DISSASEMBLE_INDENT: usize = 4;

pub trait AstDissasemble {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String;
}

#[derive(Debug)]
pub struct Ast<'ast, T> where T: AstState {
    main_scope: GlobalScope<'ast, T>, // Very nested
    /// To make sure the arena lives at least as long as the ast
    _ast_arena: &'ast AstArena<'ast>,
}

impl<'ast, T> Ast<'ast, T> where T: AstState {
    pub fn new(main_scope: GlobalScope<'ast, T>, ast_arena: &'ast AstArena<'ast>) -> Self {
        Self { main_scope, _ast_arena: ast_arena }
    }

    pub fn dissasemble(&self, src: &str) -> String {
        self.main_scope.ast_dissasemble(0, src)
    }
}

impl<'ast> Ast<'ast, AstUnvalidated> {
    pub fn _next_state(self) -> Ast<'ast, AstValidated> {
        unsafe { std::mem::transmute(self) }
    }
}

impl<'ast> Ast<'ast, AstValidated> {}

#[derive(Debug)]
pub enum Stmt<'ast, T> where T: AstState {
    ItemStmt(ItemStmt<'ast, T>),
    DefineStmt(DefineStmt<'ast, T>),
    AssignStmt(AssignStmt<'ast, T>),
    ExprStmt(Expr<'ast, T>),
}

impl<'ast, T> AstDissasemble for Stmt<'ast, T> where T: AstState {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::AssignStmt(stmt) => stmt.ast_dissasemble(scope_depth, src),
            Self::DefineStmt(stmt) => stmt.ast_dissasemble(scope_depth, src),
            Self::ExprStmt(stmt) => stmt.ast_dissasemble(scope_depth, src),
            Self::ItemStmt(stmt) => stmt.ast_dissasemble(scope_depth, src),
        }
    }
}

#[derive(Debug)]
pub enum ItemStmt<'ast, T> where T: AstState {
    FunctionStmt(FunctionStmt<'ast, T>),
}

impl<'ast, T> AstDissasemble for ItemStmt<'ast, T> where T: AstState {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::FunctionStmt(stmt) => stmt.ast_dissasemble(scope_depth, src),
        }
    }
}

#[derive(Debug)]
pub struct FunctionStmt<'ast, T> where T: AstState {
    ident_expr: IdentExpr<T>,
    body: Expr<'ast, T>,
}
impl<'ast, T> FunctionStmt<'ast, T> where T: AstState {
    pub fn new(ident_expr: IdentExpr<T>, body: Expr<'ast, T>) -> Self {
        Self { ident_expr, body }
    }
}

impl<'ast, T> AstDissasemble for FunctionStmt<'ast, T> where T: AstState {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        format!(
            "def {}()\n{}end\n",
            self.ident_expr.ast_dissasemble(scope_depth, src),
            self.body.ast_dissasemble(scope_depth, src)
        )
    }
}

#[derive(Debug)]
pub struct DefineStmt<'ast, T> where T: AstState {
    setter_expr: PatternExpr<T>,
    value_expr: Expr<'ast, T>,
}

impl<'ast, T> DefineStmt<'ast, T> where T: AstState {
    pub fn new(setter_expr: PatternExpr<T>, value_expr: Expr<'ast, T>) -> Self {
        Self { setter_expr, value_expr }
    }
}

impl<'ast, T> AstDissasemble for DefineStmt<'ast, T> where T: AstState {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        format!(
            "{} := {}",
            self.setter_expr.ast_dissasemble(scope_depth, src),
            self.value_expr.ast_dissasemble(scope_depth, src)
        )
    }
}

#[derive(Debug)]
pub struct AssignStmt<'ast, T> where T: AstState {
    setter_expr: PlaceExpr<T>,
    value_expr: Expr<'ast, T>,
}

impl<'ast, T> AssignStmt<'ast, T> where T: AstState {
    pub fn new(setter_expr: PlaceExpr<T>, value_expr: Expr<'ast, T>) -> Self {
        Self { setter_expr, value_expr }
    }
}

impl<'ast, T> AstDissasemble for AssignStmt<'ast, T> where T: AstState {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        format!(
            "{} = {}",
            self.setter_expr.ast_dissasemble(scope_depth, src),
            self.value_expr.ast_dissasemble(scope_depth, src)
        )
    }
}

#[derive(Debug)]
pub enum PatternExpr<T> where T: AstState {
    IdentPattern(IdentExpr<T>),
}

impl<T> AstDissasemble for PatternExpr<T> where T: AstState {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::IdentPattern(expr) => expr.ast_dissasemble(scope_depth, src),
        }
    }
}

#[derive(Debug)]
pub enum Expr<'ast, T> where T: AstState {
    ExprWithBlock(ExprWithBlock<'ast, T>),
    ExprWithoutBlock(ExprWithoutBlock<'ast, T>),
}

impl<'ast, T> AstDissasemble for Expr<'ast, T> where T: AstState {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::ExprWithBlock(expr) => expr.ast_dissasemble(scope_depth, src),
            Self::ExprWithoutBlock(expr) => expr.ast_dissasemble(scope_depth, src),
        }
    }
}

#[derive(Debug)]
pub enum ExprWithBlock<'ast, T> where T: AstState {
    BlockExpr(BlockExpr<'ast, T>),
    IfExpr(IfExpr<'ast, T>),
}

impl<'ast, T> AstDissasemble for ExprWithBlock<'ast, T> where T: AstState {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::BlockExpr(expr) => expr.ast_dissasemble(scope_depth, src),
            Self::IfExpr(expr) => expr.ast_dissasemble(scope_depth, src),
        }
    }
}

type Stmts<'ast, T> = Vec<Stmt<'ast, T>>;

#[derive(Debug)]
pub struct GlobalScope<'ast, T> where T: AstState {
    stmts: Stmts<'ast, T>,
}
impl<'ast, T> GlobalScope<'ast, T> where T: AstState {
    pub fn new(stmts: Stmts<'ast, T>) -> Self {
        Self {
            stmts,
        }
    }
}
impl<'ast, T> AstDissasemble for GlobalScope<'ast, T> where T: AstState {
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

#[derive(Debug)]
pub struct BlockExpr<'ast, T> where T: AstState {
    stmts: Stmts<'ast, T>,
}
impl<'ast, T> BlockExpr<'ast, T> where T: AstState {
    pub fn new(stmts: Stmts<'ast, T>) -> Self {
        Self {
            stmts,
        }
    }
}

impl<'ast, T> AstDissasemble for BlockExpr<'ast, T> where T: AstState {
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

#[derive(Debug)]
pub struct IfExpr<'ast, T> where T: AstState {
    condition: &'ast Expr<'ast, T>,
    true_block: BlockExpr<'ast, T>,
    false_block: Option<IfFalseExpr<'ast, T>>,
}

impl<'ast, T> IfExpr<'ast, T> where T: AstState {
    pub fn new(
        condition: &'ast Expr<'ast, T>,
        true_block: BlockExpr<'ast, T>,
        false_block: Option<IfFalseExpr<'ast, T>>
    ) -> Self {
        Self { condition, true_block, false_block }
    }
}

impl<'ast, T> AstDissasemble for IfExpr<'ast, T> where T: AstState {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        let mut string_builder = format!(
            "if {} \n",
            self.condition.ast_dissasemble(scope_depth, src)
        );
        string_builder += self.true_block.ast_dissasemble(scope_depth, src).as_str();

        match &self.false_block {
            Some(if_false) => {
                match if_false {
                    IfFalseExpr::ElifExpr(expr) => {
                        string_builder += format!(
                            "el{}",
                            expr.ast_dissasemble(scope_depth, src)
                        ).as_str();
                    }
                    IfFalseExpr::ElseExpr(expr) => {
                        string_builder += format!(
                            "else {}",
                            expr.ast_dissasemble(scope_depth, src)
                        ).as_str();
                    }
                }
            }
            _ => {}
        }

        string_builder
    }
}

#[derive(Debug)]
pub enum IfFalseExpr<'ast, T> where T: AstState {
    ElseExpr(BlockExpr<'ast, T>),
    ElifExpr(&'ast IfExpr<'ast, T>),
}

impl<'ast, T> AstDissasemble for IfFalseExpr<'ast, T> where T: AstState {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::ElifExpr(expr) => expr.ast_dissasemble(scope_depth, src),
            Self::ElseExpr(expr) => expr.ast_dissasemble(scope_depth, src),
        }
    }
}

#[derive(Debug)]
pub enum ExprWithoutBlock<'ast, T> where T: AstState {
    PlaceExpr(PlaceExpr<T>),
    ValueExpr(ValueExpr<'ast, T>),
}

impl<'ast, T> AstDissasemble for ExprWithoutBlock<'ast, T> where T: AstState {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::PlaceExpr(expr) => expr.ast_dissasemble(scope_depth, src),
            Self::ValueExpr(expr) => expr.ast_dissasemble(scope_depth, src),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PlaceExpr<T> where T: AstState {
    IdentExpr(IdentExpr<T>),
}

impl<T> AstDissasemble for PlaceExpr<T> where T: AstState {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::IdentExpr(expr) => expr.ast_dissasemble(scope_depth, src),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IdentExpr<T> {
    span: Span,
    marker: PhantomData<T>,
}

impl<T> IdentExpr<T> where T: AstState {
    pub fn new(span: Span) -> Self {
        Self { span, marker: PhantomData }
    }

    pub fn get_lexeme<'a>(&self, src: &'a str) -> &'a str {
        &src[self.span.get_byte_start()..self.span.get_byte_end()]
    }
}

impl<T> AstDissasemble for IdentExpr<T> where T: AstState {
    fn ast_dissasemble(&self, _: usize, src: &str) -> String {
        src[self.span.get_byte_range()].to_string()
    }
}

#[derive(Debug)]
pub enum ValueExpr<'ast, T> where T: AstState {
    BinaryExpr(BinaryExpr<'ast, T>),
    GroupExpr(GroupExpr<'ast, T>),
    ConstExpr(ConstExpr),
}

impl<'ast, T> AstDissasemble for ValueExpr<'ast, T> where T: AstState {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        match self {
            Self::BinaryExpr(expr) => expr.ast_dissasemble(scope_depth, src),
            Self::GroupExpr(expr) => expr.ast_dissasemble(scope_depth, src),
            Self::ConstExpr(expr) => expr.dissasemble(),
        }
    }
}

#[derive(Debug)]
pub struct GroupExpr<'ast, T> where T: AstState {
    expr: &'ast Expr<'ast, T>,
}

impl<'ast, T> GroupExpr<'ast, T> where T: AstState {
    pub fn new(expr: &'ast Expr<'ast, T>) -> Self {
        Self { expr }
    }
}

impl<'ast, T> AstDissasemble for GroupExpr<'ast, T> where T: AstState {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        format!("({})", self.expr.ast_dissasemble(scope_depth, src))
    }
}

#[derive(Debug)]
pub struct BinaryExpr<'ast, T> where T: AstState {
    lhs: &'ast Expr<'ast, T>,
    op: BinaryOp,
    rhs: &'ast Expr<'ast, T>,
}

impl<'ast, T> BinaryExpr<'ast, T> where T: AstState {
    pub fn new(lhs: &'ast Expr<'ast, T>, op: BinaryOp, rhs: &'ast Expr<'ast, T>) -> Self {
        Self {
            lhs,
            op,
            rhs,
        }
    }
}
impl<'ast, T> AstDissasemble for BinaryExpr<'ast, T> where T: AstState {
    fn ast_dissasemble(&self, scope_depth: usize, src: &str) -> String {
        format!(
            "{} {} {}",
            self.lhs.ast_dissasemble(scope_depth, src),
            self.op.dissasemble(),
            self.rhs.ast_dissasemble(scope_depth, src)
        )
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct IntegerExpr {
    val: i32,
}

impl IntegerExpr {
    pub fn new(val: i32) -> Self {
        Self { val }
    }
}

impl Dissasemble for IntegerExpr {
    fn dissasemble(&self) -> String {
        self.val.to_string()
    }
}

#[derive(Debug)]
pub struct FloatExpr {
    val: f64,
}
