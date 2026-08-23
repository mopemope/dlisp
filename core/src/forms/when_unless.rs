use crate::ast::Value;
use crate::environment::Environment;
use crate::eval_failure::EvalFailure;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

/// `when` special form: evaluates body only if condition is truthy.
///
/// Syntax: `(when condition body...)`
/// Returns the result of the last body expression, or Nil if condition is false.
pub async fn when_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, EvalFailure> {
    if args.is_empty() {
        return Err("when requires at least a condition".to_string().into());
    }

    let cond = interpreter.eval(args[0].clone(), env).await?;
    if cond.is_truthy() {
        let mut result = Value::Nil;
        for expr in &args[1..] {
            result = interpreter.eval(expr.clone(), env).await?;
        }
        Ok(Some(result))
    } else {
        Ok(Some(Value::Nil))
    }
}

/// `unless` special form: evaluates body only if condition is falsy.
///
/// Syntax: `(unless condition body...)`
/// Returns the result of the last body expression, or Nil if condition is true.
pub async fn unless_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, EvalFailure> {
    if args.is_empty() {
        return Err("unless requires at least a condition".to_string().into());
    }

    let cond = interpreter.eval(args[0].clone(), env).await?;
    if !cond.is_truthy() {
        let mut result = Value::Nil;
        for expr in &args[1..] {
            result = interpreter.eval(expr.clone(), env).await?;
        }
        Ok(Some(result))
    } else {
        Ok(Some(Value::Nil))
    }
}
