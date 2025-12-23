use crate::interpreter::Interpreter;
use colored::*;
use std::io::{self, Write};

pub struct Repl {}

impl Repl {
    pub fn repl() {
        let mut interp = Interpreter::new();

        let mut depth = 0;
        let mut saw_semicolon = true;
        let mut in_string = false;
        let mut saw_empty_line = false;
        let mut buff = String::new();

        println!("{}", "[BRAINROT] REPL\n<Ctrl-C> to quit.".purple().bold());
        // write to buff until ; or {}
        // don't bother increasing depth on {} found in strings
        loop {
            let mut line = String::new();

            print!(
                "{}",
                if depth == 0 && saw_semicolon {
                    ">>> ".purple()
                } else {
                    "... ".purple()
                }
            );
            io::stdout().flush().unwrap();
            if io::stdin().read_line(&mut line).unwrap() == 0 {
                break;
            }

            saw_semicolon = false;
            for c in line.chars() {
                match c {
                    '"' => {
                        in_string = !in_string;
                    }
                    '{' if !in_string => depth += 1,
                    '}' if !in_string => depth -= 1,
                    ';' if !in_string => saw_semicolon = true,
                    _ => {}
                }
            }

            // reset on two empty lines
            if line.trim().len() == 0 {
                if saw_empty_line {
                    depth = 0;
                    saw_semicolon = true;
                    in_string = false;
                    continue;
                }
                saw_empty_line = true;
            }

            buff.push_str(&line.trim());

            if depth == 0 && saw_semicolon {
                // eval buff
                // println!("{}", buff);
                let res = interp.run(buff.clone());
                if let Err(e) = res {
                    eprintln!("{:?}", e);
                }
                buff.clear();
                in_string = false;
            }
        }
    }
}
