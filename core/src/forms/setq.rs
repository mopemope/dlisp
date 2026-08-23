use crate::ast::Value;
use crate::environment::Environment;
use crate::eval_failure::EvalFailure;
use crate::interpreter::Interpreter;
use futures::future::LocalBoxFuture;
use std::cell::RefCell;
use std::rc::Rc;

pub fn setq<'a>(
    interpreter: &'a mut Interpreter,
    args: &'a [Value],
    env: &'a mut Rc<RefCell<Environment>>,
) -> LocalBoxFuture<'a, Result<Option<Value>, EvalFailure>> {
    Box::pin(async move {
        let len = args.len();
        if len == 0 || !len.is_multiple_of(2) {
            return Err("setq requires pairs of (symbol value) arguments"
                .to_string()
                .into());
        }

        let mut last_val = Value::Nil;
        for i in (0..len).step_by(2) {
            let symbol_name = match &args[i] {
                Value::Symbol(s) => s.clone(),
                _ => return Err("setq even arguments must be symbols".to_string().into()),
            };

            let val = interpreter.eval(args[i + 1].clone(), env).await?;

            // Attempt to update existing variable in env or its parents
            env.borrow_mut().assign(&symbol_name, val.clone())?;
            last_val = val;
        }

        Ok(Some(last_val))
    })
}
