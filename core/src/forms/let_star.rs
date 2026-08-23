use crate::ast::Value;
use crate::environment::Environment;
use crate::eval_failure::EvalFailure;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

/// `let*` special form: sequential binding.
/// Each binding can refer to previously bound variables in the same `let*`.
///
/// Syntax: `(let* ((x 1) (y (+ x 1))) body...)`
pub async fn let_star_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, EvalFailure> {
    if args.is_empty() {
        return Err("let* requires at least a list of bindings"
            .to_string()
            .into());
    }

    let empty_vec = Vec::new();
    let bindings = match &args[0] {
        Value::List(l) => l,
        Value::Nil => &empty_vec,
        _ => return Err("let* bindings must be a list".to_string().into()),
    };

    let body = args[1..].to_vec();

    // Start with the incoming environment
    let mut current_env = env.clone();

    // Evaluate each binding sequentially, creating a new nested environment for each
    for binding in bindings {
        match binding {
            Value::List(bind_pair) => {
                if bind_pair.len() != 2 {
                    return Err(EvalFailure::message(
                        "let* binding must be a list of two elements: (pattern value)",
                    ));
                }
                let pattern = &bind_pair[0];

                // Evaluate in the current environment
                let val = interpreter
                    .eval(bind_pair[1].clone(), &mut current_env)
                    .await?;

                let mut new_bindings = Vec::new();
                crate::forms::destructure::bind_destructure(pattern, &val, &mut new_bindings)?;

                // Create a new scope for the next bindings and body, extending the current one
                let mut new_scope = Environment::new(Some(current_env.clone()));
                for (sym, v) in new_bindings {
                    new_scope.set(sym, v);
                }
                current_env = Rc::new(RefCell::new(new_scope));
            }
            _ => return Err("let* binding must be a list".to_string().into()),
        }
    }

    // Evaluate body in the final nested scope
    let mut result = Value::Nil;
    for expr in body {
        result = interpreter.eval(expr, &mut current_env).await?;
    }

    Ok(Some(result))
}
