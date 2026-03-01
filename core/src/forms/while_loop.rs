use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

pub async fn while_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, String> {
    if args.is_empty() {
        return Err("while requires at least 1 argument (condition)".to_string());
    }

    let condition_expr = args[0].clone();
    let body_exprs = &args[1..];
    let mut last_result = Value::Nil;

    loop {
        // Evaluate the condition
        let cond_val = interpreter.eval(condition_expr.clone(), env).await?;
        if !cond_val.is_truthy() {
            break;
        }

        // Evaluate the body expressions sequentially
        for expr in body_exprs {
            last_result = interpreter.eval(expr.clone(), env).await?;
        }
    }

    Ok(Some(last_result))
}
