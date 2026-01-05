use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type EnvRef = Rc<RefCell<Env>>;

#[derive(Debug)]
pub struct Env {
    pub(crate) parent: Option<EnvRef>,
    pub(crate) bucket: HashMap<String, Value>,
}

impl Env {
    pub fn new(parent: Option<EnvRef>) -> Env {
        return Self {
            parent: parent,
            bucket: HashMap::new(),
        };
    }

    pub fn push_scope(parent: &EnvRef) -> EnvRef {
        // prev <- new, ret new
        Rc::new(RefCell::new(Env::new(Some(Rc::clone(parent)))))
    }

    pub fn get(head: EnvRef, key: &str) -> Option<Value> {
        // search from curr node until parent == None
        let mut curr = head;

        loop {
            let parent = {
                let env = curr.borrow();

                // search
                if let Some(v) = env.bucket.get(key) {
                    return Some(v.clone());
                }

                env.parent.clone()
            };

            match parent {
                Some(next) => curr = next,
                None => return None,
            }
        }
    }

    pub fn define(start: EnvRef, key: String, val: Value) {
        let mut env = start.borrow_mut();
        env.bucket.insert(key.clone(), val.clone());
    }

    pub fn assign(start: EnvRef, key: String, val: Value) -> Result<(), String> {
        let mut curr = start;

        loop {
            let parent = {
                let mut env = curr.borrow_mut();

                if let Some(_) = env.bucket.get(&key) {
                    env.bucket.insert(key, val);
                    return Ok(());
                }

                env.parent.clone()
            };

            match parent {
                Some(p) => curr = p,
                None => return Err("cannot reassign value to undefined variable".into()),
            }
        }
    }
}
