use brainrot::interpreter::{Interpreter, LangError};
use brainrot::value::Value;

#[test]
fn assignment_updates_nearest_scope_only() -> Result<(), LangError> {
    let mut i = Interpreter::new();

    i.run(
        r#"
        let x = 1;
        {
            let x = 2;
            x = 3;
        }
        "#
        .into(),
    )?;

    assert_eq!(i.get("x").unwrap(), Value::Int(1));
    Ok(())
}

#[test]
fn dag_assignment_updates_nearest_ancestor() -> Result<(), LangError> {
    let mut i = Interpreter::new();

    i.run(
        r#"
        let x = 1;
        {
            let y = 2;
            {
                x = 5;
            }
        }
        "#
        .into(),
    )?;

    assert_eq!(i.get("x").unwrap(), Value::Int(5));
    assert!(i.get("y").is_none());
    Ok(())
}
