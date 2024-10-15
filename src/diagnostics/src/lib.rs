use ir::{ Symbol, Ty };
use span::Span;
use std::{ fmt::Write, marker::PhantomData };

pub struct ProgramDiagnostics<'ctx> {
    diagnostics: Vec<Diagnostic<'ctx>>,
}

pub struct Diagnostic<'ctx> {
    kind: DiagnosticKind,
    suggestions: Vec<Suggestion<'ctx>>,
}

pub struct Suggestion<'ctx> {
    kind: SuggestionKind<'ctx>,
}

pub enum SuggestionKind<'ctx> {
    E(PhantomData<&'ctx ()>),
}

pub enum DiagnosticKind {
    Error(Error),
    // Warning
}

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

/// Describes how serious an error is
#[derive(PartialEq, Clone, Copy)]
pub enum Severity {
    /// Cannot continue
    Fatal,
    /// No effect on program, except at codegen
    NoImpact,
}

#[derive(Clone, Copy)]
pub enum ErrorKind {
    UndefinedVariable(Symbol),
    ExpectedBoolExpr(Ty),
    InvalidPattern,
}

impl ErrorKind {
    fn get_severity(&self) -> Severity {
        match self {
            Self::UndefinedVariable(_) => Severity::Fatal,
            Self::InvalidPattern => Severity::Fatal,
            Self::ExpectedBoolExpr(_) => Severity::NoImpact,
        }
    }

    pub fn write_msg(&self, buffer: &mut String, span: &Span, src: &str) {
        let write_error = match self {
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
