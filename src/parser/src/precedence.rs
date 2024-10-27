use std::fmt::Display;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
/// The higher the precedence
pub enum Precedence {
    /// No precedence. Cannot appear in an expression
    PrecNone,
    /// Precedence of `=` and `:=`. Used as an infix precedence
    PrecAssign,
    /// Precedence of `or`
    PrecOr,
    /// Precedence of `and`
    PrecAnd,
    /// Precedence of `==` and `!=`
    PrecEquality,
    /// Precedence of `>=`, `>`, `<=` and `<`
    PrecComparison,
    /// Precedence of `+` and `-`
    PrecTerm,
    /// Precedence of `*` and `/`
    PrecFactor,
    /// Precedence of unary operators: `!`, `-`, `.` (prefix)
    PrecUnary,
    /// Precedence of `()` and `.` (infix)
    PrecCall,
    /// Precedence of `[` (infix)
    PrecIndex,
    /// Used for tokens that can appear anywhere in expression and isn't tied to any precedence (e.g. `{` as a struct expression)
    PrecPrimary,
}

impl From<usize> for Precedence {
    fn from(value: usize) -> Self {
        match value {
            0 => Precedence::PrecNone,
            1 => Precedence::PrecAssign,
            2 => Precedence::PrecOr,
            3 => Precedence::PrecAnd,
            4 => Precedence::PrecEquality,
            5 => Precedence::PrecComparison,
            6 => Precedence::PrecTerm,
            7 => Precedence::PrecFactor,
            8 => Precedence::PrecUnary,
            9 => Precedence::PrecCall,
            10 => Precedence::PrecPrimary,
            _ => panic!("Invalid precedence value: {}", value),
        }
    }
}
impl Precedence {
    pub fn get_next(self) -> Self {
        if self == Precedence::PrecPrimary {
            Precedence::PrecNone
        } else {
            Precedence::from((self as usize) + 1)
        }
    }
    pub fn get_previous(self) -> Self {
        if self == Precedence::PrecNone {
            Precedence::PrecPrimary
        } else {
            Precedence::from((self as usize) - 1)
        }
    }
}

impl Display for Precedence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
