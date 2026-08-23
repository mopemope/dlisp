use crate::ast::Value;
use crate::environment::Environment;
use crate::eval_failure::EvalFailure;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

pub async fn eval_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, EvalFailure> {
    if args.len() != 1 {
        return Err("eval requires exactly 1 argument".to_string().into());
    }

    // Evaluate the argument to get the data structure (e.g. evaluating a quoted list returns the list itself)
    let expr_to_eval = interpreter.eval(args[0].clone(), env).await?;

    // Now evaluate the returned data structure as an expression
    let res = interpreter.eval(expr_to_eval, env).await?;

    Ok(Some(res))
}
