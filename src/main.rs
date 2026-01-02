use std::fs;

use brainrot::interpreter::Interpreter;
use brainrot::lexer::Lexer;
use brainrot::parser::Parser;
use brainrot::repl::Repl;

#[allow(dead_code)]
fn main() {
    let src = fs::read_to_string("src/brain.rot").unwrap();

    let mut lxr = Lexer::new(src.clone());
    let tokens = lxr.tokenize();
    // dbg!(&tokens);

    let mut parser = Parser::new(tokens);
    let res = parser.parse();
    let Ok(_ast) = res else {
        eprintln!("parser error: {}", res.unwrap_err());
        return;
    };
    // dbg!(_ast);

    let mut _interp = Interpreter::new();
    // let res = _interp.run(src);
    // if let Err(e) = res {
    //     eprintln!("runtime error: {:?}", e);
    // }

    // _interp.env_dump();
    // if let Some(p) = _interp.get("x") {
    //     println!("{}", p);
    // }

    Repl::repl();
}
