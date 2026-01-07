use brainrot::interpreter::{Interpreter, LangError};
use brainrot::value::Value;

#[test]
fn block_scope_does_not_leak() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        {
            let x = 10;
        }
        "#
        .into(),
    )?;

    assert!(i.get("x").is_none());
    Ok(())
}

#[test]
fn inner_scope_can_read_outer() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let x = 5;
        {
            let y = x + 1;
        }
        "#
        .into(),
    )?;

    assert_eq!(i.get("x").unwrap(), Value::Int(5));
    Ok(())
}

#[test]
fn inner_scope_can_shadow_outer() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let x = 1;
        {
            let x = 2;
        }
        "#
        .into(),
    )?;

    assert_eq!(i.get("x").unwrap(), Value::Int(1));
    Ok(())
}

#[test]
fn assignment_updates_nearest_scope() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let x = 1;
        {
            x = 2;
        }
        "#
        .into(),
    )?;

    assert_eq!(i.get("x").unwrap(), Value::Int(2));
    Ok(())
}

#[test]
fn for_loop_scope_does_not_leak() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        for (let i = 0; i < 3; i = i + 1) {
            let x = i;
        }
        "#
        .into(),
    )?;

    assert!(i.get("i").is_none());
    assert!(i.get("x").is_none());
    Ok(())
}

#[test]
fn assigning_undefined_variable_errs() {
    let mut i = Interpreter::new();
    let res = i.run(
        r#"
        x = 10;
        "#
        .into(),
    );

    assert!(matches!(res, Err(LangError::Runtime(_))));
}

#[test]
fn arithmetic_ops() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let a = 1 + 2 * 3;
        let b = (1 + 2) * 3;
        let c = -a;
        let d = a + b + c;
        let e = "ho" * 3;
        let s = "hi";
        let f = "hello " + "world";
        let g = 1 + 2 * 3 + 4 / 5 - 6 % 7; // should be 1
        "#
        .into(),
    )?;

    assert_eq!(i.get("a").unwrap(), Value::Int(7));
    assert_eq!(i.get("b").unwrap(), Value::Int(9));
    assert_eq!(i.get("c").unwrap(), Value::Int(-7));
    assert_eq!(i.get("d").unwrap(), Value::Int(9));
    assert_eq!(i.get("e").unwrap(), Value::Str("hohoho".into()));
    assert_eq!(i.get("s").unwrap(), Value::Str("hi".into()));
    assert_eq!(i.get("f").unwrap(), Value::Str("hello world".into()));
    assert_eq!(i.get("g").unwrap(), Value::Int(1));
    Ok(())
}

#[test]
fn assignment() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let a = 0;
        a = 1;
        "#
        .into(),
    )?;

    assert_eq!(i.get("a").unwrap(), Value::Int(1));
    Ok(())
}

#[test]
fn comparisons() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let a = true;
        let b = false;
        let c = a == "2";
        let d = a == b;
        "#
        .into(),
    )?;

    assert_eq!(i.get("c").unwrap(), Value::Bool(false));
    assert_eq!(i.get("d").unwrap(), Value::Bool(false));
    Ok(())
}

#[test]
fn if_elif_else() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let a = 13;

        if (a <= 10) {
            a = a + 1;
        } elif (15 > a) {
            a = a - 1;
        } elif (a == 15) {
            a = 0;
        } else {
            a = a + 20;
        }
        "#
        .into(),
    )?;

    assert_eq!(i.get("a").unwrap(), Value::Int(12));
    Ok(())
}

#[test]
fn while_break() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let a = 0;
        while (a <= 10) {
            a = a + 1;
            if (a > 5) {
                break;
            }
        }
        "#
        .into(),
    )?;

    assert_eq!(i.get("a").unwrap(), Value::Int(6));
    Ok(())
}

#[test]
fn for_basic_increment() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let sum = 0;
        for (let i = 0; i < 5; i = i + 1) {
            sum = sum + i;
        }
        "#
        .into(),
    )?;

    assert_eq!(i.get("sum").unwrap(), Value::Int(10));
    Ok(())
}

