use std::{ str::Chars, time::Instant };

use span::Span;
use token::{ Token, TokenKind };

pub const EOF_CHAR: char = '\0';

pub struct Lexer<'a> {
    chars: Chars<'a>,
    byte_start: usize,
    byte_current: usize,
    line: usize,
    current_char: char,
    str_layer: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(file_content: &'a str) -> Self {
        Self {
            chars: file_content.chars(),
            byte_start: 0,
            byte_current: 0,
            line: 1,
            current_char: EOF_CHAR,
            str_layer: 0,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        let char = self.advance();

        if self.is_tokenizing_string() {
            return match char {
                '"' => {
                    self.str_layer -= 1;
                    return self.make_token(TokenKind::DoubleQuote);
                }
                _ => self.make_token(TokenKind::StringChar),
            };
        }

        match char {
            '+' => self.make_token_or_other_if(TokenKind::Plus, '+', TokenKind::Increment),
            '-' => self.make_token(TokenKind::Minus),
            '*' => self.make_token(TokenKind::Star),
            '/' => self.make_token(TokenKind::Slash),
            '(' => self.make_token(TokenKind::LeftParen),
            ')' => self.make_token(TokenKind::RightParen),
            '{' => self.make_token(TokenKind::LeftCurly),
            '}' => self.make_token(TokenKind::RightCurly),
            '[' => self.make_token(TokenKind::LeftSquare),
            ']' => self.make_token(TokenKind::RightSquare),
            '"' => {
                self.str_layer += 1;
                self.make_token(TokenKind::DoubleQuote)
            }
            '!' => self.make_token_or_other_if(TokenKind::Bang, '=', TokenKind::Ne),
            '>' => self.make_token_or_other_if(TokenKind::Gt, '=', TokenKind::Ge),
            '<' => self.make_token_or_other_if(TokenKind::Lt, '=', TokenKind::Le),
            ':' => self.make_token_or_other_if(TokenKind::Colon, '=', TokenKind::Define),
            '=' => self.make_token_or_other_if(TokenKind::Assign, '=', TokenKind::Eq),
            ' ' => self.skip_char_and_scan(),
            ',' => self.make_token(TokenKind::Comma),
            '\n' => self.newline_and_scan(),
            // this shouldn't be called if char before is ident or ')'
            // '.' if Self::can_be_before_dot_float(prev) && Self::is_digit(self.peek_next()) => {
            //     self.make_float_number()
            // }
            '.' => {
                if self.peek_next() == '.' && self.peek_two_next() == '.' {
                    self.advance();
                    self.advance();

                    self.make_token(TokenKind::Ellipsis)
                } else {
                    self.make_token(TokenKind::Dot)
                }
            }
            _ if Self::is_digit(char) => self.make_number(),
            _ if Self::is_alphabetic(char) => self.make_ident_or_keyword(),
            EOF_CHAR => self.make_token(TokenKind::Eof),
            _ => panic!("Unkown token: {} (produce token_kind: Unknown)", char),
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
        if self.peek_next() == predicate {
            self.advance();
            self.make_token(other_kind)
        } else {
            self.make_token(first_kind)
        }
    }

    fn skip_char_and_scan(&mut self) -> Token {
        self.reset_byte_tracker();
        self.scan_token()
    }

    fn newline_and_scan(&mut self) -> Token {
        self.line += 1;
        self.skip_char_and_scan()
    }

    fn make_ident_or_keyword(&mut self) -> Token {
        // The length of the word is probably not more than 64 characters
        let mut buffer = String::with_capacity(64);

        if Self::is_alphabetic(self.current_char) {
            buffer.push(self.eat_if(|c| Self::is_alphabetic(c)));
        }

        self.eat_while_do_from_current(
            |c| (Self::is_alphabetic(c) || Self::is_digit(c)),
            |c| buffer.push(c)
        );

        self.make_token(Self::match_keyword_or_ident(buffer.as_str()))
    }

    fn match_keyword_or_ident(ident: &str) -> TokenKind {
        // Make a faster way than a match statement here (match a char at a time)

        match ident {
            t if t == TokenKind::Import.to_keyword_str() => TokenKind::Import,
            t if t == TokenKind::From.to_keyword_str() => TokenKind::From,
            t if t == TokenKind::Export.to_keyword_str() => TokenKind::Export,
            t if t == TokenKind::Fn.to_keyword_str() => TokenKind::Fn,
            t if t == TokenKind::Declare.to_keyword_str() => TokenKind::Declare,
            t if t == TokenKind::SmallSelf.to_keyword_str() => TokenKind::SmallSelf,
            t if t == TokenKind::BigSelf.to_keyword_str() => TokenKind::BigSelf,
            t if t == TokenKind::Mut.to_keyword_str() => TokenKind::Mut,
            t if t == TokenKind::Impl.to_keyword_str() => TokenKind::Impl,
            t if t == TokenKind::Struct.to_keyword_str() => TokenKind::Struct,
            t if t == TokenKind::Enum.to_keyword_str() => TokenKind::Enum,
            t if t == TokenKind::Null.to_keyword_str() => TokenKind::Null,
            t if t == TokenKind::Loop.to_keyword_str() => TokenKind::Loop,
            t if t == TokenKind::While.to_keyword_str() => TokenKind::While,
            t if t == TokenKind::If.to_keyword_str() => TokenKind::If,
            t if t == TokenKind::Else.to_keyword_str() => TokenKind::Else,
            t if t == TokenKind::Elif.to_keyword_str() => TokenKind::Elif,
            t if t == TokenKind::Break.to_keyword_str() => TokenKind::Break,
            t if t == TokenKind::Continue.to_keyword_str() => TokenKind::Continue,
            t if t == TokenKind::Return.to_keyword_str() => TokenKind::Return,
            t if t == TokenKind::True.to_keyword_str() => TokenKind::True,
            t if t == TokenKind::False.to_keyword_str() => TokenKind::False,
            t if t == TokenKind::Typedef.to_keyword_str() => TokenKind::Typedef,
            _ => TokenKind::Ident,
        }
    }

    fn make_float_number(&mut self) -> Token {
        self.advance();
        self.eat_while_from_next(|c| Self::is_digit(c));
        self.make_token(TokenKind::Float)
    }

    fn make_number(&mut self) -> Token {
        if self.eat_while_from_next(|c| Self::is_digit(c)) == '@' {
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

    /// If predicate from next char is true, then advances
    ///
    /// At the end it returns the next char
    fn eat_while_from_next(&mut self, predicate: impl Fn(char) -> bool) -> char {
        while predicate(self.peek_next()) && !self.is_eof() {
            self.advance();
        }
        self.peek_next()
    }

    /// Returns the current char and if predicate is true then also advances
    fn eat_if(&mut self, predicate: impl Fn(char) -> bool) -> char {
        let c = self.current_char;
        if predicate(self.peek_next()) {
            self.advance();
        }
        c
    }

    /// If predicate from current char is true, executes body and then advances if next char also matches predicate
    ///
    /// At the end it returns the current char
    fn eat_while_do_from_current(
        &mut self,
        cond: impl Fn(char) -> bool,
        mut body: impl FnMut(char)
    ) -> char {
        while cond(self.current_char) && self.current_char != EOF_CHAR {
            body(self.current_char);
            if cond(self.peek_next()) {
                self.advance();
            } else {
                break;
            }
        }
        self.current_char
    }

    pub fn peek_next(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    pub fn peek_two_next(&self) -> char {
        let mut chars = self.chars.clone();
        chars.next();
        chars.next().unwrap_or(EOF_CHAR)
    }

    pub fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    pub fn is_tokenizing_string(&self) -> bool {
        self.str_layer > 0
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

    fn can_be_before_dot_float(char: char) -> bool {
        matches!(char, '+' | '-' | '*' | '/' | ' ')
    }
}

#[cfg(test)]
mod test {
    use token::TokenKind;
    use crate::Lexer;

    fn expect_tokens(src: &str, expected_tokens: &[TokenKind]) {
        let mut lexer = Lexer::new(src);
        for i in 0..expected_tokens.len() {
            assert_eq!(expected_tokens[i], lexer.scan_token().get_kind());
        }
        assert_eq!(TokenKind::Eof, lexer.scan_token().get_kind());
    }
}
