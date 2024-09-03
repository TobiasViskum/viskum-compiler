use std::fmt::Display;

use enum_iterator::Sequence;
use span::Span;

#[derive(Debug, Clone, Copy)]
pub struct Token {
    kind: TokenKind,
    span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn get_kind(&self) -> TokenKind {
        self.kind
    }

    pub fn get_span(&self) -> Span {
        self.span
    }

    pub fn dummy() -> Self {
        Self {
            kind: TokenKind::Eof,
            span: Span::dummy(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum TokenKind {
    /* Symbols */
    /// `(`
    LeftParen,
    /// `)`
    RightParen,
    /// `=`
    Eq,
    /// `!=`
    Ne,
    /// `>=`
    Ge,
    /// `>`
    Gt,
    /// `<=`
    Le,
    /// `<`
    Lt,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `:`
    Colon,
    /// `=`
    Assign,
    /// `:=`
    Define,
    /// `.`
    Dot,

    /* Literals */
    /// Integer e.g. `69`
    Integer,
    /// Float e.g. `6.9`
    Float,

    /* Identifier */
    /// Identifier e.g. `a`
    Ident,

    /// Keyword `def`
    Def,
    /// Keyword `class`
    Class,
    /// Keyword `while`
    While,
    /// Keyword `loop`
    Loop,
    /// Keyword `do`
    Do,
    /// Keyword `end`
    End,

    /// End of ofile
    Eof,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
