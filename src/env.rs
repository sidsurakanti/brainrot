use crate::value::Value;
use std::collections::HashMap;
use std::ops::Index;

pub struct Env {
    pub scopes: Vec<HashMap<String, Value>>,
}

impl Env {
    pub fn new() -> Env {
        return Self {
            scopes: vec![HashMap::new()],
        };
    }

    pub fn push_scope(&mut self) {
        let scope: HashMap<String, Value> = HashMap::new();
        self.scopes.push(scope);
    }

    // sounds like pop smoke lmao
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        // search from top of stack to bottom to find var
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(key) {
                return Some(v);
            }
        }

        None
    }

    pub fn define(&mut self, key: String, val: Value) {
        // search from top of stack to bottom to find var
        if let Some(curr_scope) = self.scopes.last_mut() {
            curr_scope.insert(key, val);
        }
    }

    pub fn assign(&mut self, key: String, val: Value) -> Result<(), String> {
        // search from top of stack to bottom to find var
        for scope in self.scopes.iter_mut().rev() {
            if let Some(_) = scope.get(&key) {
                scope.insert(key, val);
                return Ok(());
            }
        }

        Err("cannot reassign value to undefined variable".into())
    }
}

impl Index<&str> for Env {
    type Output = Value;

    fn index(&self, key: &str) -> &Self::Output {
        self.get(key)
            .unwrap_or_else(|| panic!("undefined variable '{}'", key))
    }
}
