use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub enum Op {
    BinaryOp(BinaryOp),
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BinaryOp(binary_op) => binary_op.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl Display for ArithmeticOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ComparisonOp {
    Eq,
    Ne,
    Ge,
    Gt,
    Le,
    Lt,
}

impl Display for ComparisonOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Ge => ">=",
            Self::Gt => ">",
            Self::Le => "<=",
            Self::Lt => "<",
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    ArithmeticOp(ArithmeticOp),
    ComparisonOp(ComparisonOp),
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArithmeticOp(arithmetic_op) => arithmetic_op.fmt(f),
            Self::ComparisonOp(comparison_op) => comparison_op.fmt(f),
        }
    }
}
