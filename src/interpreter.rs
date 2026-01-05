use crate::env::Env;
use crate::lexer::Lexer;
use crate::parser::{Expr, Parser, Stmt};
use crate::token::TokenType;
use crate::value::Value;
use colored::*;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub enum RuntimeError {
    Message(String),
}

#[derive(Debug)]
pub enum LangError {
    Parse(String),
    Runtime(RuntimeError),
}

impl From<String> for RuntimeError {
    fn from(s: String) -> Self {
        RuntimeError::Message(s)
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::Message(msg) => write!(f, "{msg}"),
        }
    }
}

#[derive(Debug)]
pub enum ControlFlow {
    None,
    Continue,
    Break,
    Return(Value),
}

type EnvRef = Rc<RefCell<Env>>;

pub struct Interpreter {
    pub curr_env: EnvRef,
    pub global: EnvRef,
}

#[allow(dead_code, unused_variables)]
impl Interpreter {
    pub fn new() -> Self {
        let g = Rc::new(RefCell::new(Env::new(None)));

        Self {
            curr_env: Rc::clone(&g),
            global: Rc::clone(&g),
        }
    }

    pub fn run(&mut self, src: String) -> Result<(), LangError> {
        let tokens = Lexer::new(src).tokenize();
        let ast = Parser::new(tokens).parse().map_err(LangError::Parse)?;

        self.eval(ast).map_err(LangError::Runtime)?;

        Ok(())
    }

    fn eval(&mut self, ast: Vec<Stmt>) -> Result<(), RuntimeError> {
        for stmt in ast {
            // stmt only valid iff yielding ControlFLow::None at top level
            match self.eval_stmt(&stmt)? {
                ControlFlow::None => {}
                cf => {
                    return Err(RuntimeError::Message(format!(
                        "invalid control flow at top level {:?} in statement {:#?}",
                        cf, stmt,
                    )));
                }
            }

            // self.env_dump();
        }

        Ok(())
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        match stmt {
            Stmt::Block(stmts) => self.eval_block(stmts),

            Stmt::Expr(expr) => {
                self.eval_expr(expr)?;
                Ok(ControlFlow::None)
            }

            Stmt::Let(name, expr) => self.eval_let(name, expr),
            Stmt::Assignment(name, expr) => self.eval_assign(name, expr),

            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => self.eval_if(cond, then_branch, else_branch),

            Stmt::While(cond, block) => self.eval_while(cond, block),
            Stmt::For {
                init,
                cond,
                step,
                block,
            } => self.eval_for(init, cond, step, block),

            Stmt::Continue => Ok(ControlFlow::Continue),
            Stmt::Break => Ok(ControlFlow::Break),
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    let ret = self.eval_expr(e)?;
                    Ok(ControlFlow::Return(ret))
                } else {
                    Ok(ControlFlow::Return(Value::Null))
                }
            }

            Stmt::Fn { name, args, block } => {
                // add func to env values
                let func = Value::Fn {
                    name: name.clone(),
                    args: args.clone(),
                    body: block.clone(),
                    captured_env: Rc::clone(&self.curr_env),
                };

                let mut defining_env = self.curr_env.borrow_mut();
                defining_env.bucket.insert(name.clone(), func);

                Ok(ControlFlow::None)
            }

