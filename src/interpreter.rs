use crate::ast::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)] // Rc<RefCell> implies we need careful PartialEq or just bypass?
// Environment usually is not compared for equality in tests.
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

    // Define a root environment with builtins?
}

pub fn eval(val: Value, env: &mut Rc<RefCell<Environment>>) -> Result<Value, String> {
    match val {
        Value::Symbol(s) => env
            .borrow()
            .get(&s)
            .ok_or_else(|| format!("Undefined symbol: {}", s)),
        Value::List(list) => {
            if list.is_empty() {
                return Ok(Value::Nil);
            }
            // Function call
            let func_val = eval(list[0].clone(), env)?;
            let mut args = Vec::new();
            for arg in &list[1..] {
                // Eager evaluation of arguments
                args.push(eval(arg.clone(), env)?);
            }

            match func_val {
                Value::NativeFunc(f) => f(&args),
                _ => Err(format!("Not a function: {}", list[0])),
            }
        }
        _ => Ok(val), // Self-evaluating (int, float, etc.)
    }
}

pub fn default_env() -> Rc<RefCell<Environment>> {
    let mut env = Environment::new(None);
    crate::builtins::install(&mut env);
    Rc::new(RefCell::new(env))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_add() {
        let env = default_env();
        // (+ 1 2)
        let ast = Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Integer(1),
            Value::Integer(2),
        ]);
        let res = eval(ast, &mut env.clone());
        assert_eq!(res.unwrap(), Value::Integer(3));
    }

    #[test]
    fn test_eval_nested() {
        let env = default_env();
        // (+ 1 (* 2 3)) is hard to construct manually, let's assume we have it correct or use parser in integration test.
        // Let's test basic recursion if we had it.
        // For now, manual AST construction:
        // (+ 1 (+ 2 3))
        let ast = Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Integer(1),
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Integer(2),
                Value::Integer(3),
            ]),
        ]);
        let res = eval(ast, &mut env.clone());
        assert_eq!(res.unwrap(), Value::Integer(6));
    }
}
