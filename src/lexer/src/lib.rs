use std::str::Chars;

use token::{ Span, Token, TokenKind };

pub const EOF_CHAR: char = '\0';

pub struct Lexer<'a> {
    chars: Chars<'a>,
    byte_start: usize,
    byte_current: usize,
    line: usize,
    current_char: char,
}

impl<'a> Lexer<'a> {
    pub fn new(file_content: &'a str) -> Self {
        Self {
            chars: file_content.chars(),
            byte_start: 0,
            byte_current: 0,
            line: 1,
            current_char: EOF_CHAR,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        let char = self.advance();

        match char {
            '+' => self.make_token(TokenKind::Plus),
            '-' => self.make_token(TokenKind::Minus),
            '*' => self.make_token(TokenKind::Star),
            '/' => self.make_token(TokenKind::Slash),
            ':' => self.make_token_or_other_if(TokenKind::Colon, '=', TokenKind::Define),
            ' ' => self.skip_char_and_scan(),
            '\n' => self.newline(),
            _ if Self::is_digit(char) => self.make_number(),
            _ if Self::is_alphabetic(char) => self.make_ident_or_keyword(),
            EOF_CHAR => self.make_token(TokenKind::Eof),
            _ => panic!("Unkown token: {}", char),
        }
    }

    fn make_token(&mut self, kind: TokenKind) -> Token {
        let token = Token::new(kind, self.span());
        self.reset_byte_tracker();
        token
    }

    fn make_token_or_other_if(
        &mut self,
        first_kind: TokenKind,
        predicate: char,
        other_kind: TokenKind
    ) -> Token {
        if self.next() == predicate {
            self.advance();
            self.make_token(other_kind)
        } else {
            self.make_token(first_kind)
        }
    }

    fn skip_char_and_scan(&mut self) -> Token {
        self.byte_current += self.current_char.len_utf8();
        self.byte_start = self.byte_current;
        self.scan_token()
    }

    fn newline(&mut self) -> Token {
        self.line += 1;
        self.skip_char_and_scan()
    }

    fn make_ident_or_keyword(&mut self) -> Token {
        let mut buffer = String::with_capacity(32);

        self.eat_while_do_from_current(
            |c| Self::is_alphabetic(c),
            |c| buffer.push(c)
        );

        if let Some(keyword_token_kind) = Self::match_keyword(buffer.as_str()) {
            self.make_token(keyword_token_kind)
        } else {
            self.make_token(TokenKind::Ident)
        }
    }

    fn match_keyword(ident: &str) -> Option<TokenKind> {
        // Make a faster way than a match statement here (match a char at a time)

        match ident {
            "def" => Some(TokenKind::Def),
            "class" => Some(TokenKind::Class),
            "end" => Some(TokenKind::End),
            "do" => Some(TokenKind::Do),
            "loop" => Some(TokenKind::Loop),
            "while" => Some(TokenKind::While),
            _ => None,
        }
    }

    fn make_number(&mut self) -> Token {
        if self.eat_while_from_next(|c| Self::is_digit(c)) == '.' {
            self.advance();
            self.eat_while_from_next(|c| Self::is_digit(c));
            self.make_token(TokenKind::Float)
        } else {
            self.make_token(TokenKind::Integer)
        }
    }

    fn advance(&mut self) -> char {
        let char = self.chars.next().unwrap_or(EOF_CHAR);
        self.byte_current += char.len_utf8();
        self.current_char = char;
        char
    }

    fn eat_while_from_next(&mut self, closure: impl Fn(char) -> bool) -> char {
        while closure(self.next()) && !self.is_eof() {
            self.advance();
        }
        self.next()
    }

    fn eat_while_do_from_current(
        &mut self,
        cond: impl Fn(char) -> bool,
        mut body: impl FnMut(char)
    ) -> char {
        while cond(self.current_char) && self.current_char != EOF_CHAR {
            body(self.current_char);
            self.advance();
        }
        self.current_char
    }

    fn next(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    fn span(&self) -> Span {
        Span::new(self.byte_start, self.byte_current, self.line)
    }

    fn reset_byte_tracker(&mut self) {
        self.byte_start = self.byte_current;
    }

    fn is_digit(char: char) -> bool {
        char >= '0' && char <= '9'
    }

    fn is_alphabetic(char: char) -> bool {
        char.is_alphabetic() || char == '_'
    }
}
