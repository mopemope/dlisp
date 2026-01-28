use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

pub async fn spawn(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, String> {
    if args.is_empty() {
        return Err("spawn requires a function or function call".to_string());
    }

    let func_val = interpreter.eval(args[0].clone(), env).await?;
    let env_clone = env.clone();

    tokio::task::spawn_local(async move {
        match func_val {
            Value::UserFunc {
                args: _param_names,
                body,
                jit_code: _,
            } => {
                let func_env = Environment::new(Some(env_clone));
                let func_env_rc = Rc::new(RefCell::new(func_env));
                let mut interpreter = Interpreter::new();
                for expr in body {
                    if let Err(e) = interpreter.eval(expr, &mut func_env_rc.clone()).await {
                        eprintln!("Spawned task error: {}", e);
                    }
                }
            }
            Value::NativeFunc(f) => {
                if let Err(e) = f(&[]).await {
                    eprintln!("Spawned task error: {}", e);
                }
            }
            _ => eprintln!("Spawn expected a function"),
        }
    });

    Ok(Some(Value::Nil))
}
