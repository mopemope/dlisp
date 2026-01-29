use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

pub async fn apply(
    interpreter: &mut Interpreter,
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
            env: captured_env,
        } => {
            if args.len() != param_names.len() {
                return Err(format!(
                    "Function expects {} arguments, got {}",
                    param_names.len(),
                    args.len()
                ));
            }

            #[allow(clippy::collapsible_if)]
            if let Some(code_ptr) = jit_code {
                if let Some(result) =
                    unsafe { crate::jit_runner::run_jit_function(code_ptr as *const u8, &args) }
                {
                    return Ok(result);
                }
            }

            // Lexical scoping: use captured_env if available, otherwise use current env (dynamic/fallback)
            let parent_env = if let Some(c_env) = captured_env {
                c_env
            } else {
                env.clone()
            };

            let mut func_env = Environment::new(Some(parent_env));
            for (name, val) in param_names.iter().zip(args.into_iter()) {
                func_env.set(name.clone(), val);
            }
            let func_env_rc = Rc::new(RefCell::new(func_env));
            let mut result = Value::Nil;
            for expr in body {
                result = interpreter.eval(expr, &mut func_env_rc.clone()).await?;
            }
            Ok(result)
        }
        _ => Err("Value is not a function".to_string()),
    }
}
