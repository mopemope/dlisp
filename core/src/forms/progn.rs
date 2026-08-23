use crate::ast::Value;
use crate::environment::Environment;
use crate::eval_failure::EvalFailure;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

pub async fn progn(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, EvalFailure> {
    let mut result = Value::Nil;
    for expr in args {
        result = interpreter.eval(expr.clone(), env).await?;
    }
    Ok(Some(result))
}
