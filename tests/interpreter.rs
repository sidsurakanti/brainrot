use brainrot::interpreter::Interpreter;
use brainrot::interpreter::Value;

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
