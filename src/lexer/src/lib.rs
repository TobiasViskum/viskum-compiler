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
            '(' => self.make_token(TokenKind::LeftParen),
            ')' => self.make_token(TokenKind::RightParen),
            ':' => self.make_token_or_other_if(TokenKind::Colon, '=', TokenKind::Define),
            '=' => self.make_token_or_other_if(TokenKind::Assign, '=', TokenKind::Eq),
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

    fn newline(&mut self) -> Token {
        self.line += 1;
        self.skip_char_and_scan()
    }

    fn make_ident_or_keyword(&mut self) -> Token {
        let mut buffer = String::with_capacity(64);

        self.eat_while_do_from_current(
            |c| Self::is_alphabetic(c),
            |c| { buffer.push(c) }
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
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "elif" => Some(TokenKind::Elif),
            "break" => Some(TokenKind::Break),
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
        while closure(self.peek_next()) && !self.is_eof() {
            self.advance();
        }
        self.peek_next()
    }

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

    pub fn is_eof(&self) -> bool {
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

#[cfg(test)]
mod test {
    use token::TokenKind;
    use crate::Lexer;

    #[test]
    fn tokenize_1() {
        let input = "1 + 2 * if 2 == 2 do 8 + 9 end";
        let expected_tokens = [
            TokenKind::Integer,
            TokenKind::Plus,
            TokenKind::Integer,
            TokenKind::Star,
            TokenKind::If,
            TokenKind::Integer,
            TokenKind::Eq,
            TokenKind::Integer,
            TokenKind::Do,
            TokenKind::Integer,
            TokenKind::Plus,
            TokenKind::Integer,
            TokenKind::End,
        ];

        expect_tokens(input, &expected_tokens)
    }

    #[test]
    fn tokenize_2() {
        let input =
            "
            def main()
                a := 2
                b := (8 * 2)
                loop
                    break
                end
            end
        ";
        let expected_tokens = [
            TokenKind::Def,
            TokenKind::Ident,
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::Ident,
            TokenKind::Define,
            TokenKind::Integer,
            TokenKind::Ident,
            TokenKind::Define,
            TokenKind::LeftParen,
            TokenKind::Integer,
            TokenKind::Star,
            TokenKind::Integer,
            TokenKind::RightParen,
            TokenKind::Loop,
            TokenKind::Break,
            TokenKind::End,
            TokenKind::End,
        ];

        expect_tokens(input, &expected_tokens)
    }

    fn expect_tokens(src: &str, expected_tokens: &[TokenKind]) {
        let mut lexer = Lexer::new(src);
        for i in 0..expected_tokens.len() {
            assert_eq!(expected_tokens[i], lexer.scan_token().get_kind());
        }
        assert_eq!(TokenKind::Eof, lexer.scan_token().get_kind());
    }
}
