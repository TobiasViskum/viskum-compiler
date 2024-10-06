use ir_defs::ResKind;
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
    UndefinedLookup(Symbol, ResKind),
    MismatchedFieldTypes(Symbol, Symbol, Ty, Ty),
    MissingStructField(Symbol),
    UndefinedStructField(Symbol),
    ExpectedBoolExpr(Ty),
    AssignmentToImmutable(Symbol),
    BreakTypeError(Ty, Ty),
    BreakOutsideLoop,
    BinaryExprTypeError(BinaryOp, Ty, Ty),
    InvalidTuple(Ty),
    TupleAccessOutOfBounds(&'static [Ty], usize),
    InvalidPattern,
}

impl ErrorKind {
    fn get_severity(&self) -> Severity {
        match self {
            Self::InvalidTuple(_) => Severity::Fatal,
            Self::UndefinedLookup(_, _) => Severity::Fatal,
            Self::InvalidPattern => Severity::Fatal,
            Self::BinaryExprTypeError(_, _, _) => Severity::Fatal,
            Self::TupleAccessOutOfBounds(_, _) => Severity::Fatal,
            Self::MismatchedFieldTypes(_, _, _, _) => Severity::NoImpact,
            Self::MissingStructField(_) => Severity::NoImpact,
            Self::UndefinedStructField(_) => Severity::NoImpact,
            Self::BreakOutsideLoop => Severity::NoImpact,
            Self::BreakTypeError(_, _) => Severity::NoImpact,
            Self::ExpectedBoolExpr(_) => Severity::NoImpact,
            Self::AssignmentToImmutable(_) => Severity::NoImpact,
        }
    }

    pub fn write_msg(&self, buffer: &mut String, span: &Span, src: &str) {
        let write_error = match self {
            Self::TupleAccessOutOfBounds(tuple_ty, accessed_len) => {
                write!(
                    buffer,
                    "Tried to access element {} of tuple '{}', which only has {} elements",
                    accessed_len,
                    Ty::Tuple(tuple_ty),
                    tuple_ty.len()
                )
            }
            Self::InvalidTuple(found_ty) => {
                write!(buffer, "Expected tuple but found type `{}`", found_ty)
            }
            Self::MismatchedFieldTypes(struct_name, field_name, found_ty, expected_ty) => {
                write!(
                    buffer,
                    "Field `{}` in struct `{}` is of type `{}` but got type `{}`",
                    field_name.get(),
                    struct_name.get(),
                    expected_ty,
                    found_ty
                )
            }
            Self::MissingStructField(symbol) => {
                write!(buffer, "Missing struct field: `{}`", symbol.get())
            }
            Self::UndefinedStructField(symbol) => {
                write!(buffer, "Struct field doesn't exist `{}`", symbol.get())
            }
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
            Self::UndefinedLookup(symbol, kind) => {
                let kind_str = match kind {
                    ResKind::Struct => "struct",
                    ResKind::Variable => "varable",
                };

                write!(
                    buffer,
                    "Undefined {} `{}` at line {}",
                    kind_str,
                    symbol.get(),
                    span.get_line()
                )
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
