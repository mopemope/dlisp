use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

/// `dolist` special form: iterate over elements of a list.
///
/// Syntax: `(dolist (var list-expr) body...)`
/// Binds `var` to each element of the list and evaluates body for each.
/// Returns Nil.
pub async fn dolist_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, String> {
    if args.is_empty() {
        return Err("dolist requires at least a binding form (var list-expr)".to_string());
    }

    let binding = match &args[0] {
        Value::List(l) => l,
        _ => return Err("dolist first argument must be (var list-expr)".to_string()),
    };

    if binding.len() != 2 {
        return Err("dolist binding must be (var list-expr)".to_string());
    }

    let var_name = match &binding[0] {
        Value::Symbol(s) => s.clone(),
        _ => return Err("dolist variable must be a symbol".to_string()),
    };

    let list_val = interpreter.eval(binding[1].clone(), env).await?;
    let elements = match list_val {
        Value::List(l) => l,
        Value::Vector(v) => v,
        Value::Nil => Vec::new(),
        _ => return Err("dolist requires a list or vector to iterate over".to_string()),
    };

    let body = &args[1..];
    let mut loop_env = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));

    for elem in elements {
        loop_env.borrow_mut().set(var_name.clone(), elem);
        for expr in body {
            interpreter.eval(expr.clone(), &mut loop_env).await?;
        }
    }

    Ok(Some(Value::Nil))
}
