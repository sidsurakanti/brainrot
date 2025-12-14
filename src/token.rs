use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TokenType {
    // literals
    Number,
    Identifier,
    String,

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
    Elif,
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
    pub lexeme: String,
}
