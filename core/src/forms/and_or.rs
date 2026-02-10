use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

fn is_truthy(val: &Value) -> bool {
    !matches!(val, Value::Nil | Value::Integer(0))
}

/// (and expr1 expr2 ...)
/// Short-circuit: returns the first falsy value, or the last value if all are truthy.
/// (and) with no args returns true (represented as Integer(1)).
pub async fn and_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, String> {
    if args.is_empty() {
        return Ok(Some(Value::Integer(1)));
    }
    let mut result = Value::Nil;
    for expr in args {
        result = interpreter.eval(expr.clone(), env).await?;
        if !is_truthy(&result) {
            return Ok(Some(result));
        }
    }
    Ok(Some(result))
}

/// (or expr1 expr2 ...)
/// Short-circuit: returns the first truthy value, or the last value if all are falsy.
/// (or) with no args returns nil.
pub async fn or_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, String> {
    if args.is_empty() {
        return Ok(Some(Value::Nil));
    }
    let mut result = Value::Nil;
    for expr in args {
        result = interpreter.eval(expr.clone(), env).await?;
        if is_truthy(&result) {
            return Ok(Some(result));
        }
    }
    Ok(Some(result))
}
