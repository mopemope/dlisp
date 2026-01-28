use crate::ast::Value;
use crate::environment::Environment;
use async_recursion::async_recursion;
use std::cell::RefCell;
use std::rc::Rc;

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

                // Try special forms first
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
        match name {
            "defun" => crate::forms::defun::defun(&mut self.jit, args, env),
            "spawn" => crate::forms::spawn::spawn(self, args, env).await,
            "let" => crate::forms::let_expr::let_form(self, args, env).await,
            _ => Ok(None),
        }
    }

    async fn apply(
        &mut self,
        func: Value,
        args: Vec<Value>,
        env: &mut Rc<RefCell<Environment>>,
    ) -> Result<Value, String> {
        match func {
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

                if let Some(code_ptr) = jit_code {
                    let all_ints = args.iter().all(|v| matches!(v, Value::Integer(_)));
                    if all_ints {
                        match args.len() {
                            0 => {
                                let func_ptr: extern "C" fn() -> i64 =
                                    unsafe { std::mem::transmute(code_ptr as *const u8) };
                                return Ok(Value::Integer(func_ptr()));
                            }
                            1 => {
                                let a1 = match args[0] {
                                    Value::Integer(i) => i,
                                    _ => 0,
                                };
                                let func_ptr: extern "C" fn(i64) -> i64 =
                                    unsafe { std::mem::transmute(code_ptr as *const u8) };
                                return Ok(Value::Integer(func_ptr(a1)));
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
                                let func_ptr: extern "C" fn(i64, i64) -> i64 =
                                    unsafe { std::mem::transmute(code_ptr as *const u8) };
                                return Ok(Value::Integer(func_ptr(a1, a2)));
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
                                let func_ptr: extern "C" fn(i64, i64, i64) -> i64 =
                                    unsafe { std::mem::transmute(code_ptr as *const u8) };
                                return Ok(Value::Integer(func_ptr(a1, a2, a3)));
                            }
                            _ => {}
                        }
                    }
                }

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
            _ => Err("Value is not a function".to_string()),
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

    #[tokio::test]
    async fn test_eval_spawn() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move {
                let env = default_env();
                let mut interpreter = Interpreter::new();

                // (defun task () (+ 1 2))
                let defun_expr = Value::List(vec![
                    Value::Symbol("defun".to_string()),
                    Value::Symbol("task".to_string()),
                    Value::List(vec![]),
                    Value::List(vec![
                        Value::Symbol("+".to_string()),
                        Value::Integer(1),
                        Value::Integer(2),
                    ]),
                ]);
                interpreter
                    .eval(defun_expr, &mut env.clone())
                    .await
                    .unwrap();

                // (spawn task)
                let spawn_expr = Value::List(vec![
                    Value::Symbol("spawn".to_string()),
                    Value::Symbol("task".to_string()),
                ]);

                let res = interpreter.eval(spawn_expr, &mut env.clone()).await;
                assert_eq!(res.unwrap(), Value::Nil);

                // Allow the spawned task to execute
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            })
            .await;
    }

    #[tokio::test]
    async fn test_eval_let() {
        let env = default_env();
        let mut interpreter = Interpreter::new();
        // (let ((x 10) (y 20)) (+ x y))
        let let_expr = Value::List(vec![
            Value::Symbol("let".to_string()),
            Value::List(vec![
                Value::List(vec![Value::Symbol("x".to_string()), Value::Integer(10)]),
                Value::List(vec![Value::Symbol("y".to_string()), Value::Integer(20)]),
            ]),
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Symbol("x".to_string()),
                Value::Symbol("y".to_string()),
            ]),
        ]);
        let res = interpreter.eval(let_expr, &mut env.clone()).await;
        assert_eq!(res.unwrap(), Value::Integer(30));
    }

    #[tokio::test]
    async fn test_eval_let_parallel_binding() {
        let env = default_env();
        // Define x = 100 in outer scope
        env.borrow_mut().set("x".to_string(), Value::Integer(100));

        let mut interpreter = Interpreter::new();
        // (let ((x 1) (y x)) y)
        // If sequential, y would be 1. If parallel, y should be 100.
        let let_expr = Value::List(vec![
            Value::Symbol("let".to_string()),
            Value::List(vec![
                Value::List(vec![Value::Symbol("x".to_string()), Value::Integer(1)]),
                Value::List(vec![
                    Value::Symbol("y".to_string()),
                    Value::Symbol("x".to_string()),
                ]),
            ]),
            Value::Symbol("y".to_string()),
        ]);
        let res = interpreter.eval(let_expr, &mut env.clone()).await;
        assert_eq!(res.unwrap(), Value::Integer(100));
    }
    #[tokio::test]
    async fn test_eval_let_empty_bindings() {
        let env = default_env();
        let mut interpreter = Interpreter::new();
        // (let () 1)
        let let_expr = Value::List(vec![
            Value::Symbol("let".to_string()),
            Value::List(vec![]),
            Value::Integer(1),
        ]);
        let res = interpreter.eval(let_expr, &mut env.clone()).await;
        assert_eq!(res.unwrap(), Value::Integer(1));
    }
}
