use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

/// `dotimes` special form: iterate a fixed number of times.
///
/// Syntax: `(dotimes (var count) body...)`
/// Binds `var` to 0, 1, ..., count-1 and evaluates body for each.
/// Returns Nil.
pub async fn dotimes_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, String> {
    if args.is_empty() {
        return Err("dotimes requires at least a binding form (var count)".to_string());
    }

    let binding = match &args[0] {
        Value::List(l) => l,
        _ => return Err("dotimes first argument must be (var count)".to_string()),
    };

    if binding.len() != 2 {
        return Err("dotimes binding must be (var count)".to_string());
    }

    let var_name = match &binding[0] {
        Value::Symbol(s) => s.clone(),
        _ => return Err("dotimes variable must be a symbol".to_string()),
    };

    let count_val = interpreter.eval(binding[1].clone(), env).await?;
    let count = match count_val {
        Value::Integer(n) => {
            if n < 0 {
                return Err("dotimes count must be non-negative".to_string());
            }
            n
        }
        _ => return Err("dotimes count must be an integer".to_string()),
    };

    let body = &args[1..];
    let mut loop_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));

    for i in 0..count {
        loop_env
            .borrow_mut()
            .set(var_name.clone(), Value::Integer(i));
        for expr in body {
            interpreter.eval(expr.clone(), &mut loop_env).await?;
        }
    }

    Ok(Some(Value::Nil))
}