#[test]
fn for_without_init() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let i = 0;
        let sum = 0;

        for (; i < 5; i = i + 1) {
            sum = sum + i;
        }
        "#
        .into(),
    )?;

    assert_eq!(i.get("sum").unwrap(), Value::Int(10));
    Ok(())
}

#[test]
fn for_without_step() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let i = 0;
        let count = 0;

        for (let j = 0; j < 10;) {
            i = i + 1;
            j = j + 1;
            if (j == 3) {
                break;
            }
        }
        "#
        .into(),
    )?;

    assert_eq!(i.get("i").unwrap(), Value::Int(3));
    Ok(())
}

#[test]
fn for_with_break() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let i = 0;
        for (let j = 0; j < 10; j = j + 1) {
            i = i + 1;
            if (i == 3) {
                break;
            }
        }
        "#
        .into(),
    )?;

    assert_eq!(i.get("i").unwrap(), Value::Int(3));
    Ok(())
}

#[test]
fn for_with_continue() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let sum = 0;

        for (let i = 0; i < 5; i = i + 1) {
            if (i == 2) {
                continue;
            }
            sum = sum + i;
        }
        "#
        .into(),
    )?;

    assert_eq!(i.get("sum").unwrap(), Value::Int(8));
    Ok(())
}

#[test]
fn nested_for_loops() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let count = 0;
        for (let i = 0; i < 3; i = i + 1) {
            for (let j = 0; j < 2; j = j + 1) {
                count = count + 1;
            }
        }
        "#
        .into(),
    )?;

    assert_eq!(i.get("count").unwrap(), Value::Int(6));
    Ok(())
}

#[test]
fn closure_captures_outer_var() -> Result<(), LangError> {
    let mut i = Interpreter::new();

    i.run(
        r#"
        let x = 10;

        fn f(a) {
            print(a + x);
        }

        f(5);
        "#
        .into(),
    )?;

    Ok(())
}

#[test]
fn closure_uses_nearest_scope() -> Result<(), LangError> {
    let mut i = Interpreter::new();

    i.run(
        r#"
        let x = 10;

        fn f() {
            let x = 3;
            fn g() {
                print(x);
            }
            g();
        }

        f();
        "#
        .into(),
    )?;

    Ok(())
}

#[test]
fn closure_is_lexical_not_dynamic() -> Result<(), LangError> {
    let mut i = Interpreter::new();

    i.run(
        r#"
        fn f() {
            print(x);
        }

        let x = 10;
        f();
        "#
        .into(),
    )?;

    Ok(())
}

#[test]
fn nested_closure_capture() -> Result<(), LangError> {
    let mut i = Interpreter::new();

    i.run(
        r#"
        let x = 1;

        fn outer(a) {
            let y = 2;
            fn inner(b) {
                print(x + a + y + b);
            }
            inner(4);
        }

        outer(3);
        "#
        .into(),
    )?;

    Ok(())
}

#[test]
fn function_basic_returnless_call() -> Result<(), LangError> {
    let mut i = Interpreter::new();

    i.run(
        r#"
        fn foo() {
            let x = 1;
        }

        foo();
        "#
        .into(),
    )?;

    Ok(())
}

#[test]
fn function_with_multiple_params() -> Result<(), LangError> {
    let mut i = Interpreter::new();

    i.run(
        r#"
        fn add(a, b, c) {
            print(a + b + c);
        }

        add(1, 2, 3);
        "#
        .into(),
    )?;

    Ok(())
}

#[test]
fn function_does_not_leak_locals() -> Result<(), LangError> {
    let mut i = Interpreter::new();

    i.run(
        r#"
        fn foo() {
            let x = 42;
        }

        foo();
        "#
        .into(),
    )?;

    assert!(i.get("x").is_none());
    Ok(())
}

#[test]
fn function_can_mutate_outer_var() -> Result<(), LangError> {
    let mut i = Interpreter::new();

    i.run(
        r#"
        let x = 1;

        fn inc() {
            x = x + 1;
        }

        inc();
        inc();
        "#
        .into(),
    )?;

    assert_eq!(i.get("x").unwrap(), Value::Int(3));
    Ok(())
}

