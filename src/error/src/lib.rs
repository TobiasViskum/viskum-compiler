use span::Span;
use symbol::Symbol;
use std::fmt::Write;

/// Not used yet since there's only one variant
pub enum ErrorSeverity {
    Fatal,
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
}

impl Error {
    pub fn write_msg(&self, buffer: &mut String) {
        self.kind.write_msg(buffer, &self.span)
    }
}

#[derive(Clone, Copy)]
pub enum ErrorKind {
    UndefinedVariable(Symbol),
}

impl ErrorKind {
    pub fn write_msg(&self, buffer: &mut String, span: &Span) {
        let write_error = match self {
            Self::UndefinedVariable(symbol) => {
                writeln!(
                    buffer,
                    "Undefined variable `{}` at line {}",
                    symbol.get(),
                    span.get_line()
                )
            }
        };

        write_error.expect("Unexpected write error");
    }
}
