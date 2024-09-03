use op::BinaryOp;
use span::Span;

#[derive(Debug)]
pub enum Stmt<'ast> {
    DefineStmt(DefineStmt<'ast>),
    ExprStmt(Expr<'ast>),
}

#[derive(Debug)]
pub struct DefineStmt<'ast> {
    setter_expr: Expr<'ast>, // PatternExpr
    value_expr: Expr<'ast>,
}

impl<'ast> DefineStmt<'ast> {
    pub fn new(setter_expr: Expr<'ast>, value_expr: Expr<'ast>) -> Self {
        Self { setter_expr, value_expr }
    }
}

#[derive(Debug)]
pub enum PatternExpr {
    IdentPattern(IdentExpr),
}

#[derive(Debug)]
pub enum Expr<'ast> {
    ExprWithoutBlock(ExprWithoutBlock<'ast>),
}

#[derive(Debug)]
pub enum ExprWithoutBlock<'ast> {
    PlaceExpr(PlaceExpr),
    ValueExpr(ValueExpr<'ast>),
}

#[derive(Debug)]
pub enum PlaceExpr {
    IdentExpr(IdentExpr),
}

#[derive(Debug)]
pub struct IdentExpr {
    span: Span,
}

impl IdentExpr {
    pub fn new(span: Span) -> Self {
        Self { span }
    }

    pub fn get_lexeme<'a>(&self, src: &'a str) -> &'a str {
        &src[self.span.get_byte_start()..self.span.get_byte_end()]
    }
}

#[derive(Debug)]
pub enum ValueExpr<'ast> {
    BinaryExpr(BinaryExpr<'ast>),
    ConstExpr(ConstExpr),
}

#[derive(Debug)]
pub struct BinaryExpr<'ast> {
    lhs: &'ast mut Expr<'ast>,
    op: BinaryOp,
    rhs: &'ast mut Expr<'ast>,
}

impl<'ast> BinaryExpr<'ast> {
    pub fn new(lhs: &'ast mut Expr<'ast>, op: BinaryOp, rhs: &'ast mut Expr<'ast>) -> Self {
        Self {
            lhs,
            op,
            rhs,
        }
    }
}

#[derive(Debug)]
pub enum ConstExpr {
    IntegerExpr(IntegerExpr),
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

#[derive(Debug)]
pub struct FloatExpr {}
