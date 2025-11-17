use crate::token::{Token, TokenType};

pub struct Lexer {
    src: String,
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(src: String) -> Lexer {
        let chars = src.chars().collect();
        Lexer { src, chars, pos: 0 }
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
                None => return self.make_token(TokenType::EOF, start, self.pos),
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

            // match single chars now
            match ch {
                '(' => {
                    self.next();
                    return self.make_token(TokenType::LParen, start, self.pos);
                }
                ')' => {
                    self.next();
                    return self.make_token(TokenType::RParen, start, self.pos);
                }
                '{' => {
                    self.next();
                    return self.make_token(TokenType::LBrace, start, self.pos);
                }
                '}' => {
                    self.next();
                    return self.make_token(TokenType::RBrace, start, self.pos);
                }
                ',' => {
                    self.next();
                    return self.make_token(TokenType::Comma, start, self.pos);
                }
                ';' => {
                    self.next();
                    return self.make_token(TokenType::Semicolon, start, self.pos);
                }
                ':' => {
                    self.next();
                    return self.make_token(TokenType::Colon, start, self.pos);
                }
                '.' => {
                    self.next();
                    return self.make_token(TokenType::Dot, start, self.pos);
                }
                '+' => {
                    self.next();
                    return self.make_token(TokenType::Plus, start, self.pos);
                }
                '-' => {
                    self.next();
                    return self.make_token(TokenType::Minus, start, self.pos);
                }
                '*' => {
                    self.next();
                    return self.make_token(TokenType::Times, start, self.pos);
                }
                '/' => {
                    self.next();
                    return self.make_token(TokenType::Divide, start, self.pos);
                }
                '%' => {
                    self.next();
                    return self.make_token(TokenType::Modulo, start, self.pos);
                }
                '!' => {
                    self.next();
                    return self.make_token(TokenType::Bang, start, self.pos);
                }
                _ => {}
            }

            match (ch, self.peek_n(1)) {
                ('=', Some('=')) => {
                    self.next();
                    self.next();
                    return self.make_token(TokenType::EqualEqual, start, self.pos);
                }
                ('!', Some('=')) => {
                    self.next();
                    self.next();
                    return self.make_token(TokenType::NotEqual, start, self.pos);
                }
                ('<', Some('=')) => {
                    self.next();
                    self.next();
                    return self.make_token(TokenType::LessEqual, start, self.pos);
                }
                ('>', Some('=')) => {
                    self.next();
                    self.next();
                    return self.make_token(TokenType::GreaterEqual, start, self.pos);
                }
                ('<', _) => {
                    self.next();
                    return self.make_token(TokenType::Less, start, self.pos);
                }
                ('>', _) => {
                    self.next();
                    return self.make_token(TokenType::Greater, start, self.pos);
                }
                ('&', Some('&')) => {
                    self.next();
                    self.next();
                    return self.make_token(TokenType::And, start, self.pos);
                }
                ('|', Some('|')) => {
                    self.next();
                    self.next();
                    return self.make_token(TokenType::Or, start, self.pos);
                }
                ('=', _) => {
                    self.next();
                    return self.make_token(TokenType::Assign, start, self.pos);
                }
                _ => {}
            }
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
            "else" => TokenType::Else,
            "for" => TokenType::For,
            "while" => TokenType::While,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "fn" => TokenType::Fn,
            "return" => TokenType::Return,
            _ => TokenType::Identifier(ident.to_string()),
        };

        self.make_token(kind, start, self.pos)
    }

    fn bag_string(&mut self) -> Token {
        let start = self.pos; // start of string "xxx"
        self.next(); // consume first "

        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.next(); // consume last "
                break;
            }

            self.next();
        }

        let text: String = self.src[start..self.pos].to_string();
        self.make_token(TokenType::String(text), start, self.pos)
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

        let text = &self.src[start..self.pos];
        let item: i32 = text.parse().unwrap();
        self.make_token(TokenType::Number(item), start, self.pos)
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

    fn make_token(&self, kind: TokenType, start: usize, end: usize) -> Token {
        Token {
            kind,
            span: start..end,
        }
    }
}
