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
pub struct Error<'ctx> {
    kind: ErrorKind<'ctx>,
    span: Span,
}

impl<'ctx> Error<'ctx> {
    pub fn new(kind: ErrorKind<'ctx>, span: Span) -> Self {
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
pub enum ErrorKind<'ctx> {
    UndefinedVariable(Symbol),
    ExpectedBoolExpr(&'ctx Ty),
    AssignmentToImmutable(Symbol),
    InvalidPattern,
}

impl<'ctx> ErrorKind<'ctx> {
    fn get_severity(&self) -> Severity {
        match self {
            Self::UndefinedVariable(_) => Severity::Fatal,
            Self::InvalidPattern => Severity::Fatal,
            Self::ExpectedBoolExpr(_) => Severity::NoImpact,
            Self::AssignmentToImmutable(_) => Severity::NoImpact,
        }
    }

    pub fn write_msg(&self, buffer: &mut String, span: &Span, src: &str) {
        let write_error = match self {
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
