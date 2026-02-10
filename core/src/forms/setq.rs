use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use futures::future::LocalBoxFuture;
use std::cell::RefCell;
use std::rc::Rc;

pub fn setq<'a>(
    interpreter: &'a mut Interpreter,
    args: &'a [Value],
    env: &'a mut Rc<RefCell<Environment>>,
) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
    Box::pin(async move {
        if args.len() != 2 {
            return Err("setq requires exactly 2 arguments (symbol, value)".to_string());
        }

        let symbol_name = match &args[0] {
            Value::Symbol(s) => s.clone(),
            _ => return Err("setq first argument must be a symbol".to_string()),
        };

        let val = interpreter.eval(args[1].clone(), env).await?;

        // Attempt to update existing variable in env or its parents
        env.borrow_mut().assign(&symbol_name, val.clone())?;

        Ok(Some(val))
    })
}