            Stmt::Call(name, args) => self.eval_call(name, args),
            // _ => Err(RuntimeError::Message(format!("unimplemented: {:?}", stmt))),
        }
    }

    fn eval_block(&mut self, stmts: &Vec<Stmt>) -> Result<ControlFlow, RuntimeError> {
        // enter scope
        let prev = Rc::clone(&self.curr_env);
        self.curr_env = Env::push_scope(&self.curr_env);

        for stmt in stmts {
            let res = self.eval_stmt(stmt)?;
            // wlog, a block short circuits on non-none
            if !matches!(res, ControlFlow::None) {
                self.curr_env = prev;
                return Ok(res);
            }
        }

        // pop that smoke lmao
        self.curr_env = prev; // temp env gets dropped rc--
        Ok(ControlFlow::None)
    }

    fn eval_call(&mut self, name: &String, args: &Vec<Expr>) -> Result<ControlFlow, RuntimeError> {
        let vals: Vec<Value> = args
            .iter()
            .map(|arg| self.eval_expr(arg))
            .collect::<Result<Vec<_>, _>>()?;

        // TODO: builtins
        if name.eq("print") {
            for val in vals.clone() {
                match val {
                    Value::Int(_) => println!("{}", val.to_string().cyan()),
                    Value::Str(_) => println!("{}", val.to_string().green()),
                    Value::Bool(_) => println!("{}", val.to_string().yellow()),
                    Value::Null => println!("{}", "null".dimmed()),
                    Value::Fn { .. } => println!("{}", val),
                }
            }
            return Ok(ControlFlow::None);
        }

        let func = self
            .get(name)
            .ok_or(format!("function not defined: {}", name))?;

        match func {
            Value::Fn {
                name,
                args,
                body,
                captured_env,
            } => {
                // attach worker env for fn to it's local env
                let calling_env = Rc::clone(&self.curr_env);
                // worker -> captured -> parents
                self.curr_env = Env::push_scope(&captured_env);

                // push args to working env
                if !(vals.len() == args.len()) {
                    return Err(RuntimeError::Message(format!(
                        "expected {} args got {}",
                        args.len(),
                        vals.len()
                    )));
                }

                {
                    let mut working_env = self.curr_env.borrow_mut();
                    for (arg, val) in args.iter().zip(&vals) {
                        working_env.bucket.insert(arg.clone(), val.clone());
                    }
                }

                // eval body
                let res = self.eval_stmt(body.as_ref());
                // reset back to calling env
                self.curr_env = calling_env;

                match res {
                    Ok(cf) => return Ok(cf),
                    Err(e) => return Err(e),
                }
            }
            _ => panic!(), // should never get here
        }

        // Ok(ControlFlow::None)
    }

    // ifStmt -> "if" "(" expr ")" block ("elif" block)* ("else" block)?
    fn eval_if(
        &mut self,
        cond: &Expr,
        then_branch: &Box<Stmt>,
        else_branch: &Option<Box<Stmt>>,
    ) -> Result<ControlFlow, RuntimeError> {
        if self.eval_expr(cond)?.is_truthy() {
            self.eval_stmt(then_branch.as_ref())
        } else {
            // else_block is of type Stmt::If or None
            if let Some(else_block) = else_branch {
                self.eval_stmt(else_block.as_ref())
            } else {
                Ok(ControlFlow::None)
            }
        }
    }

    fn eval_while(&mut self, cond: &Expr, block: &Box<Stmt>) -> Result<ControlFlow, RuntimeError> {
        'outer: while self.eval_expr(cond)?.is_truthy() {
            match self.eval_stmt(block.as_ref())? {
                ControlFlow::Continue => continue 'outer,
                ControlFlow::Break => break 'outer,
                ControlFlow::None => {}
                ControlFlow::Return(val) => return Ok(ControlFlow::Return(val)),
            }
        }

        Ok(ControlFlow::None)
    }

    fn eval_for(
        &mut self,
        init: &Box<Stmt>,
        cond: &Expr,
        step: &Box<Stmt>,
        block: &Box<Stmt>,
    ) -> Result<ControlFlow, RuntimeError> {
        // enter for scope
        let prev = Rc::clone(&self.curr_env);
        self.curr_env = Env::push_scope(&self.curr_env);

        // adds init to env
        match init.as_ref() {
            Stmt::Let(name, expr) => {
                self.eval_let(name, expr)?;
            }
            // precondition expr == Expr::Null
            Stmt::Expr(_) => {}
            _ => {
                return Err(RuntimeError::Message(
                    "unexpected stmt inside for loop: should be init".into(),
                ));
            }
        };

        while self.eval_expr(cond)?.is_truthy() {
            match self.eval_stmt(block.as_ref())? {
                ControlFlow::Continue => {
                    // otherwise we will hit same condition again wo increment
                    self.eval_stmt(step.as_ref())?;
                    continue;
                }
                ControlFlow::Break => break,
                ControlFlow::None => {}
                ControlFlow::Return(val) => return Ok(ControlFlow::Return(val)),
            }

            // NOTE: what if step returns !CF::None
            self.eval_stmt(step.as_ref())?;
        }

        self.curr_env = prev;
        Ok(ControlFlow::None)
    }

    fn eval_let(&mut self, name: &String, expr: &Expr) -> Result<ControlFlow, RuntimeError> {
        let val = self.eval_expr(expr)?;
        // println!("{}: {}", name, val);
        Env::define(Rc::clone(&self.curr_env), name.clone(), val);
        Ok(ControlFlow::None)
    }

    fn eval_assign(&mut self, name: &String, expr: &Expr) -> Result<ControlFlow, RuntimeError> {
        let val = self.eval_expr(expr)?;
        Env::assign(Rc::clone(&self.curr_env), name.clone(), val)?;
        Ok(ControlFlow::None)
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Binary(left, op, right) => {
                let l = self.eval_expr(left.as_ref())?;
                let r = self.eval_expr(right.as_ref())?;

                match op {
                    TokenType::Plus => l + r,
                    TokenType::Minus => l - r,
                    TokenType::Times => l * r,
                    TokenType::Divide => l / r,
                    TokenType::Modulo => l % r,
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
                let val = self.eval_expr(right.as_ref())?;

                match op {
                    TokenType::Minus => -val,
                    TokenType::Bang => !val,
                    _ => return Err("unimplemented".into()),
                }
            }
            Expr::Number(val) => Ok(Value::Int(val.clone())),
            Expr::String(val) => Ok(Value::Str(val.clone())),
            Expr::Bool(b) => Ok(Value::Bool(b.clone())),
            Expr::Null => Ok(Value::Null),
            Expr::Ident(name) => self
                .get(name)
                .ok_or_else(|| format!("undefined variable '{}'", name)),
            Expr::Group(e) => self.eval_expr(e.as_ref()),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        Env::get(Rc::clone(&self.curr_env), name)
    }

    pub fn env_dump(&self) {
        let mut curr = Rc::clone(&self.curr_env);

        loop {
            let parent = {
                let env = curr.borrow();
                dbg!(env.bucket.keys());
                env.parent.clone()
            };

            match parent {
                Some(p) => curr = p,
                None => return,
            }
        }
    }
}
