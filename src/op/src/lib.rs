use std::fmt::Display;

use core_traits::Dissasemble;

pub enum Op {
    BinaryOp(BinaryOp),
}

#[derive(Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl Dissasemble for BinaryOp {
    fn dissasemble(&self) -> String {
        match self {
            Self::Add => "+".to_string(),
            Self::Sub => "-".to_string(),
            Self::Mul => "*".to_string(),
            Self::Div => "/".to_string(),
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod test {
    use core_traits::Dissasemble;

    use crate::BinaryOp;

    #[test]
    fn binary_op() {
        assert_eq!(BinaryOp::Add.dissasemble(), "+");
        assert_eq!(BinaryOp::Sub.dissasemble(), "-");
        assert_eq!(BinaryOp::Mul.dissasemble(), "*");
        assert_eq!(BinaryOp::Div.dissasemble(), "/");
    }
}
