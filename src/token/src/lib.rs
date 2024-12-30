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
    /// `"`
    DoubleQuote,
    /// Any character inside a string
    StringChar,
    /// `...`
    Ellipsis,

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
    /// Keyword `impl`
    Impl,
    /// Keyword `self`
    SmallSelf,
    /// Keyword `Self`
    BigSelf,
    /// Keyword `declare`
    Declare,
    /// Keyword `fn`
    Fn,
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
    /// Keyword `import`
    Import,
    /// Keyword `export`
    Export,
    /// Keyword `from`
    From,
    /// Keyword `pkg`
    Pkg,

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

    pub const fn to_keyword_str(&self) -> &str {
        match self {
            Self::Impl => "impl",
            Self::SmallSelf => "self",
            Self::BigSelf => "Self",
            Self::Declare => "declare",
            Self::Fn => "fn",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::If => "if",
            Self::While => "while",
            Self::Loop => "loop",
            Self::Break => "break",
            Self::Continue => "continue",
            Self::Else => "else",
            Self::Elif => "elif",
            Self::Mut => "mut",
            Self::Return => "ret",
            Self::Typedef => "typedef",
            Self::Import => "import",
            Self::Export => "export",
            Self::From => "from",
            Self::True => "true",
            Self::False => "false",
            Self::Null => "null",
            Self::Pkg => "pkg",
            _ => "",
        }
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
