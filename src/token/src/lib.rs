use enum_iterator::Sequence;

#[derive(Debug, Clone, Copy)]
pub struct Token {
    kind: TokenKind,
    span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    byte_start: usize,
    byte_end: usize,
    line: usize,
}

impl Span {
    pub fn new(byte_start: usize, byte_end: usize, line: usize) -> Self {
        Self { byte_start, byte_end, line }
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
    /// `:=`
    Define,

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
