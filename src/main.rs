use std::fs;

use brainrot::interpreter::Interpreter;
use brainrot::lexer::Lexer;
use brainrot::parser::Parser;

#[allow(dead_code)]
fn main() {
    let src = fs::read_to_string("src/brain.rot").unwrap();

    let mut lxr = Lexer::new(src.clone());
    let tokens = lxr.tokenize();
    // println!("{:#?}", tokens);

    let mut parser = Parser::new(tokens);
    let res = parser.parse();
    let Ok(_ast) = res else {
        eprintln!("parser error: {}", res.unwrap_err());
        return;
    };
    // println!("{:#?}", _ast);

    let mut interpreter = Interpreter::new();
    let res = interpreter.run(src);
    if let Err(e) = res {
        eprintln!("runtime error: {:?}", e);
    }

    for scope in &interpreter.env.scopes {
        for (k, v) in scope {
            println!("{}: {:?}", k, v);
        }
    }
}
