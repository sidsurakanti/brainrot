use crate::lexer::Lexer;
use crate::parser::{Expr, Parser, Stmt};
use crate::token::TokenType;
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Neg, Not, Sub};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Value {
    Int(i32),
    Str(String),
    Bool(bool),
    Void,
}

#[derive(Debug)]
pub enum ControlFlow {
    None,
    Continue,
    Break,
    Return(Value),
}

pub struct Interpreter {
    pub env: HashMap<String, Value>,
}

#[allow(dead_code)]
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
            // stmt only valid iff yielding ControlFLow::None at top level
            let cf = self.eval_stmt(stmt.clone());
            if !matches!(cf, ControlFlow::None) {
                panic!(
                    "invalid control flow at top level {:?} in statement {:#?}",
                    cf,
                    stmt.clone()
                )
            }
        }
    }

    fn eval_stmt(&mut self, stmt: Stmt) -> ControlFlow {
        match stmt {
            Stmt::Block(stmts) => {
                // TODO: handle in-loop v. out
                for stmt in stmts {
                    let res = self.eval_stmt(stmt);
                    match res {
                        // wlog, a block short circuits on non-none
                        ControlFlow::Continue => return ControlFlow::Continue,
                        ControlFlow::Break => return ControlFlow::Break,
                        ControlFlow::Return(v) => return ControlFlow::Return(v),
                        ControlFlow::None => {}
                    }
                }

                ControlFlow::None
            }
            Stmt::Let(name, expr) => {
                self.eval_let(name, expr);
                ControlFlow::None
            }
            Stmt::Expr(expr) => {
                self.eval_expr(expr).unwrap();
                ControlFlow::None
            }
            Stmt::While(cond, block) => {
                self.eval_while(cond, block);
                ControlFlow::None
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    let ret = self.eval_expr(e).unwrap();
                    ControlFlow::Return(ret)
                } else {
                    ControlFlow::Return(Value::Void)
                }
            }
            Stmt::Continue => ControlFlow::Continue,
            Stmt::Break => ControlFlow::Break,
            Stmt::Assignment(name, expr) => {
                // make sure var is already defined
                if self.env.contains_key(&name) {
                    self.eval_let(name, expr);
                } else {
                    // TODO: better errors
                    panic!("cannot reassign uninitialized variable '{:?}'", name)
                }
                ControlFlow::None
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.eval_if(cond, then_branch, else_branch);
                ControlFlow::None
            }

            // Stmt::For(init, cond, step, block) => self.eval_for(init, cond, step, block),
            // Stmt::Call(name, args) => self.eval_call(name, args),
            _ => panic!("unimplemented: {:?}", stmt),
        }
    }

    // ifStmt -> "if" "(" expr ")" block ("elif" block)* ("else" block)?
    fn eval_if(&mut self, cond: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>>) {
        let block = *then_branch.clone();
        let stmts = match block {
            Stmt::Block(v) => v,
            _ => panic!("expected to unwrap block after while condition"),
        };

        if self.eval_expr(cond.clone()).unwrap().is_truthy() {
            for stmt in stmts.clone() {
                self.eval_stmt(stmt);
            }
        } else {
            if let Some(else_block) = else_branch {
                // else_block is of type Stmt::If or None
                self.eval_stmt(*else_block);
            }
        }
    }

    fn eval_while(&mut self, cond: Expr, block: Box<Stmt>) {
        'outer: while self.eval_expr(cond.clone()).unwrap().is_truthy() {
            match self.eval_stmt(*block.clone()) {
                ControlFlow::Continue => continue 'outer,
                ControlFlow::Break => break 'outer,
                ControlFlow::None => {}
                ControlFlow::Return(_) => panic!("unexpected return inside loop"),
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
                    TokenType::Less => {
                        let res = l.partial_cmp(&r);
                        let b = matches!(res, Some(Ordering::Less));
                        Ok(Value::Bool(b))
                    }
                    TokenType::Greater => {
                        let res = l.partial_cmp(&r);
                        let b = matches!(res, Some(Ordering::Greater));
                        Ok(Value::Bool(b))
                    }
                    TokenType::LessEqual => {
                        let res = l.partial_cmp(&r);
                        let b = matches!(res, Some(Ordering::Equal) | Some(Ordering::Less));
                        Ok(Value::Bool(b))
                    }
                    TokenType::GreaterEqual => {
                        let res = l.partial_cmp(&r);
                        let b = matches!(res, Some(Ordering::Equal) | Some(Ordering::Greater));
                        Ok(Value::Bool(b))
                    }
                    TokenType::EqualEqual => Ok(Value::Bool(l == r)),
                    TokenType::NotEqual => Ok(Value::Bool(l != r)),
                    _ => return Err(format!("unimplemented {:?}", op)),
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
            Expr::Bool(b) => Ok(Value::Bool(b)),
            Expr::Null => Ok(Value::Void),
            Expr::Ident(name) => self
                .env
                .get(&name)
                .cloned()
                .ok_or_else(|| format!("undefined variable '{}'", name)),
            Expr::Group(e) => return self.eval_expr(*e),
        }
    }
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
            Value::Void => Err("cannot evaluate not for void".into()),
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

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            (Value::Str(a), Value::Str(b)) => a.partial_cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Int(n) => !(*n == 0),
            Value::Bool(b) => *b,
            Value::Str(s) => !s.is_empty(),
            Value::Void => false,
        }
    }
}
