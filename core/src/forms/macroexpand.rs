use crate::ast::Value;
use crate::environment::Environment;
use crate::eval_failure::EvalFailure;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

pub async fn macroexpand_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, EvalFailure> {
    if args.is_empty() {
        return Err("macroexpand requires 1 argument".to_string().into());
    }

    // Evaluate the argument to get the data structure
    let expr = interpreter.eval(args[0].clone(), env).await?;

    // Now expand it
    let expanded = interpreter.expand(expr, env).await?;

    Ok(Some(expanded))
}