#[test]
fn function_shadowing_does_not_affect_outer() -> Result<(), LangError> {
    let mut i = Interpreter::new();

    i.run(
        r#"
        let x = 5;

        fn foo() {
            let x = 10;
        }

        foo();
        "#
        .into(),
    )?;

    assert_eq!(i.get("x").unwrap(), Value::Int(5));
    Ok(())
}

#[test]
fn calling_undefined_function_errors() {
    let mut i = Interpreter::new();

    let res = i.run(
        r#"
        foo();
        "#
        .into(),
    );

    assert!(matches!(res, Err(LangError::Runtime(_))));
}

#[test]
fn wrong_number_of_args_errors() {
    let mut i = Interpreter::new();

    let res = i.run(
        r#"
        fn foo(a, b) {
            print(a + b);
        }

        foo(1);
        "#
        .into(),
    );

    assert!(matches!(res, Err(LangError::Runtime(_))));
}

#[test]
fn function_return_basic() -> Result<(), LangError> {
    let mut i = Interpreter::new();

    i.run(
        r#"
        fn add(a, b) {
            return a + b;
        }

        let x = add(2, 3);
        "#
        .into(),
    )?;

    assert_eq!(i.get("x").unwrap(), Value::Int(5));
    Ok(())
}

#[test]
fn recursion_factorial() -> Result<(), LangError> {
    let mut i = Interpreter::new();

    i.run(
        r#"
        fn fact(n) {
            if (n <= 1) {
                return 1;
            }
            return n * fact(n - 1);
        }

        let x = fact(5);
        "#
        .into(),
    )?;

    assert_eq!(i.get("x").unwrap(), Value::Int(120));
    Ok(())
}
#[test]
fn logical_and_or_basic() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let a = true && true;
        let b = true && false;
        let c = false || true;
        let d = false || false;
        "#
        .into(),
    )?;

    assert_eq!(i.get("a").unwrap(), Value::Bool(true));
    assert_eq!(i.get("b").unwrap(), Value::Bool(false));
    assert_eq!(i.get("c").unwrap(), Value::Bool(true));
    assert_eq!(i.get("d").unwrap(), Value::Bool(false));
    Ok(())
}

#[test]
fn logical_precedence_and_over_or() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let x = true || false && false;
        let y = (true || false) && false;
        "#
        .into(),
    )?;

    assert_eq!(i.get("x").unwrap(), Value::Bool(true));
    assert_eq!(i.get("y").unwrap(), Value::Bool(false));
    Ok(())
}

#[test]
fn logical_with_comparisons() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let a = 5 > 3 && 2 < 4;
        let b = 5 < 3 || 10 == 10;
        "#
        .into(),
    )?;

    assert_eq!(i.get("a").unwrap(), Value::Bool(true));
    assert_eq!(i.get("b").unwrap(), Value::Bool(true));
    Ok(())
}

#[test]
fn logical_short_circuit_and() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let x = 0;
        false && (x = 1);
        "#
        .into(),
    )?;

    assert_eq!(i.get("x").unwrap(), Value::Int(0));
    Ok(())
}

#[test]
fn logical_short_circuit_or() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let x = 0;
        true || (x = 1);
        "#
        .into(),
    )?;

    assert_eq!(i.get("x").unwrap(), Value::Int(0));
    Ok(())
}

#[test]
fn logical_in_if_conditions() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let x = 0;

        if (x == 0 || x == 1) {
            x = 5;
        }

        if (x == 5 && true) {
            x = 10;
        }
        "#
        .into(),
    )?;

    assert_eq!(i.get("x").unwrap(), Value::Int(10));
    Ok(())
}

#[test]
fn logical_with_functions_and_side_effects() -> Result<(), LangError> {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let x = 0;

        fn inc() {
            x = x + 1;
            return true;
        }

        false && inc();
        true || inc();
        true && inc();
        "#
        .into(),
    )?;

    assert_eq!(i.get("x").unwrap(), Value::Int(1));
    Ok(())
}
