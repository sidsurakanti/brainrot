mod interpreter;
mod lexer;
mod parser;
mod token;

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::{Parser, Stmt};
use std::fs;

fn main() {
    let src = fs::read_to_string("src/brain.rot").unwrap();

    let mut lxr = Lexer::new(src.clone());
    let tokens = lxr.tokenize();
    println!("{:#?}", tokens);

    let mut parser = Parser::new(tokens);
    let ast: Vec<Stmt> = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("parser error: {}", e);
            return;
        }
    };

    println!("{:#?}", ast);

    let mut interpreter = Interpreter::new();

    interpreter.run(src);
    interpreter.env.iter().for_each(|(k, v)| {
        println!("{}: {:?}", k, v);
    });
}
