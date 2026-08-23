use crate::ast::Value;
use crate::environment::Environment;
use crate::eval_failure::EvalFailure;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

pub async fn if_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, EvalFailure> {
    if args.len() < 2 {
        return Err("if requires at least condition and then-branch"
            .to_string()
            .into());
    }

    let cond = interpreter.eval(args[0].clone(), env).await?;

    if cond.is_truthy() {
        Ok(Some(interpreter.eval(args[1].clone(), env).await?))
    } else if args.len() > 2 {
        Ok(Some(interpreter.eval(args[2].clone(), env).await?))
    } else {
        Ok(Some(Value::Nil))
    }
}
