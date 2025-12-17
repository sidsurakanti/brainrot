use std::cmp::{PartialEq, PartialOrd};
use std::ops::{Add, Div, Mul, Neg, Not, Sub};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Value {
    Int(i32),
    Str(String),
    Bool(bool),
    Null,
}

impl Add for Value {
    type Output = Result<Value, String>;

    fn add(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(a + b.as_str())),
            _ => Err("type mismatch".into()),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, String>;
    fn sub(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            _ => Err("type mismatch".into()),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value, String>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Int(a) => Ok(Value::Int(-a)),
            _ => Err("type mismatch".into()),
        }
    }
}

impl Not for Value {
    type Output = Result<Value, String>;

    fn not(self) -> Self::Output {
        match self {
            Value::Str(_) | Value::Int(_) => Ok(Value::Bool(self.is_truthy())),
            Value::Bool(val) => Ok(Value::Bool(!val)),
            Value::Null => Err("cannot evaluate not for void".into()),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, String>;

    fn div(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    Err("cannot divide by zero".into())
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            _ => Err("type mismatch".into()),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, String>;

    fn mul(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Str(s), Value::Int(n)) | (Value::Int(n), Value::Str(s)) => {
                if n < 0 {
                    return Err("cannot multiply string by a negative integer".into());
                }

                Ok(Value::Str(s.repeat(n as usize)))
            }
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            _ => Err("type mismatch".into()),
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
        }
    }
}
