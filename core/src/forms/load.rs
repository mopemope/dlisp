use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use crate::parser::parse;
use std::cell::RefCell;
use std::rc::Rc;

pub async fn load_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, String> {
    if args.is_empty() {
        return Err("load requires a string argument representing the file path".to_string());
    }

    let path_val = interpreter.eval(args[0].clone(), env).await?;

    let path = match path_val {
        Value::String(s) => s,
        _ => return Err("load requires a string file path".to_string()),
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => return Err(format!("Failed to read file '{}': {}", path, e)),
    };

    let mut result = Value::Nil;
    match parse(&content) {
        Ok(vals) => {
            for val in vals {
                result = interpreter.eval(val, env).await?;
            }
            Ok(Some(result))
        }
        Err(e) => Err(format!("Parse error in '{}': {:?}", path, e)),
    }
}
