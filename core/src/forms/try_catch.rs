use crate::ast::Value;
use crate::environment::Environment;
use crate::forms::registry::SpecialForm;
use crate::interpreter::Interpreter;
use futures::future::LocalBoxFuture;
use std::cell::RefCell;
use std::rc::Rc;

pub struct TryCatchForm;

impl SpecialForm for TryCatchForm {
    fn call<'a>(
        &self,
        interp: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        let args_vec = args.to_vec();
        Box::pin(try_catch_impl(interp, args_vec, env))
    }
}

async fn try_catch_impl(
    interp: &mut Interpreter,
    args: Vec<Value>,
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, String> {
    // (try body... (catch var handler-body...))
    // We look for the `catch` clause at the end.

    if args.is_empty() {
        return Ok(Some(Value::Nil));
    }

    let mut body = args;
    let last_idx = body.len() - 1;

    // Find catch clause
    let mut catch_var = String::new();
    let mut catch_body = Vec::new();
    let mut has_catch = false;

    if let Value::List(last_list) = &body[last_idx]
        && !last_list.is_empty()
            && let Value::Symbol(s) = &last_list[0]
                && s == "catch" {
                    if last_list.len() < 2 {
                        return Err("catch requires a variable name".to_string());
                    }
                    if let Value::Symbol(var) = &last_list[1] {
                        catch_var = var.clone();
                        catch_body = last_list[2..].to_vec();
                        has_catch = true;
                        body.pop(); // Remove catch from body
                    } else {
                        return Err("catch variable must be a symbol".to_string());
                    }
                }

    // Evaluate body sequentially
    let mut last_val = Value::Nil;
    for form in body {
        match interp.eval(form, env).await {
            Ok(val) => {
                last_val = val;
            }
            Err(e) => {
                // Try to extract DLISP_THROW or wrap error string
                if has_catch {
                    // Extract thrown value from interpreter state if present.
                    let caught_val = if e == "DLISP_THROW" {
                        let thrown_val = interp.last_error.take().unwrap_or(Value::Nil);
                        Value::Error(Box::new(thrown_val))
                    } else {
                        // If it's a native Rust error or built-in error, wrap the string.
                        Value::Error(Box::new(Value::String(e.clone())))
                    };

                    // Create scope with bound variable
                    let mut catch_env = Environment::new(Some(env.clone()));
                    catch_env.set(catch_var, caught_val);
                    let mut catch_env_rc = Rc::new(RefCell::new(catch_env));

                    let mut result = Value::Nil;
                    for handler_form in catch_body {
                        result = interp.eval(handler_form, &mut catch_env_rc).await?;
                    }
                    return Ok(Some(result));
                } else {
                    // Uncaught, bubble up
                    return Err(e);
                }
            }
        }
    }

    Ok(Some(last_val))
}
