use std::fs;

use brainrot::interpreter::Interpreter;
use brainrot::lexer::Lexer;
use brainrot::parser::{Parser, Stmt};

#[allow(dead_code)]
fn main() {
    let src = fs::read_to_string("src/brain.rot").unwrap();

    let mut lxr = Lexer::new(src.clone());
    let tokens = lxr.tokenize();
    // println!("{:#?}", tokens);

    let mut parser = Parser::new(tokens);
    let _ast: Vec<Stmt> = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("parser error: {}", e);
            return;
        }
    };
    // println!("{:#?}", _ast);

    let mut interpreter = Interpreter::new();
    interpreter.run(src);

    for scope in &interpreter.env.scopes {
        for (k, v) in scope {
            println!("{}: {:?}", k, v);
        }
    }
}
