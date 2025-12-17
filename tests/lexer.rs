use brainrot::lexer::Lexer;
use brainrot::token::TokenType;

fn kinds(src: &str) -> Vec<TokenType> {
    let mut l = Lexer::new(src.into());
    l.tokenize().into_iter().map(|t| t.kind).collect()
}

#[test]
fn lexes_numbers_and_identifiers() {
    let toks = kinds("let x = 123;");
    assert_eq!(
        toks,
        vec![
            TokenType::Let,
            TokenType::Identifier,
            TokenType::Assign,
            TokenType::Number,
            TokenType::Semicolon,
            TokenType::EOF,
        ]
    );
}

#[test]
fn lexes_string_literal() {
    let mut l = Lexer::new(r#""hello world""#.into());
    let toks = l.tokenize();

    assert_eq!(toks[0].kind, TokenType::String);
    assert_eq!(toks[0].lexeme, "hello world");
}

#[test]
fn distinguishes_keywords_from_identifiers() {
    let toks = kinds("let letter = true;");
    assert_eq!(
        toks,
        vec![
            TokenType::Let,
            TokenType::Identifier, // letter
            TokenType::Assign,
            TokenType::True,
            TokenType::Semicolon,
            TokenType::EOF,
        ]
    );
}

#[test]
fn lexes_two_char_operators() {
    let toks = kinds("a == b != c <= d >= e");
    assert_eq!(
        toks,
        vec![
            TokenType::Identifier,
            TokenType::EqualEqual,
            TokenType::Identifier,
            TokenType::NotEqual,
            TokenType::Identifier,
            TokenType::LessEqual,
            TokenType::Identifier,
            TokenType::GreaterEqual,
            TokenType::Identifier,
            TokenType::EOF,
        ]
    );
}

#[test]
fn lexes_logical_operators() {
    let toks = kinds("a && b || c");
    assert_eq!(
        toks,
        vec![
            TokenType::Identifier,
            TokenType::And,
            TokenType::Identifier,
            TokenType::Or,
            TokenType::Identifier,
            TokenType::EOF,
        ]
    );
}

#[test]
fn skips_comments() {
    let toks = kinds(
        r#"
        let x = 1; // comment
        let y = 2;
        "#,
    );

    assert_eq!(
        toks,
        vec![
            TokenType::Let,
            TokenType::Identifier,
            TokenType::Assign,
            TokenType::Number,
            TokenType::Semicolon,
            TokenType::Let,
            TokenType::Identifier,
            TokenType::Assign,
            TokenType::Number,
            TokenType::Semicolon,
            TokenType::EOF,
        ]
    );
}

#[test]
fn lexes_keywords() {
    let toks = kinds("fn while for break continue");
    assert_eq!(
        toks,
        vec![
            TokenType::Fn,
            TokenType::While,
            TokenType::For,
            TokenType::Break,
            TokenType::Continue,
            TokenType::EOF,
        ]
    );
}

#[test]
fn always_ends_with_eof() {
    let toks = kinds("");
    assert_eq!(toks, vec![TokenType::EOF]);
}
