use crate::token::Token;

// lexer.tokenize() -> Vec<Token> ->
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}
