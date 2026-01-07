use crate::token::{Token, TokenType};
use maplit::hashmap;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Let(String, Expr),
    While(Expr, Box<Stmt>),
    For {
        init: Box<Stmt>, // let
        cond: Expr,
        step: Box<Stmt>,
        block: Box<Stmt>,
    },
    Fn {
        name: String,
        args: Vec<String>,
        block: Box<Stmt>,
    },
    If {
        cond: Expr,
        then_branch: Box<Stmt>, // Stmt::Block
        else_branch: Option<Box<Stmt>>,
    },
    Expr(Expr),
    Assignment(String, Expr),
    Return(Option<Expr>),
    Continue,
    Break,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Expr {
    Binary(Box<Expr>, TokenType, Box<Expr>),
    Unary(TokenType, Box<Expr>),
    Number(i32),
    Ident(String),
    String(String),
    Group(Box<Expr>),
    Bool(bool),
    Null,
    Callable { name: Box<Expr>, args: Vec<Expr> },
}

// lexer.tokenize() -> Vec<Token> ->
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    lookup: HashMap<TokenType, (usize, usize)>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            // pratt power
            lookup: hashmap! {
                TokenType::Plus => (10, 11),
                TokenType::Minus => (10, 11),
                TokenType::Times => (20, 21),
                TokenType::Divide => (20, 21),
                TokenType::Modulo => (20, 21),

                TokenType::Assign => (1, 2),
                TokenType::Bang => (40, 41),

                TokenType::Less => (6, 7),
                TokenType::Greater => (6, 7),
                TokenType::EqualEqual => (5, 6),
                TokenType::NotEqual => (5, 6),
                TokenType::LessEqual => (6, 7),
                TokenType::GreaterEqual => (6, 7),

                TokenType::Or => (3, 4),
                TokenType::And => (4, 5),

            },
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = vec![];

        while self.pos < self.tokens.len() && self.peek().kind != TokenType::EOF {
            let statement = self.parse_statement()?;
            statements.push(statement);
        }

        Ok(statements)
    }

    // NOTE:
    // we know a program essentially only contains statements + an EOF
    // so, all we have to do is parse all the statements in the prog
    pub fn parse_statement(&mut self) -> Result<Stmt, String> {
        let token = self.peek();

        let statement: Stmt = match token.kind {
            TokenType::LBrace => self.parse_block()?,

            TokenType::Let => self.parse_let()?,

            TokenType::While => self.parse_while()?,

            TokenType::For => self.parse_for()?,

            TokenType::If => self.parse_if()?,

            TokenType::Fn => self.parse_fn()?,

            TokenType::Identifier => {
                match self.at(1).kind {
                    // assignStmt -> IDENT "=" expr
                    TokenType::Assign
                    | TokenType::PlusEqual
                    | TokenType::MinusEqual
                    | TokenType::TimesEqual
                    | TokenType::DivEqual
                    | TokenType::ModuloEqual => {
                        let ret = self.parse_assign()?;
                        self.expect(TokenType::Semicolon, "expected ';' after assignment")?;
                        ret
                    }
                    // callStmt -> IDENT "(" params? ")"
                    // TokenType::LParen => {
                    //     let name = token.lexeme.clone();
                    //     self.next(); // consume ident
                    //
                    //     let args: Vec<Expr> = self.parse_args()?;
                    //     self.expect(TokenType::Semicolon, "expected ';' after expression")?;
                    //     Stmt::Call(name, args)
                    // }
                    _ => {
                        let expr = self.parse_expr()?;
                        self.expect(TokenType::Semicolon, "expected ';' after expression")?;
                        Stmt::Expr(expr)
                    }
                }
            }

            TokenType::Continue => {
                self.next();
                self.expect(TokenType::Semicolon, "expected ';' after continue")?;
                Stmt::Continue
            }

            TokenType::Break => {
                self.next(); // consume curr tok
                self.expect(TokenType::Semicolon, "expected ';' after break")?;
                Stmt::Break
            }

            TokenType::Return => {
                self.next(); // consume ret
                if self.peek().kind != TokenType::Semicolon {
                    let expr = self.parse_expr()?;
                    self.expect(TokenType::Semicolon, "expected ';' after return")?;
                    Stmt::Return(Some(expr))
                } else {
                    self.expect(TokenType::Semicolon, "expected ';' after return")?;
                    Stmt::Return(None)
                }
            }

            _ => {
                let expr = self.parse_expr()?;
                self.expect(TokenType::Semicolon, "expected ';' after expression")?;
                Stmt::Expr(expr)
            }
        };

        Ok(statement)
    }

    // fnDecl -> "fn" IDENT "(" (IDENT ("," IDENT)*)? ")" block
    fn parse_fn(&mut self) -> Result<Stmt, String> {
        self.next(); // ::fn

        let name = self.next().lexeme;

        self.expect(TokenType::LParen, "expected '(' after fn definition")?;

        let mut args: Vec<String> = vec![]; // vec<expr::ident>

        if self.check(TokenType::RParen) {
            self.next();
        } else {
            // must be ident
            args.push(
                self.expect(
                    TokenType::Identifier,
                    "func arg names can only be identifiers",
                )?
                .lexeme,
            );

            while !matches!(self.peek().kind, TokenType::RParen) {
                self.expect(TokenType::Comma, "expected ',' between args")?;
                args.push(
                    self.expect(
                        TokenType::Identifier,
                        "func arg names can only be identifiers",
                    )?
                    .lexeme,
                );
            }

            self.expect(TokenType::RParen, "expected ')' after args")?;
        }

        let block = self.parse_block()?;

        Ok(Stmt::Fn {
            name,
            args,
            block: Box::new(block),
        })
    }

    // assignStmt -> IDENT "=" expr
    fn parse_assign(&mut self) -> Result<Stmt, String> {
        // consume ident
        let name = self.next().lexeme;

        let assign_op: Option<TokenType> = match self.peek().kind {
            TokenType::PlusEqual => Some(TokenType::Plus),
            TokenType::MinusEqual => Some(TokenType::Minus),
            TokenType::TimesEqual => Some(TokenType::Times),
            TokenType::DivEqual => Some(TokenType::Divide),
            TokenType::ModuloEqual => Some(TokenType::Modulo),
            _ => {
                self.expect(TokenType::Assign, "expected assignment")?;
                None
            }
        };

        if let Some(op) = assign_op {
            self.next(); // consume op
            let expr = self.parse_expr()?;
            Ok(Stmt::Assignment(
                name.clone(),
                Expr::Binary(Box::new(Expr::Ident(name)), op, Box::new(expr)),
            ))
        } else {
            let expr = self.parse_expr()?;
            Ok(Stmt::Assignment(name, expr))
        }
    }

    // block -> "{" statement* "}"
    fn parse_block(&mut self) -> Result<Stmt, String> {
        let mut statements = vec![];

        // consume LBrace
        self.expect(TokenType::LBrace, "expected '{'")?;

        while !self.check(TokenType::RBrace) {
            statements.push(self.parse_statement()?);
        }

        // consume RBrace
        self.expect(TokenType::RBrace, "expected '}'")?;

        Ok(Stmt::Block(statements))
    }

    // params -> "(" (expr ("," expr)*)? ")"
    fn parse_args(&mut self) -> Result<Vec<Expr>, String> {
        self.expect(TokenType::LParen, "expected '(' after function")?;

        let mut args: Vec<Expr> = vec![];

        if self.check(TokenType::RParen) {
            self.next();
            return Ok(args);
        }

        let arg = self.parse_expr()?; // returns error if expr not found
        args.push(arg);

        // parse rest of the args
        while self.peek().kind == TokenType::Comma {
            // consume comma
            self.expect(TokenType::Comma, "expected comma after arg")?;
            let arg = self.parse_expr()?;
            args.push(arg);
        }

        self.expect(TokenType::RParen, "expected ')' after function call")?;
        Ok(args)
    }

    // letStmt -> "let" IDENT "=" expr ";"
    fn parse_let(&mut self) -> Result<Stmt, String> {
        // consume let
        self.expect(TokenType::Let, "expected 'let'")?;

        let ident = self.expect(TokenType::Identifier, "expected identifier after let")?;
        let name = ident.lexeme;

        self.expect(TokenType::Assign, "expected assignment after identifier")?;

        let expr = self.parse_expr()?;

        self.expect(
            TokenType::Semicolon,
            "expected semicolon after let statement",
        )?;

        Ok(Stmt::Let(name, expr))
    }

    // whileStmt -> "while" "(" expr ")" statement
    fn parse_while(&mut self) -> Result<Stmt, String> {
        // consume while
        self.expect(TokenType::While, "expected 'while'")?;

        self.expect(TokenType::LParen, "expected '(' after while")?;
        let cond: Expr = self.parse_expr()?;
        self.expect(TokenType::RParen, "expected ')' after condition")?;
        let block: Stmt = self.parse_block()?;

        Ok(Stmt::While(cond, Box::new(block)))
    }

    // forStmt -> "for" "(" letStmt expr ";" expr ")" statement
    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.expect(TokenType::For, "expected 'for'")?;

        self.expect(TokenType::LParen, "expected '(' after for")?;

        // init expr or skip
        let init: Stmt = match self.peek().kind {
            TokenType::Let => self.parse_let()?,
            TokenType::Semicolon => {
                self.next();
                Stmt::Expr(Expr::Null)
            }
            _ => {
                return Err(format!(
                    "unexpected error while parsing init in for loop, found {:?} expected ';'",
                    self.peek().kind,
                ));
            }
        };

        let cond: Expr = self.parse_expr()?;
        self.expect(TokenType::Semicolon, "expected ';' after end condition")?;

        let step: Stmt = match self.peek().kind {
            TokenType::Identifier => self.parse_assign()?,
            TokenType::RParen => Stmt::Expr(Expr::Null),
            _ => {
                return Err(format!(
                    "unexpected error while parsing step in for loop, found {:?} expected ')'",
                    self.peek().kind,
                ));
            }
        };

        self.expect(TokenType::RParen, "expected ')' after condition")?;

        let block: Stmt = self.parse_block()?;

        Ok(Stmt::For {
            init: Box::new(init),
            cond,
            step: Box::new(step),
            block: Box::new(block),
        })
    }

    // ifStmt -> "if" "(" expr ")" block ("elif" block)* ("else" block)?
    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.next(); // precondition: curr tok == if || elif

        self.expect(TokenType::LParen, "expected '(' after while")?;
        let cond: Expr = self.parse_expr()?;
        self.expect(TokenType::RParen, "expected ')' after condition")?;

        let block = self.parse_block()?;

        // NOTE:
        // if (0) {}
        // elif (1) {}
        // else {}
        // ======>
        // if (0) {}
        // else {
        //  if (1) {}
        //  else {}
        // }
        //
        // 1) parse if stmt
        // 2) see elif -> recurse parse if stmt
        // 3) repeat until else is spotted
        // 4) return else block

        let else_branches: Option<Box<Stmt>> = match self.peek().kind {
            // wlog, elif reduces to else statements containing if statements
            TokenType::Elif => Some(Box::new(self.parse_if()?)),
            TokenType::Else => {
                self.next(); // consume else
                Some(Box::new(Stmt::If {
                    cond: Expr::Bool(true),
                    then_branch: Box::new(self.parse_block()?),
                    else_branch: None,
                }))
            }
            _ => None,
        };

        Ok(Stmt::If {
            cond,
            then_branch: Box::new(block),
            else_branch: else_branches,
        })
    }

    // fn parse_fn(&mut self) -> Result<Stmt, String> {}

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let expr = self.pratt(0)?;
        Ok(expr)
    }

    fn pratt(&mut self, min_bp: usize) -> Result<Expr, String> {
        // 6 * 2 + 3 / (3 + 2) -> (6 * 2) + (3 / (3 + 2))
        let start: Token = self.next();
        // println!("{:?}", start);

        let mut lhs = self.nud(start)?;
        // println!("{:?}", lhs);

        loop {
            // parse postfix
            if matches!(self.peek().kind, TokenType::LParen) {
                let args: Vec<Expr> = self.parse_args()?;

                // we will verify that this callable is valid when eval'ing
                // for now we just pass any nud
                lhs = Expr::Callable {
                    name: Box::new(lhs),
                    args,
                };
                continue;
            }

            // lhs <op>, else ignore curr token and break
            // we let parent's handle other tokens
            let op = match self.peek().kind.clone() {
                k if self.is_infix(&k) => k,
                _ => break,
            };

            // println!("{:?} {}", op, min_bp);

            let (lbp, rbp) = self.bp(&op);
            if min_bp > lbp {
                break;
            }

            self.next(); // consume op

            let rhs = self.pratt(rbp)?;

            lhs = Expr::Binary(Box::new(lhs), op, Box::new(rhs))
        }

        Ok(lhs)
    }

    fn bp(&self, op: &TokenType) -> (usize, usize) {
        self.lookup.get(&op).copied().unwrap()
    }

    fn is_infix(&self, op: &TokenType) -> bool {
        self.lookup.contains_key(op)
    }

    fn nud(&mut self, token: Token) -> Result<Expr, String> {
        let out = match token.kind {
            TokenType::Number => Expr::Number(token.lexeme.parse().unwrap()),
            TokenType::String => Expr::String(token.lexeme),
            TokenType::True => Expr::Bool(true),
            TokenType::False => Expr::Bool(false),
            TokenType::Identifier => Expr::Ident(token.lexeme),

            TokenType::Minus => {
                let rhs = self.pratt(30)?;
                Expr::Unary(TokenType::Minus, Box::new(rhs))
            }

            TokenType::Bang => {
                let rhs = self.pratt(30)?;
                Expr::Unary(TokenType::Bang, Box::new(rhs))
            }

            TokenType::LParen => {
                let rhs = self.pratt(0)?;
                self.expect(TokenType::RParen, "expected ')' after expression")?;
                Expr::Group(Box::new(rhs))
            }

            other => {
                return Err(format!(
                    "unexpected token in nud: {:?} {:#?}",
                    other,
                    self.peek().span
                ));
            }
        };

        Ok(out)
    }

    fn check(&mut self, kind: TokenType) -> bool {
        if self.pos >= self.tokens.len() {
            return false;
        }

        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(&kind)
    }

    fn expect(&mut self, kind: TokenType, msg: &str) -> Result<Token, String> {
        if !self.check(kind) {
            return Err(format!("{} at {:#?}", msg, self.peek().span));
        }

        Ok(self.next())
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn at(&self, offset: usize) -> &Token {
        &self.tokens[self.pos + offset]
    }

    fn next(&mut self) -> Token {
        self.pos += 1;
        self.tokens[self.pos - 1].clone()
    }
}
