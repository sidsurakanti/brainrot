use crate::env::Env;
use crate::lexer::Lexer;
use crate::parser::{Expr, Parser, Stmt};
use crate::token::TokenType;
use crate::value::Value;
use std::cmp::Ordering;

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

#[derive(Debug)]
pub enum ControlFlow {
    None,
    Continue,
    Break,
    Return(Value),
}

pub struct Interpreter {
    pub env: Env,
}

#[allow(dead_code)]
impl Interpreter {
    pub fn new() -> Self {
        Self { env: Env::new() }
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
        }

        Ok(())
    }

    fn eval_block(&mut self, stmts: &Vec<Stmt>) -> Result<ControlFlow, RuntimeError> {
        // TODO: handle in-loop v. out for return

        // enter scope
        self.env.push_scope();

        for stmt in stmts {
            let res = self.eval_stmt(stmt)?;
            // wlog, a block short circuits on non-none
            if !matches!(res, ControlFlow::None) {
                self.env.pop_scope();
                return Ok(res);
            }
        }

        // pop that smoke lmao
        self.env.pop_scope();
        Ok(ControlFlow::None)
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

            Stmt::Call(name, args) => self.eval_call(name, args),
            _ => Err(RuntimeError::Message(format!("unimplemented: {:?}", stmt))),
        }
    }

    fn eval_call(&mut self, name: &String, args: &Vec<Expr>) -> Result<ControlFlow, RuntimeError> {
        if name.eq("print") {
            for expr in args {
                let val = self.eval_expr(expr)?;
                println!("{:#?}", val)
            }
        }
        Ok(ControlFlow::None)
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
        self.env.push_scope();

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

        self.env.pop_scope(); // lol pop smoke
        Ok(ControlFlow::None)
    }

    fn eval_let(&mut self, name: &String, expr: &Expr) -> Result<ControlFlow, RuntimeError> {
        let val = self.eval_expr(expr)?;
        self.env.define(name.clone(), val);
        Ok(ControlFlow::None)
    }

    fn eval_assign(&mut self, name: &String, expr: &Expr) -> Result<ControlFlow, RuntimeError> {
        let val = self.eval_expr(expr)?;
        self.env.assign(name.clone(), val)?;
        Ok(ControlFlow::None)
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Binary(left, op, right) => {
                let l = self.eval_expr(left.as_ref())?;
                let r = self.eval_expr(right.as_ref())?;

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
                .env
                .get(name)
                .cloned()
                .ok_or_else(|| format!("undefined variable '{}'", name)),
            Expr::Group(e) => self.eval_expr(e.as_ref()),
        }
    }
}
