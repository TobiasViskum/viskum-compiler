use std::{ fmt::Display, ops::Add };

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
    /// `{`
    LeftCurly,
    /// `}`
    RightCurly,
    /// `[`
    LeftSquare,
    /// `]`
    RightSquare,
    /// `==`
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
    /// `,`
    Comma,
    /// `!`
    Bang,
    /// `++`
    Increment,
    /// `--`
    Decrement,

    /* Literals */
    /// Integer e.g. `69`
    Integer,
    /// Float e.g. `6.9`
    Float,

    /* Booleans */
    /// Boolean literal `true`
    True,
    /// Boolean literal `false`
    False,

    /* Identifier */
    /// Identifier e.g. `a`
    Ident,

    /* Null */
    /// Null literal `null`
    Null,

    /* Keywords */
    /// Keyword `declare`
    Declare,
    /// Keyword `def`
    Fn,
    /// Keyword `def`
    Def,
    /// Keyword `class`
    Class,
    /// Keyword `struct`
    Struct,
    /// Keyword `enum`
    Enum,
    /// Keyword `if`
    If,
    /// Keyword `while`
    While,
    /// Keyword `loop`
    Loop,
    /// Keyword `break`
    Break,
    /// Keyword `continue`
    Continue,
    /// Keyword `else`
    Else,
    /// Keyword `elif`
    Elif,
    /// Keyword `mut`
    Mut,
    /// Keyword `ret`
    Return,
    /// Keyword `typedef`
    Typedef,

    /// End of ofile
    Eof,
}

impl TokenKind {
    pub fn has_assign_prec(&self) -> bool {
        matches!(self, Self::Assign | Self::Define)
    }

    pub fn can_end_scope(&self) -> bool {
        matches!(self, Self::RightCurly)
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
