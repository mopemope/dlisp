use crate::ast::Value;
use crate::environment::Environment;
use crate::eval_failure::EvalFailure;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

/// `throw` special form: evaluate its argument, store it in interpreter state,
/// and interrupt execution by returning a catchable error message.
pub async fn throw_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, EvalFailure> {
    if args.len() != 1 {
        return Err(EvalFailure::message("throw requires exactly 1 argument"));
    }
    let val = interpreter.eval(args[0].clone(), env).await?;
    interpreter.last_error = Some(val);

    Err(EvalFailure::Message("DLISP_THROW".to_string()))
}
