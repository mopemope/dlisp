use crate::ast::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Environment {
    pub parent: Option<Rc<RefCell<Environment>>>,
    pub values: HashMap<String, Value>,
}

impl Environment {
    pub fn new(parent: Option<Rc<RefCell<Environment>>>) -> Self {
        Environment {
            parent,
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.values.get(name) {
            Some(val.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }
}
