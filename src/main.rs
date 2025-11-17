mod lexer;
mod token;

use crate::lexer::Lexer;
use std::fs;

fn main() {
    let src = fs::read_to_string("src/brain.rot").unwrap();
    let mut lxr = Lexer::new(src);
    let tokens = lxr.tokenize();
    println!("{:#?}", tokens);
}
