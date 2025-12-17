use crate::env::Env;
use crate::lexer::Lexer;
use crate::parser::{Expr, Parser, Stmt};
use crate::token::TokenType;
use crate::value::Value;
use std::cmp::Ordering;

#[derive(Debug)]
pub enum ControlFlow {
    None,
    Continue,
    Break,
    Return(Value),
}

pub struct Interpreter {
    // pub env: HashMap<String, Value>,
    pub env: Env,
}

#[allow(dead_code)]
impl Interpreter {
    pub fn new() -> Self {
        Self {
            // env: HashMap::new(),
            env: Env::new(),
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
                // TODO: handle in-loop v. out for return

                // enter scope
                self.env.push_scope();

                for stmt in stmts {
                    let res = self.eval_stmt(stmt);
                    // wlog, a block short circuits on non-none
                    if !matches!(res, ControlFlow::None) {
                        self.env.pop_scope();
                        return res;
                    }
                }

                // pop that smoke lmao
                self.env.pop_scope();
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

            Stmt::For {
                init,
                cond,
                step,
                block,
            } => {
                self.eval_for(init, cond, step, block);
                ControlFlow::None
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    let ret = self.eval_expr(e).unwrap();
                    ControlFlow::Return(ret)
                } else {
                    ControlFlow::Return(Value::Null)
                }
            }
            Stmt::Continue => ControlFlow::Continue,
            Stmt::Break => ControlFlow::Break,
            Stmt::Assignment(name, expr) => self.eval_assign(name, expr),
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => self.eval_if(cond, then_branch, else_branch),

            // Stmt::Call(name, args) => self.eval_call(name, args),
            _ => panic!("unimplemented: {:?}", stmt),
        }
    }

    // ifStmt -> "if" "(" expr ")" block ("elif" block)* ("else" block)?
    fn eval_if(
        &mut self,
        cond: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    ) -> ControlFlow {
        let block = then_branch; // Stmt::Block

        if self.eval_expr(cond.clone()).unwrap().is_truthy() {
            self.eval_stmt(*block.clone())
        } else {
            // else_block is of type Stmt::If or None
            if let Some(else_block) = else_branch {
                self.eval_stmt(*else_block)
            } else {
                ControlFlow::None
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

    fn eval_for(&mut self, init: Box<Stmt>, cond: Expr, step: Box<Stmt>, block: Box<Stmt>) {
        // enter for scope
        self.env.push_scope();

        // adds init to env
        match *init {
            Stmt::Let(name, expr) => {
                self.eval_let(name, expr);
            }
            // precondition expr == Expr::Null
            Stmt::Expr(_) => {}
            _ => {
                panic!("unexpected stmt inside for loop: should be init")
            }
        };

        while self.eval_expr(cond.clone()).unwrap().is_truthy() {
            match self.eval_stmt(*block.clone()) {
                ControlFlow::Continue => {
                    // otherwise we will hit same condition again wo increment
                    self.eval_stmt(*step.clone());
                    continue;
                }
                ControlFlow::Break => break,
                ControlFlow::None => {}
                ControlFlow::Return(_) => panic!("unexpected return inside loop"),
            }
            self.eval_stmt(*step.clone());
        }

        self.env.pop_scope(); // lol pop smoke
    }

    fn eval_let(&mut self, name: String, expr: Expr) -> ControlFlow {
        let val = self.eval_expr(expr).unwrap();
        self.env.define(name, val);
        ControlFlow::None
    }

    fn eval_assign(&mut self, name: String, expr: Expr) -> ControlFlow {
        let val = self.eval_expr(expr).unwrap();
        self.env.assign(name, val).unwrap();
        ControlFlow::None
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
            Expr::Null => Ok(Value::Null),
            Expr::Ident(name) => self
                .env
                .get(&name)
                .cloned()
                .ok_or_else(|| format!("undefined variable '{}'", name)),
            Expr::Group(e) => return self.eval_expr(*e),
        }
    }
}
