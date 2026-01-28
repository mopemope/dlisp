use crate::ast::Value;
use async_recursion::async_recursion;
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

use crate::jit::JIT;

pub struct Interpreter {
    pub jit: JIT,
}

impl Interpreter {
    pub fn new() -> Self {
        Self { jit: JIT::new() }
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
                // Handle special forms like defun
                if let Value::Symbol(ref s) = list[0] {
                    if s == "defun" {
                        // (defun name (args...) body...)
                        if list.len() < 4 {
                            return Err("defun requires at least 3 arguments".to_string());
                        }
                        let name = match &list[1] {
                            Value::Symbol(n) => n.clone(),
                            _ => return Err("defun name must be a symbol".to_string()),
                        };
                        let args_val = match &list[2] {
                            Value::List(l) => l,
                            _ => return Err("defun args must be a list".to_string()),
                        };
                        let mut arg_names = Vec::new();
                        for arg in args_val {
                            match arg {
                                Value::Symbol(n) => arg_names.push(n.clone()),
                                _ => return Err("defun arg must be a symbol".to_string()),
                            }
                        }
                        // Body is the rest
                        let body = list[3..].to_vec();

                        // Try JIT compilation
                        let jit_code = match self.jit.compile(&name, &arg_names, &body) {
                            Ok(code) => Some(code as usize),
                            Err(_e) => {
                                // JIT failed (maybe unsupported ops), ignore and use interpreter
                                // println!("JIT compilation failed for {}: {}", name, e);
                                None
                            }
                        };

                        let func = Value::UserFunc {
                            args: arg_names,
                            body,
                            jit_code,
                        };
                        env.borrow_mut().set(name.clone(), func.clone());
                        return Ok(Value::Symbol(name));
                    } else if s == "spawn" {
                        // (spawn (func args...)) or (spawn func)
                        // This accepts an expression that evaluates to a function, or a function application
                        if list.len() < 2 {
                            return Err("spawn requires a function or function call".to_string());
                        }

                        // Evaluate the first argument to get the function
                        let func_val = self.eval(list[1].clone(), env).await?;
                        let func_args = if list.len() > 2 {
                            // (spawn func arg1 arg2) - wait, classic lisp spawn usually takes a lambda
                            // If we do (spawn (task)) -> list[1] is (task) which evaluates to result of task? No.
                            // If we do (spawn task) -> list[1] is task -> evaluates to UserFunc.
                            // Let's support (spawn func_symbol) for now as 0-arity call.
                            Vec::new()
                        } else {
                            Vec::new()
                        };

                        // Environment cloning for the new task
                        let env_clone = env.clone();

                        // We need to move `func_val` and `env_clone` into the future
                        tokio::task::spawn_local(async move {
                            match func_val {
                                Value::UserFunc {
                                    args: param_names,
                                    body,
                                    jit_code: _,
                                } => {
                                    // Check arg count
                                    if param_names.len() != func_args.len() {
                                        eprintln!("Spawned task argument mismatch");
                                        return;
                                    }

                                    // Create function scope env with PARENT as the cloned env
                                    let func_env = Environment::new(Some(env_clone));
                                    // Bind args if we supported them (currently empty)

                                    let func_env_rc = Rc::new(RefCell::new(func_env));
                                    let mut interpreter = Interpreter::new(); // New interpreter instance, but shared Env!

                                    for expr in body {
                                        if let Err(e) =
                                            interpreter.eval(expr, &mut func_env_rc.clone()).await
                                        {
                                            eprintln!("Spawned task error: {}", e);
                                        }
                                    }
                                }
                                Value::NativeFunc(f) => {
                                    if let Err(e) = f(&func_args).await {
                                        eprintln!("Spawned task error: {}", e);
                                    }
                                }
                                _ => {
                                    eprintln!("Spawn expected a function");
                                }
                            }
                        });

                        return Ok(Value::Nil);
                    }
                }

                // Function call
                let func_val = self.eval(list[0].clone(), env).await?;
                let mut args = Vec::new();
                for arg in &list[1..] {
                    args.push(self.eval(arg.clone(), env).await?);
                }

                match func_val {
                    Value::NativeFunc(f) => f(&args).await,
                    Value::UserFunc {
                        args: param_names,
                        body,
                        jit_code,
                    } => {
                        if args.len() != param_names.len() {
                            return Err(format!(
                                "Function expects {} arguments, got {}",
                                param_names.len(),
                                args.len()
                            ));
                        }

                        // Try JIT execution if available and args are integers
                        if let Some(code_ptr) = jit_code {
                            let all_ints = args.iter().all(|v| matches!(v, Value::Integer(_)));
                            if all_ints {
                                match args.len() {
                                    0 => {
                                        let func: extern "C" fn() -> i64 =
                                            unsafe { std::mem::transmute(code_ptr as *const u8) };
                                        return Ok(Value::Integer(func()));
                                    }
                                    1 => {
                                        let a1 = match args[0] {
                                            Value::Integer(i) => i,
                                            _ => 0,
                                        };
                                        let func: extern "C" fn(i64) -> i64 =
                                            unsafe { std::mem::transmute(code_ptr as *const u8) };
                                        return Ok(Value::Integer(func(a1)));
                                    }
                                    2 => {
                                        let a1 = match args[0] {
                                            Value::Integer(i) => i,
                                            _ => 0,
                                        };
                                        let a2 = match args[1] {
                                            Value::Integer(i) => i,
                                            _ => 0,
                                        };
                                        let func: extern "C" fn(i64, i64) -> i64 =
                                            unsafe { std::mem::transmute(code_ptr as *const u8) };
                                        return Ok(Value::Integer(func(a1, a2)));
                                    }
                                    3 => {
                                        let a1 = match args[0] {
                                            Value::Integer(i) => i,
                                            _ => 0,
                                        };
                                        let a2 = match args[1] {
                                            Value::Integer(i) => i,
                                            _ => 0,
                                        };
                                        let a3 = match args[2] {
                                            Value::Integer(i) => i,
                                            _ => 0,
                                        };
                                        let func: extern "C" fn(i64, i64, i64) -> i64 =
                                            unsafe { std::mem::transmute(code_ptr as *const u8) };
                                        return Ok(Value::Integer(func(a1, a2, a3)));
                                    }
                                    _ => {
                                        // Fallback
                                    }
                                }
                            }
                        }

                        // Interpreter fallback
                        let mut func_env = Environment::new(Some(env.clone()));
                        for (name, val) in param_names.iter().zip(args.into_iter()) {
                            func_env.set(name.clone(), val);
                        }
                        let func_env_rc = Rc::new(RefCell::new(func_env));
                        let mut result = Value::Nil;
                        for expr in body {
                            result = self.eval(expr, &mut func_env_rc.clone()).await?;
                        }
                        Ok(result)
                    }
                    _ => Err(format!("Not a function: {}", list[0])),
                }
            }
            _ => Ok(val), // Self-evaluating
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_eval_add() {
        let env = default_env();
        let mut interpreter = Interpreter::new();
        // (+ 1 2)
        let ast = Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Integer(1),
            Value::Integer(2),
        ]);
        let res = interpreter.eval(ast, &mut env.clone()).await;
        assert_eq!(res.unwrap(), Value::Integer(3));
    }

    #[tokio::test]
    async fn test_eval_nested() {
        let env = default_env();
        let mut interpreter = Interpreter::new();
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
        let res = interpreter.eval(ast, &mut env.clone()).await;
        assert_eq!(res.unwrap(), Value::Integer(6));
    }

    #[tokio::test]
    async fn test_eval_defun() {
        let env = default_env();
        let mut interpreter = Interpreter::new();
        // (defun add2 (x) (+ x 2))
        let defun_expr = Value::List(vec![
            Value::Symbol("defun".to_string()),
            Value::Symbol("add2".to_string()),
            Value::List(vec![Value::Symbol("x".to_string())]),
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Symbol("x".to_string()),
                Value::Integer(2),
            ]),
        ]);
        interpreter
            .eval(defun_expr, &mut env.clone())
            .await
            .unwrap();

        // (add2 3)
        let call_expr = Value::List(vec![Value::Symbol("add2".to_string()), Value::Integer(3)]);
        let res = interpreter.eval(call_expr, &mut env.clone()).await;
        assert_eq!(res.unwrap(), Value::Integer(5));
    }

    #[tokio::test]
    async fn test_eval_defun_3args() {
        let env = default_env();
        let mut interpreter = Interpreter::new();
        // (defun add3 (x y z) (+ x (+ y z)))
        let body_expr = Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Symbol("x".to_string()),
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Symbol("y".to_string()),
                Value::Symbol("z".to_string()),
            ]),
        ]);

        let defun_expr = Value::List(vec![
            Value::Symbol("defun".to_string()),
            Value::Symbol("add3".to_string()),
            Value::List(vec![
                Value::Symbol("x".to_string()),
                Value::Symbol("y".to_string()),
                Value::Symbol("z".to_string()),
            ]),
            body_expr,
        ]);
        interpreter
            .eval(defun_expr, &mut env.clone())
            .await
            .unwrap();

        // (add3 1 2 3)
        let call_expr = Value::List(vec![
            Value::Symbol("add3".to_string()),
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let res = interpreter.eval(call_expr, &mut env.clone()).await;
        assert_eq!(res.unwrap(), Value::Integer(6));
    }
}
