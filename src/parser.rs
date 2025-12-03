use crate::token::{Token, TokenType};
use maplit::hashmap;
use std::collections::HashMap;

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
pub enum Expr {
    Binary(Box<Expr>, TokenType, Box<Expr>),
    Unary(TokenType, Box<Expr>),
    Number(i32),
    Ident(String),
    String(String),
    Group(Box<Expr>),
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
            // TokenType::Fn => self.parse_fn()?,
            _ => {
                let expr = self.parse_expr()?;
                self.expect(TokenType::Semicolon, "expected ';' after expression");
                Stmt::Expr(expr)
            }
        };

        Ok(statement)
    }

    // block -> "{" statement* "}"
    fn parse_block(&mut self) -> Result<Stmt, String> {
        let mut statements = vec![];

        // consume LBrace
        self.expect(TokenType::LBrace, "expected '{'");

        while !self.check(TokenType::RBrace) {
            statements.push(self.parse_statement()?);
        }

        // consume RBrace
        self.expect(TokenType::RBrace, "expected '}'");

        Ok(Stmt::Block(statements))
    }

    // letStmt -> "let" IDENT "=" expr ";"
    fn parse_let(&mut self) -> Result<Stmt, String> {
        // consume let
        self.expect(TokenType::Let, "expected 'let'");

        let name = match self.next().kind {
            TokenType::Identifier(s) => s,
            _ => return Err("expected identifier after let".into()),
        };

        self.expect(TokenType::Assign, "expected assignment after identifier");

        let expr = self.parse_expr()?;

        self.expect(
            TokenType::Semicolon,
            "expected semicolon after end condition",
        );

        Ok(Stmt::Let(name, expr))
    }

    // whileStmt -> "while" "(" expr ")" statement
    fn parse_while(&mut self) -> Result<Stmt, String> {
        // consume while
        self.expect(TokenType::While, "expected 'while'");

        self.expect(TokenType::LParen, "expected '(' after while");
        let cond: Expr = self.parse_expr()?;
        let block: Stmt = self.parse_block()?;
        self.expect(TokenType::RParen, "expected ')' after condition");

        Ok(Stmt::While(cond, Box::new(block)))
    }

    // forStmt -> "for" "(" letStmt expr ";" expr ")" statement
    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.next(); // consume for

        self.expect(TokenType::LParen, "expected '(' after for");

        let init: Stmt = self.parse_let()?;
        let cond: Expr = self.parse_expr()?;
        self.expect(TokenType::Semicolon, "expected ';' after end condition");
        let step: Expr = self.parse_expr()?;

        self.expect(TokenType::RParen, "expected ')' after condition");

        let block: Stmt = self.parse_block()?;

        Ok(Stmt::For(Box::new(init), cond, step, Box::new(block)))
    }

    // fn parse_fn(&mut self) -> Result<Stmt, String> {}

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let expr = self.pratt(0);

        Ok(expr)
    }

    fn pratt(&mut self, min_bp: usize) -> Expr {
        // NOTE:
        // if first token starts with a prefix op (! or - or '(') >
        //     turn into unary or parse group expr
        // else return js the literal

        // *a
        //
        // if (next tok is an infix op) consume op > else break and return expr
        // parse left side w bp of op
        // if (bp of next op < curr) stop and then make this lhs
        // ^ say 6 * 6 + 5 -> we need (6 * 6) + 5 and not 6 * (6 + 5)
        // else loop back to *a
        //
        // start again from point a

        let start: Token = self.next();
        println!("{:?}", start);

        let mut lhs = self.nud(start);
        println!("{:?}", lhs);

        loop {
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

            let rhs = self.pratt(rbp);

            lhs = Expr::Binary(Box::new(lhs), op, Box::new(rhs))
        }

        lhs
    }

    fn bp(&self, op: &TokenType) -> (usize, usize) {
        self.lookup.get(&op).copied().unwrap()
    }

    fn is_infix(&self, op: &TokenType) -> bool {
        self.lookup.contains_key(op)
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
                let rhs = self.pratt(0);
                self.expect(TokenType::RParen, "expected ')' after expression");
                Expr::Group(Box::new(rhs))
            }

            other => panic!("unexpected token in nud: {:?}", other),
        }
    }

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
            panic!("{} at {:#?}", msg, self.peek().span);
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
