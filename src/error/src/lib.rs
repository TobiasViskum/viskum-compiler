use op::BinaryOp;
use span::Span;
use symbol::Symbol;
use ty::Ty;
use std::fmt::Write;

/// Describes how serious an error is
#[derive(PartialEq, Clone, Copy)]
pub enum Severity {
    /// Cannot continue
    Fatal,
    /// No effect on program, except at codegen
    NoImpact,
}

/// Use in Diagnostics instead
#[derive(Clone, Copy)]
pub struct Error {
    kind: ErrorKind,
    span: Span,
}

impl Error {
    pub fn new(kind: ErrorKind, span: Span) -> Self {
        Self {
            kind,
            span,
        }
    }

    pub fn get_severity(&self) -> Severity {
        self.kind.get_severity()
    }

    pub fn write_msg(&self, buffer: &mut String, src: &str) {
        self.kind.write_msg(buffer, &self.span, src);
        writeln!(buffer).expect("Unexpected write error")
    }
}

#[derive(Clone, Copy)]
pub enum ErrorKind {
    UndefinedVariable(Symbol),
    ExpectedBoolExpr(Ty),
    AssignmentToImmutable(Symbol),
    BreakTypeError(Ty, Ty),
    BreakOutsideLoop,
    BinaryExprTypeError(BinaryOp, Ty, Ty),
    InvalidPattern,
}

impl ErrorKind {
    fn get_severity(&self) -> Severity {
        match self {
            Self::UndefinedVariable(_) => Severity::Fatal,
            Self::InvalidPattern => Severity::Fatal,
            Self::BinaryExprTypeError(_, _, _) => Severity::Fatal,
            Self::BreakOutsideLoop => Severity::NoImpact,
            Self::BreakTypeError(_, _) => Severity::NoImpact,
            Self::ExpectedBoolExpr(_) => Severity::NoImpact,
            Self::AssignmentToImmutable(_) => Severity::NoImpact,
        }
    }

    pub fn write_msg(&self, buffer: &mut String, span: &Span, src: &str) {
        let write_error = match self {
            Self::BreakTypeError(expected_ty, found_ty) => {
                write!(buffer, "Expected type `{}` but found type `{}`", expected_ty, found_ty)
            }
            Self::BreakOutsideLoop => {
                write!(buffer, "Keyword `break` cannot be used outside of loops")
            }
            Self::BinaryExprTypeError(binary_op, lhs_ty, rhs_ty) => {
                write!(buffer, "`{}` is not defined for `{}` and `{}`", binary_op, lhs_ty, rhs_ty)
            }
            Self::AssignmentToImmutable(symbol) => {
                write!(buffer, "Cannot assign to immutable variable `{}`", symbol.get())
            }
            Self::UndefinedVariable(symbol) => {
                write!(buffer, "Undefined variable `{}` at line {}", symbol.get(), span.get_line())
            }
            Self::InvalidPattern => {
                write!(buffer, "Invalid pattern: {}", &src[span.get_byte_range()])
            }
            Self::ExpectedBoolExpr(found_ty) => {
                write!(
                    buffer,
                    "Expected an expression that returns a `Bool` but got `{}`",
                    found_ty
                )
            }
        };

        write_error.expect("Unexpected write error");
    }
}
