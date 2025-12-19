use crate::token::{Token, TokenType};

pub struct Lexer {
    src: String,
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(src: String) -> Self {
        let chars = src.chars().collect();
        Self { src, chars, pos: 0 }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            let tok = self.next_tok();
            if tok.kind == TokenType::EOF {
                tokens.push(tok);
                break;
            }
            tokens.push(tok);
        }

        tokens
    }

    fn next_tok(&mut self) -> Token {
        loop {
            self.skip_whitespace();

            let start = self.pos;

            let ch = match self.peek() {
                Some(ch) => ch,
                None => return self.make_token(TokenType::EOF, "EOF".into(), start, self.pos),
            };

            // match the big ones first
            if ch == '/' && self.peek_n(1) == Some('/') {
                self.next();
                self.next();
                self.skip_comment();
                continue;
            }

            if ch == '"' {
                return self.bag_string();
            }

            if ch.is_numeric() {
                return self.bag_number();
            }

            // is ascii alphabet instead of alphanum bc we dont want number first
            if ch.is_ascii_alphabetic() || ch == '_' {
                return self.bag_identifer_or_keyword();
            }

            let nc = self.peek_n(1);

            if let Some(kind) = self.try_two_char(ch) {
                self.next();
                self.next();
                return self.make_token(
                    kind,
                    [ch, nc.unwrap_or('\0')].iter().collect(),
                    start,
                    self.pos,
                );
            }

            if let Some(kind) = self.single_char_tok(ch) {
                self.next();
                return self.make_token(kind, ch.to_string(), start, self.pos);
            }
        }
    }

    fn try_two_char(&mut self, ch: char) -> Option<TokenType> {
        match (ch, self.peek_n(1)) {
            ('<', Some('=')) => Some(TokenType::LessEqual),
            ('>', Some('=')) => Some(TokenType::GreaterEqual),
            ('=', Some('=')) => Some(TokenType::EqualEqual),
            ('!', Some('=')) => Some(TokenType::NotEqual),
            ('&', Some('&')) => Some(TokenType::And),
            ('|', Some('|')) => Some(TokenType::Or),
            _ => None,
        }
    }

    fn single_char_tok(&mut self, ch: char) -> Option<TokenType> {
        match ch {
            '(' => Some(TokenType::LParen),
            ')' => Some(TokenType::RParen),
            '{' => Some(TokenType::LBrace),
            '}' => Some(TokenType::RBrace),
            ',' => Some(TokenType::Comma),
            ';' => Some(TokenType::Semicolon),
            ':' => Some(TokenType::Colon),
            '.' => Some(TokenType::Dot),
            '+' => Some(TokenType::Plus),
            '-' => Some(TokenType::Minus),
            '*' => Some(TokenType::Times),
            '/' => Some(TokenType::Divide),
            '%' => Some(TokenType::Modulo),
            '!' => Some(TokenType::Bang),
            '<' => Some(TokenType::Less),
            '>' => Some(TokenType::Greater),
            '=' => Some(TokenType::Assign),
            _ => None,
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.next();
            } else {
                break;
            };
        }
    }

    fn skip_comment(&mut self) {
        while let Some(ch) = self.next() {
            if ch == '\n' {
                break;
            };
        }
    }

    fn bag_identifer_or_keyword(&mut self) -> Token {
        let start = self.pos;

        self.next();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.next();
            } else {
                break;
            }
        }

        let ident = &self.src[start..self.pos];

        let kind = match ident {
            "let" => TokenType::Let,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "if" => TokenType::If,
            "elif" => TokenType::Elif,
            "else" => TokenType::Else,
            "for" => TokenType::For,
            "while" => TokenType::While,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "fn" => TokenType::Fn,
            "return" => TokenType::Return,
            _ => TokenType::Identifier,
        };

        self.make_token(kind, ident.to_string(), start, self.pos)
    }

    fn bag_string(&mut self) -> Token {
        self.next(); // consume first "
        let start = self.pos; // start of string "xxx"

        while let Some(ch) = self.peek() {
            if ch == '"' {
                break;
            }
            self.next();
        }

        let text: String = self.src[start..self.pos].to_string();
        self.next(); // consume last "
        self.make_token(TokenType::String, text, start, self.pos)
    }

    fn bag_number(&mut self) -> Token {
        let start = self.pos;
        let mut found_decimal = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.next();
            } else if ch == '.' && !found_decimal {
                found_decimal = true;
                self.next();
            } else {
                break;
            };
        }

        let text: String = self.src[start..self.pos].to_string();
        self.make_token(TokenType::Number, text, start, self.pos)
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek();
        self.pos += 1;
        ch
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_n(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).copied()
    }

    fn make_token(&self, kind: TokenType, lexeme: String, start: usize, end: usize) -> Token {
        Token {
            kind,
            span: start..end,
            lexeme: lexeme,
        }
    }
}
