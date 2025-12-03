use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TokenType {
    // literals
    Number(i32),
    Identifier(String),
    String(String),

    // operators
    Assign,

    // arithmetic
    Plus,
    Minus,
    Times,
    Divide,
    Modulo,

    // punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    Colon,
    Dot,

    // comparisons
    Bang,
    Less,
    Greater,
    EqualEqual,
    NotEqual,
    LessEqual,
    GreaterEqual,

    // keywords
    Let,
    True,
    False,
    And,
    Or,
    If,
    Else,
    For,
    While,
    Break,
    Continue,
    Fn,
    Return,

    EOF,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenType,
    pub span: Range<usize>,
}

impl Token {
    pub fn is_op(&self) -> bool {
        match self.kind {
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Times
            | TokenType::Divide
            | TokenType::Modulo
            | TokenType::Assign
            | TokenType::Bang
            | TokenType::Less
            | TokenType::Greater
            | TokenType::EqualEqual
            | TokenType::NotEqual
            | TokenType::LessEqual
            | TokenType::GreaterEqual => true,
            _ => false,
        }
    }
}
