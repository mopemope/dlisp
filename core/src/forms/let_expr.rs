use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

pub async fn let_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, String> {
    if args.is_empty() {
        return Err("let requires at least a list of bindings".to_string());
    }

    let empty_vec = Vec::new();
    let bindings = match &args[0] {
        Value::List(l) => l,
        Value::Nil => &empty_vec,
        _ => return Err("let bindings must be a list".to_string()),
    };

    let body = args[1..].to_vec();

    // 1. Evaluate all binding values in the CURRENT environment (Parallel binding)
    let mut evaluated_bindings = Vec::new();
    for binding in bindings {
        match binding {
            Value::List(bind_pair) => {
                if bind_pair.len() != 2 {
                    return Err(
                        "let binding must be a list of two elements: (pattern value)".to_string(),
                    );
                }
                let pattern = &bind_pair[0];
                let val_expr = &bind_pair[1];
                let val = interpreter.eval(val_expr.clone(), env).await?;

                crate::forms::destructure::bind_destructure(
                    pattern,
                    &val,
                    &mut evaluated_bindings,
                )?;
            }
            _ => return Err("let binding must be a list".to_string()),
        }
    }

    // 2. Create new scope and bind variables
    let mut new_env_inner = Environment::new(Some(env.clone()));
    for (name, val) in evaluated_bindings {
        new_env_inner.set(name, val);
    }
    let mut new_env = Rc::new(RefCell::new(new_env_inner));

    // 3. Evaluate body in new scope
    let mut result = Value::Nil;
    for expr in body {
        result = interpreter.eval(expr, &mut new_env).await?;
    }

    Ok(Some(result))
}
