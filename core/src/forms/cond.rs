use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

fn is_truthy(val: &Value) -> bool {
    !matches!(val, Value::Nil | Value::Integer(0))
}

pub async fn cond(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, String> {
    for clause in args {
        match clause {
            Value::List(pair) => {
                if pair.is_empty() {
                    return Err("cond clause must be a non-empty list".to_string());
                }
                let test_val = interpreter.eval(pair[0].clone(), env).await?;
                if is_truthy(&test_val) {
                    // Evaluate remaining expressions in the clause, return last
                    if pair.len() == 1 {
                        return Ok(Some(test_val));
                    }
                    let mut result = Value::Nil;
                    for expr in &pair[1..] {
                        result = interpreter.eval(expr.clone(), env).await?;
                    }
                    return Ok(Some(result));
                }
            }
            _ => return Err("cond clause must be a list".to_string()),
        }
    }
    Ok(Some(Value::Nil))
}
