use brainrot::interpreter::Interpreter;
use brainrot::value::Value;

// TODO: assert .run().is_ok() after converting errors from panic

#[test]
fn block_scope_does_not_leak() {
    let mut i = Interpreter::new();
    i.run(
        r#"
        {
            let x = 10;
        }
    "#
        .into(),
    );

    assert!(i.env.get("x").is_none());
}

#[test]
fn inner_scope_can_read_outer() {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let x = 5;
        {
            let y = x + 1;
        }
    "#
        .into(),
    );

    assert_eq!(i.env["x"], Value::Int(5));
}

#[test]
fn inner_scope_can_shadow_outer() {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let x = 1;
        {
            let x = 2;
        }
    "#
        .into(),
    );

    assert_eq!(i.env["x"], Value::Int(1));
}

#[test]
fn assignment_updates_nearest_scope() {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let x = 1;
        {
            x = 2;
        }
    "#
        .into(),
    );

    assert_eq!(i.env["x"], Value::Int(2));
}

#[test]
fn for_loop_scope_does_not_leak() {
    let mut i = Interpreter::new();
    i.run(
        r#"
        for (let i = 0; i < 3; i = i + 1) {
            let x = i;
        }
    "#
        .into(),
    );

    assert!(i.env.get("i").is_none());
    assert!(i.env.get("x").is_none());
}

#[test]
#[should_panic]
fn assigning_undefined_variable_panics() {
    let mut i = Interpreter::new();
    i.run(
        r#"
        x = 10;
    "#
        .into(),
    );
}

#[test]
fn arithmetic_ops() {
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
    "#
        .into(),
    );

    assert_eq!(i.env["a"], Value::Int(7));
    assert_eq!(i.env["b"], Value::Int(9));
    assert_eq!(i.env["c"], Value::Int(-7));
    assert_eq!(i.env["d"], Value::Int(9));
    assert_eq!(i.env["e"], Value::Str("hohoho".into()));
    assert_eq!(i.env["s"], Value::Str("hi".into()));
    assert_eq!(i.env["f"], Value::Str("hello world".into()));
}

#[test]
fn assignment() {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let a = 0;
        a = 1;
    "#
        .into(),
    );

    assert_eq!(i.env["a"], Value::Int(1));
}

#[test]
fn comparisons() {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let a = true;
        let b = false;
        let c = a == "2";
        let d = a == b;
    "#
        .into(),
    );

    assert_eq!(i.env["c"], Value::Bool(false));
    assert_eq!(i.env["d"], Value::Bool(false));
}

#[test]
fn if_elif_else() {
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
    );

    assert_eq!(i.env["a"], Value::Int(12));
}

#[test]
fn while_break() {
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
    );

    assert_eq!(i.env["a"], Value::Int(6));
}

#[test]
fn for_basic_increment() {
    let mut i = Interpreter::new();
    i.run(
        r#"
        let sum = 0;
        for (let i = 0; i < 5; i = i + 1) {
            sum = sum + i;
        }
        "#
        .into(),
    );

    // 0 + 1 + 2 + 3 + 4
    assert_eq!(i.env["sum"], Value::Int(10));
}

#[test]
fn for_without_init() {
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
    );

    assert_eq!(i.env["sum"], Value::Int(10));
}

#[test]
fn for_without_step() {
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
    );

    assert_eq!(i.env["i"], Value::Int(3));
}

#[test]
fn for_with_break() {
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
    );

    assert_eq!(i.env["i"], Value::Int(3));
}

#[test]
fn for_with_continue() {
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
    );

    // skips 2 → 0 + 1 + 3 + 4
    assert_eq!(i.env["sum"], Value::Int(8));
}

#[test]
fn nested_for_loops() {
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
    );

    assert_eq!(i.env["count"], Value::Int(6));
}
