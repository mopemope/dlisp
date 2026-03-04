use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

/// `throw` special form: evaluate its argument, store it in interpreter state,
/// and interrupt execution by returning an Err.
pub async fn throw_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, String> {
    if args.len() != 1 {
        return Err("throw requires exactly 1 argument".to_string());
    }

    let val = interpreter.eval(args[0].clone(), env).await?;
    interpreter.last_error = Some(val);

    Err("DLISP_THROW".to_string())
}
