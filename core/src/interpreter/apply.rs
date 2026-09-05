use crate::ast::Value;
use crate::environment::Environment;
use crate::eval_failure::EvalFailure;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

pub async fn apply(
    interpreter: &mut Interpreter,
    func: Value,
    args: Vec<Value>,
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Value, EvalFailure> {
    match func {
        Value::NativeFunc(f) => f(&args).await.map_err(EvalFailure::from),
        Value::UserFunc {
            args: param_names,
            rest_param,
            body,
            jit_code,
            env: captured_env,
        } => {
            // Validate argument count
            if let Some(ref _rest) = rest_param {
                // With &rest: at least param_names.len() args required
                if args.len() < param_names.len() {
                    return Err(format!(
                        "Function expects at least {} arguments, got {}",
                        param_names.len(),
                        args.len()
                    )
                    .into());
                }
            } else {
                // Without &rest: exact count required
                if args.len() != param_names.len() {
                    return Err(format!(
                        "Function expects {} arguments, got {}",
                        param_names.len(),
                        args.len()
                    )
                    .into());
                }
            }

            #[allow(clippy::collapsible_if)]
            if let Some(code_ptr) = jit_code {
                // Detect a throw thrown *during this call* only: a previously
                // swallowed throw (e.g. inside a higher-order builtin
                // callback, a documented limitation) leaves the flag set, and
                // keying off the absolute state would poison unrelated calls.
                let pending_before = dlisp_runtime::errors::dlisp_thrown_pending() != 0;

                let jit_result = unsafe {
                    crate::jit_runner::run_jit_function(
                        code_ptr as *const u8,
                        param_names.len(),
                        rest_param.is_some(),
                        &args,
                    )
                };

                // A compiled function that threw returns the sentinel without
                // a value; surface it as a catchable error instead of falling
                // back to the interpreter (which would re-run the body).
                if !pending_before && dlisp_runtime::errors::dlisp_thrown_pending() != 0 {
                    let thrown_ptr = dlisp_runtime::errors::dlisp_take_thrown();
                    let thrown_val = thrown_ptr
                        .is_null()
                        .then_some(Value::Nil)
                        .or_else(|| unsafe { crate::jit_runner::runtime_to_value(thrown_ptr) })
                        .unwrap_or(Value::Nil);
                    interpreter.last_error = Some(thrown_val);
                    return Err(EvalFailure::Message("DLISP_THROW".to_string()));
                }

                if let Some(result) = jit_result {
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

            // Bind named parameters
            for (name, val) in param_names.iter().zip(args.iter()) {
                func_env.set(name.clone(), val.clone());
            }

            // Bind &rest parameter (remaining args as a list)
            if let Some(rest_name) = rest_param {
                let rest_args = if args.len() > param_names.len() {
                    Value::List(args[param_names.len()..].to_vec())
                } else {
                    Value::Nil
                };
                func_env.set(rest_name, rest_args);
            }

            let mut func_env_rc = Rc::new(RefCell::new(func_env));
            let mut result = Value::Nil;
            for expr in body {
                result = interpreter.eval(expr, &mut func_env_rc).await?;
            }
            Ok(result)
        }
        _ => Err("Value is not a function".to_string().into()),
    }
}
