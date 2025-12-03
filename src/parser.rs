use crate::token::{Token, TokenType};
use maplit::hashmap;
use std::collections::HashMap;

// lexer.tokenize() -> Vec<Token> ->
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    ast: Vec<Stmt>,
    lookup: HashMap<TokenType, (usize, usize)>,
}

#[derive(Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Let(String, Expr),
    While(Expr, Box<Stmt>),
    For(Box<Stmt>, Expr, Expr, Box<Stmt>),
    Fn(String, Vec<String>, Box<Stmt>),
    Expr(Expr),
}

#[derive(Debug)]
enum Expr {
    Binary(Box<Expr>, TokenType, Box<Expr>),
    Unary(TokenType, Box<Expr>),
    Number(i32),
    Ident(String),
    String(String),
    Group(Box<Expr>),
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            ast: vec![],
            lookup: hashmap! {
                TokenType::Plus => (10, 11),
                TokenType::Minus => (10, 11),
                TokenType::Times => (20, 21),
                TokenType::Divide => (20, 21),
                TokenType::Modulo => (20, 21),

                TokenType::Assign => (1, 2),
                TokenType::Bang => (30, 31),

                TokenType::Less => (5, 6),
                TokenType::Greater => (5, 6),
                TokenType::EqualEqual => (4, 5),
                TokenType::NotEqual => (4, 5),
                TokenType::LessEqual => (5, 6),
                TokenType::GreaterEqual => (5, 6),
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

    // parse statement
    // if token is { parse block
    // if token is let parse let
    // while -> parse while loop
    // for -> parse for loop
    // fn -> parse function
    // expr -> parse expression (pratt)
    pub fn parse_statement(&mut self) -> Result<Stmt, String> {
        let token = self.peek();

        let statement: Stmt = match token.kind {
            TokenType::LBrace => self.parse_block()?,
            TokenType::Let => self.parse_let()?,
            TokenType::While => self.parse_while()?,
            TokenType::For => self.parse_for()?,
            // TokenType::Fn => self.parse_fn()?,
            _ => {
                let expr = self.parse_expr()?;
                Stmt::Expr(expr)
            }
        };

        Ok(statement)
    }

    fn parse_block(&mut self) -> Result<Stmt, String> {
        let mut statements = vec![];

        self.next(); // consume LBrace

        while !self.check(TokenType::RBrace) {
            statements.push(self.parse_statement()?);
        }

        self.next(); // consume }

        Ok(Stmt::Block(statements))
    }

    fn parse_let(&mut self) -> Result<Stmt, String> {
        // letStmt -> "let" IDENT "=" expr ";"
        // consume let
        self.next();

        let name = match self.next().kind {
            TokenType::Identifier(s) => s,
            _ => return Err("expected identifier after let".into()),
        };

        if !self.check(TokenType::Assign) {
            return Err("expected '=' after identifier".into());
        }
        self.next();

        let expr = self.parse_expr()?;

        if !self.check(TokenType::Semicolon) {
            return Err("expected ';' after statement".into());
        }
        self.next();

        Ok(Stmt::Let(name, expr))
    }

    // whileStmt -> "while" "(" expr ")" statement
    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.next(); // consume while

        let cond: Expr = self.parse_expr()?;
        let block: Stmt = self.parse_block()?;

        Ok(Stmt::While(cond, Box::new(block)))
    }

    // forStmt -> "for" "(" letStmt expr ";" expr ")" statement
    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.next(); // consume for

        let init: Stmt = self.parse_let()?;
        let cond: Expr = self.parse_expr()?;

        self.expect(
            TokenType::Semicolon,
            "expected semicolon after end condition",
        );

        let step: Expr = self.parse_expr()?;

        self.expect(TokenType::LBrace, "expected block after condition");

        let block: Stmt = self.parse_block()?;

        Ok(Stmt::For(Box::new(init), cond, step, Box::new(block)))
    }

    // fn parse_fn(&mut self) -> Result<Stmt, String> {}

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let expr = self.pratt(0);

        Ok(expr)
    }

    fn bp(&self, op: &TokenType) -> (usize, usize) {
        self.lookup.get(&op).copied().unwrap()
    }

    fn is_infix(&self, op: &TokenType) -> bool {
        self.lookup.contains_key(op)
    }

    fn pratt(&mut self, min_bp: usize) -> Expr {
        // if first token starts with a prefix op (! or - or '(') >
        //  turn into unary or parse group expr
        // else return js the literal
        //
        // *a
        // if next tok is an infix op consume op else break and return expr
        // parse left side w bp of op
        // if bp of next op < curr stop and then make this lhs
        //
        // start again from point a

        let first = self.next();
        let mut lhs = self.nud(first);

        loop {
            let op = match self.peek().kind.clone() {
                k if self.is_infix(&k) => k,
                _ => break,
            };

            let (lbp, rbp) = self.bp(&op);
            if min_bp > lbp {
                break;
            }

            self.next(); // consume op

            let rhs = self.pratt(rbp);
            lhs = Expr::Binary(Box::new(lhs), op, Box::new(rhs));
        }

        lhs
    }

    fn nud(&mut self, token: Token) -> Expr {
        match token.kind {
            TokenType::Number(n) => Expr::Number(n),
            TokenType::String(s) => Expr::String(s),
            TokenType::Identifier(name) => Expr::Ident(name),

            TokenType::Minus => {
                let rhs = self.pratt(30);
                Expr::Unary(TokenType::Minus, Box::new(rhs))
            }

            TokenType::Bang => {
                let rhs = self.pratt(30);
                Expr::Unary(TokenType::Bang, Box::new(rhs))
            }

            TokenType::LParen => {
                let expr = self.pratt(0);
                self.expect(TokenType::RParen, "expected ')' after expression");
                Expr::Group(Box::new(expr))
            }

            other => panic!("unexpected token in nud: {:?}", other),
        }
    }

    // why did i make this lol
    fn check(&mut self, kind: TokenType) -> bool {
        if self.pos >= self.tokens.len() {
            return false;
        }

        if self.peek().kind == kind {
            return true;
        }

        false
    }

    fn expect(&mut self, kind: TokenType, msg: &str) {
        if self.peek().kind != kind {
            panic!("{}", msg);
        }

        self.next();
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn next(&mut self) -> Token {
        self.pos += 1;
        self.tokens[self.pos - 1].clone()
    }
}
