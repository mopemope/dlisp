use crate::ast::Value;
use crate::environment::Environment;
use crate::eval_failure::EvalFailure;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

pub async fn apply_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, EvalFailure> {
    if args.len() < 2 {
        return Err(EvalFailure::message(
            "apply requires at least 2 arguments (function and a list of arguments)",
        ));
    }

    // Evaluate the function
    let func_val = interpreter.eval(args[0].clone(), env).await?;

    // Evaluate the rest of the arguments to build the final argument list
    let mut eval_args = Vec::with_capacity(args.len() - 1);

    // For apply, all but the last argument are evaluated and pushed verbatim.
    // The last argument is evaluated and MUST be a list, whose elements are then appended.
    for arg in args.iter().take(args.len() - 1).skip(1) {
        eval_args.push(interpreter.eval(arg.clone(), env).await?);
    }

    let last_arg = interpreter.eval(args[args.len() - 1].clone(), env).await?;
    match last_arg {
        Value::List(l) => {
            eval_args.extend(l);
        }
        Value::Vector(v) => {
            eval_args.extend(v);
        }
        Value::Nil => {}
        _ => {
            return Err("apply requires the last argument to be a list or vector"
                .to_string()
                .into());
        }
    }

    let res = interpreter.apply(func_val, eval_args, env).await?;
    Ok(Some(res))
}
