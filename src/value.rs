use crate::env::Env;
use crate::interpreter::RuntimeError;
use crate::parser::Stmt;
use std::cell::RefCell;
use std::cmp::{PartialEq, PartialOrd};
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Not, Rem, Sub};
use std::rc::Rc;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Value {
    Int(i32),
    Str(String),
    Bool(bool),
    Null,
    Fn {
        name: String,
        args: Vec<String>,
        body: Box<Stmt>,
        captured_env: Rc<RefCell<Env>>,
    },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Fn { name, .. } => write!(f, "fn<{}>", name),
        }
    }
}
impl Add for Value {
    type Output = Result<Value, RuntimeError>;

    fn add(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(a + &b)),
            _ => Err(RuntimeError::type_mismatch()),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, RuntimeError>;

    fn sub(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            _ => Err(RuntimeError::type_mismatch()),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, RuntimeError>;

    fn mul(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Str(s), Value::Int(n)) | (Value::Int(n), Value::Str(s)) => {
                if n < 0 {
                    Err(RuntimeError::Message(
                        "cannot multiply string by negative integer".into(),
                    ))
                } else {
                    Ok(Value::Str(s.repeat(n as usize)))
                }
            }
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            _ => Err(RuntimeError::type_mismatch()),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, RuntimeError>;

    fn div(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Int(_), Value::Int(0)) => {
                Err(RuntimeError::Message("cannot divide by zero".into()))
            }
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
            _ => Err(RuntimeError::type_mismatch()),
        }
    }
}

impl Rem for Value {
    type Output = Result<Value, RuntimeError>;

    fn rem(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
            _ => Err(RuntimeError::Message(
                "modulo not implemented for this type".into(),
            )),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value, RuntimeError>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Int(a) => Ok(Value::Int(-a)),
            _ => Err(RuntimeError::type_mismatch()),
        }
    }
}

impl Not for Value {
    type Output = Result<Value, RuntimeError>;

    fn not(self) -> Self::Output {
        match self {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            Value::Int(_) | Value::Str(_) => Ok(Value::Bool(!self.is_truthy())),
            Value::Null => Err(RuntimeError::Message("cannot apply '!' to null".into())),
            Value::Fn { .. } => Err(RuntimeError::Message("cannot apply '!' to function".into())),
        }
    }
}

impl Value {
    pub fn logical_not(&self) -> Result<bool, RuntimeError> {
        match self {
            Value::Bool(b) => Ok(!b),
            Value::Int(_) | Value::Str(_) => Ok(!self.is_truthy()),
            Value::Null => Err(RuntimeError::Message("cannot apply '!' to null".into())),
            Value::Fn { .. } => Err(RuntimeError::Message("cannot apply '!' to function".into())),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            (Value::Str(a), Value::Str(b)) => a.partial_cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Int(n) => !(*n == 0),
            Value::Bool(b) => *b,
            Value::Str(s) => !s.is_empty(),
            Value::Null => false,
            Value::Fn { .. } => false,
        }
    }
}
