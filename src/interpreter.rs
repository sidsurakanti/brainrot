use crate::lexer::Lexer;
use crate::parser::{Expr, Parser, Stmt};
use crate::token::TokenType;
use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Neg, Not, Sub};

#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Str(String),
    Bool(bool),
}

impl Add for Value {
    type Output = Result<Value, String>;

    fn add(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(a + b.as_str())),
            _ => Err("type mismatch".into()),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, String>;
    fn sub(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            _ => Err("type mismatch".into()),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value, String>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Int(a) => Ok(Value::Int(-a)),
            _ => Err("type mismatch".into()),
        }
    }
}

impl Not for Value {
    type Output = Result<Value, String>;

    fn not(self) -> Self::Output {
        match self {
            Value::Str(_) | Value::Int(_) => Ok(Value::Bool(false)),
            Value::Bool(val) => Ok(Value::Bool(!val)),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, String>;

    fn div(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
            _ => Err("type mismatch".into()),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, String>;

    fn mul(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Str(s), Value::Int(n)) | (Value::Int(n), Value::Str(s)) => {
                if n < 0 {
                    return Err("cannot multiply string by a negative integer".into());
                }

                Ok(Value::Str(s.repeat(n as usize)))
            }
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            _ => Err("type mismatch".into()),
        }
    }
}

pub struct Interpreter {
    pub env: HashMap<String, Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
        }
    }

    pub fn run(&mut self, src: String) {
        let tokens = Lexer::new(src).tokenize();
        let res = Parser::new(tokens).parse();

        let Ok(ast) = res else {
            let e = res.err().unwrap();
            return eprintln!("parser error: {:?}", e);
        };

        self.eval(ast);
    }

    fn eval(&mut self, ast: Vec<Stmt>) {
        for stmt in ast {
            match stmt {
                Stmt::Block(stmts) => self.eval(stmts),
                Stmt::Let(name, expr) => self.eval_let(name, expr),
                Stmt::Expr(expr) => {
                    self.eval_expr(expr).unwrap();
                }
                // Stmt::While(cond, block) => self.eval_while(cond, block),
                // Stmt::For(init, cond, step, block) => self.eval_for(init, cond, step, block),
                // Stmt::Call(name, args) => self.eval_call(name, args),
                _ => panic!("unimplemented"),
            }
        }
    }

    fn eval_let(&mut self, name: String, expr: Expr) {
        let val = self.eval_expr(expr).unwrap();
        self.env.insert(name, val);
    }

    fn eval_expr(&mut self, expr: Expr) -> Result<Value, String> {
        match expr {
            Expr::Binary(left, op, right) => {
                let l = self.eval_expr(*left)?;
                let r = self.eval_expr(*right)?;

                // TODO: these will be infix op's but js make sure or smth idk
                match op {
                    TokenType::Plus => l + r,
                    TokenType::Minus => l - r,
                    TokenType::Times => l * r,
                    TokenType::Divide => l / r,
                    _ => return Err("unimplemented".into()),
                }
            }
            Expr::Unary(op, right) => {
                let val = self.eval_expr(*right)?;

                match op {
                    TokenType::Minus => -val,
                    TokenType::Bang => !val,
                    _ => return Err("unimplemented".into()),
                }
            }
            Expr::Number(val) => Ok(Value::Int(val)),
            Expr::String(val) => Ok(Value::Str(val)),
            Expr::Ident(name) => self
                .env
                .get(&name)
                .cloned()
                .ok_or_else(|| format!("undefined variable '{}'", name)),
            Expr::Group(e) => return self.eval_expr(*e),
        }
    }
}
