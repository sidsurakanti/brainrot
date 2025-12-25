use brainrot::lexer::Lexer;
use brainrot::parser::{Expr, Parser, Stmt};
use brainrot::token::TokenType;

fn parse(src: &str) -> Vec<Stmt> {
    let mut lexer = Lexer::new(src.into());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse().expect("parser error")
}

#[test]
fn parses_number_literal() {
    let ast = parse("42;");
    if let Stmt::Expr(Expr::Number(n)) = &ast[0] {
        assert_eq!(*n, 42);
    } else {
        panic!("number literal not parsed correctly");
    }
}

#[test]
fn parses_string_literal() {
    let ast = parse(r#""hello world";"#);
    if let Stmt::Expr(Expr::String(s)) = &ast[0] {
        assert_eq!(s, "hello world");
    } else {
        panic!("string literal not parsed correctly");
    }
}

#[test]
fn parses_let_statement() {
    let ast = parse("let a = 1;");
    assert!(matches!(ast[0], Stmt::Let(_, _)));
}

#[test]
fn parses_assignment() {
    let ast = parse("a = 2;");
    assert!(matches!(ast[0], Stmt::Assignment(_, _)));
}

#[test]
fn parses_expr_statement() {
    let ast = parse("1 + 2 * 3;");
    assert!(matches!(ast[0], Stmt::Expr(_)));
}

#[test]
fn respects_operator_precedence() {
    let ast = parse("let x = 1 + 2 * 3;");
    if let Stmt::Let(_, Expr::Binary(_, TokenType::Plus, rhs)) = &ast[0] {
        assert!(matches!(**rhs, Expr::Binary(_, TokenType::Times, _)));
    } else {
        panic!("bad precedence");
    }
}

#[test]
fn respects_grouping() {
    let ast = parse("let x = (1 + 2) * 3;");
    if let Stmt::Let(_, Expr::Binary(lhs, TokenType::Times, _)) = &ast[0] {
        assert!(matches!(**lhs, Expr::Group(_)));
    } else {
        panic!("grouping broken");
    }
}

#[test]
fn parses_if_else() {
    let ast = parse(
        r#"
        if (true) {
            let x = 1;
        } else {
            let x = 2;
        }
        "#,
    );

    assert!(matches!(ast[0], Stmt::If { .. }));
}

#[test]
fn parses_while_loop() {
    let ast = parse("while (x < 10) { x = x + 1; }");
    assert!(matches!(ast[0], Stmt::While(_, _)));
}

#[test]
fn parses_for_loop() {
    let ast = parse("for (let i = 0; i < 10; i = i + 1) { }");
    assert!(matches!(ast[0], Stmt::For { .. }));
}

#[test]
fn parses_function_call() {
    let ast = parse("foo(1, 2, 3);");
    assert!(matches!(ast[0], Stmt::Call(_, _)));
}

#[test]
fn parses_nested_blocks() {
    let ast = parse("{ { let x = 1; } }");
    if let Stmt::Block(inner) = &ast[0] {
        assert!(matches!(inner[0], Stmt::Block(_)));
    } else {
        panic!("nested block parse failed");
    }
}

#[test]
fn parses_plus_equal() {
    let ast = parse("a += 3;");
    if let Stmt::Assignment(name, Expr::Binary(lhs, TokenType::Plus, rhs)) = &ast[0] {
        assert_eq!(name, "a");
        assert!(matches!(**lhs, Expr::Ident(_)));
        assert!(matches!(**rhs, Expr::Number(3)));
    } else {
        panic!("+= not parsed correctly");
    }
}

#[test]
fn parses_minus_equal() {
    let ast = parse("a -= 1;");
    assert!(matches!(
        ast[0],
        Stmt::Assignment(_, Expr::Binary(_, TokenType::Minus, _))
    ));
}

#[test]
fn parses_times_equal() {
    let ast = parse("a *= 2;");
    assert!(matches!(
        ast[0],
        Stmt::Assignment(_, Expr::Binary(_, TokenType::Times, _))
    ));
}

#[test]
fn parses_divide_equal() {
    let ast = parse("a /= 4;");
    assert!(matches!(
        ast[0],
        Stmt::Assignment(_, Expr::Binary(_, TokenType::Divide, _))
    ));
}

#[test]
fn parses_modulo_equal() {
    let ast = parse("a %= 5;");
    assert!(matches!(
        ast[0],
        Stmt::Assignment(_, Expr::Binary(_, TokenType::Modulo, _))
    ));
}

#[test]
fn compound_assignment_desugars_correctly() {
    let ast = parse("x += y * 2;");
    if let Stmt::Assignment(_, Expr::Binary(_, TokenType::Plus, rhs)) = &ast[0] {
        assert!(matches!(**rhs, Expr::Binary(_, TokenType::Times, _)));
    } else {
        panic!("compound assignment desugaring broken");
    }
}
