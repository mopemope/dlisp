use crate::ast::Value;
use crate::environment::Environment;
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
) -> Result<Option<Value>, String> {
    if args.is_empty() {
        return Err("let* requires at least a list of bindings".to_string());
    }

    let empty_vec = Vec::new();
    let bindings = match &args[0] {
        Value::List(l) => l,
        Value::Nil => &empty_vec,
        _ => return Err("let* bindings must be a list".to_string()),
    };

    let body = args[1..].to_vec();

    // Create new scope
    let mut new_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));

    // Evaluate each binding sequentially in the growing new environment
    for binding in bindings {
        match binding {
            Value::List(bind_pair) => {
                if bind_pair.len() != 2 {
                    return Err(
                        "let* binding must be a list of two elements: (symbol value)".to_string(),
                    );
                }
                let symbol = match &bind_pair[0] {
                    Value::Symbol(s) => s.clone(),
                    _ => return Err("let* binding key must be a symbol".to_string()),
                };
                // Evaluate in the NEW environment so previous bindings are visible
                let val = interpreter.eval(bind_pair[1].clone(), &mut new_env).await?;
                new_env.borrow_mut().set(symbol, val);
            }
            _ => return Err("let* binding must be a list".to_string()),
        }
    }

    // Evaluate body in new scope
    let mut result = Value::Nil;
    for expr in body {
        result = interpreter.eval(expr, &mut new_env).await?;
    }

    Ok(Some(result))
}
