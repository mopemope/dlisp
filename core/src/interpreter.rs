use crate::ast::Value;
use crate::environment::Environment;
use async_recursion::async_recursion;
use std::cell::RefCell;
use std::rc::Rc;

use crate::jit::JIT;

pub mod apply;

use crate::forms::registry::{FormRegistry, standard_registry};

pub struct Interpreter {
    pub jit: JIT,
    pub forms: FormRegistry,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            jit: JIT::new(),
            forms: standard_registry(),
        }
    }

    #[async_recursion(?Send)]
    pub async fn eval(
        &mut self,
        val: Value,
        env: &mut Rc<RefCell<Environment>>,
    ) -> Result<Value, String> {
        match val {
            Value::Symbol(s) => env
                .borrow()
                .get(&s)
                .ok_or_else(|| format!("Undefined symbol: {}", s)),
            Value::List(list) => {
                if list.is_empty() {
                    return Ok(Value::Nil);
                }

                // Try special forms first
                #[allow(clippy::collapsible_if)]
                if let Value::Symbol(ref s) = list[0] {
                    if let Some(result) = self.eval_special_form(s, &list[1..], env).await? {
                        return Ok(result);
                    }
                }

                // Standard function call
                let func_val = self.eval(list[0].clone(), env).await?;
                let mut args = Vec::new();
                for arg in &list[1..] {
                    args.push(self.eval(arg.clone(), env).await?);
                }

                self.apply(func_val, args, env).await
            }
            _ => Ok(val), // Self-evaluating
        }
    }

    async fn eval_special_form(
        &mut self,
        name: &str,
        args: &[Value],
        env: &mut Rc<RefCell<Environment>>,
    ) -> Result<Option<Value>, String> {
        if let Some(form) = self.forms.get(name) {
            form.call(self, args, env).await
        } else {
            Ok(None)
        }
    }

    async fn apply(
        &mut self,
        func: Value,
        args: Vec<Value>,
        env: &mut Rc<RefCell<Environment>>,
    ) -> Result<Value, String> {
        crate::interpreter::apply::apply(self, func, args, env).await
    }
}

pub fn default_env() -> Rc<RefCell<Environment>> {
    let mut env = Environment::new(None);
    crate::builtins::install(&mut env);
    Rc::new(RefCell::new(env))
}

pub fn default_interpreter() -> Interpreter {
    Interpreter::new()
}
