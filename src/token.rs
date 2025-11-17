use std::ops::Range;

#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
pub struct Token {
    pub kind: TokenType,
    pub span: Range<usize>,
}
