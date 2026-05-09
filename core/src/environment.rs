use crate::ast::Value;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Environment {
    pub parent: Option<Rc<RefCell<Environment>>>,
    pub values: HashMap<String, Value>,
    pub loaded_modules: HashSet<String>,
}

impl Environment {
    pub fn new(parent: Option<Rc<RefCell<Environment>>>) -> Self {
        Environment {
            parent,
            values: HashMap::new(),
            loaded_modules: HashSet::new(),
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

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().assign(name, value)
        } else {
            Err(format!("Undefined variable '{}'", name))
        }
    }

    pub fn has_loaded_module(&self, name: &str) -> bool {
        self.loaded_modules.contains(name)
            || self
                .parent
                .as_ref()
                .is_some_and(|parent| parent.borrow().has_loaded_module(name))
    }

    pub fn mark_loaded_module(&mut self, name: &str) {
        self.loaded_modules.insert(name.to_string());
    }
}
