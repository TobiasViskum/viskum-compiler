pub enum Expr<'ast> {
    ExprWithoutBlock(ExprWithoutBlock<'ast>),
}

pub enum ExprWithoutBlock<'ast> {
    ValueExpr(ValueExpr<'ast>),
}

pub enum ValueExpr<'ast> {
    BinaryExpr(BinaryExpr<'ast>),
    ConstExpr(ConstExpr),
}

pub struct BinaryExpr<'ast> {
    lhs: &'ast mut Expr<'ast>,
    op: op::BinaryOp,
    rhs: &'ast mut Expr<'ast>,
}

pub enum ConstExpr {
    IntegerExpr(IntegerExpr),
}

pub struct IntegerExpr {
    val: i32,
}
pub struct FloatExpr {}
