use crate::ast::Value;
use crate::environment::Environment;
use crate::forms::require::{normalize_source_dir, resolve_source_path};
use crate::interpreter::Interpreter;
use crate::parser::parse;
use std::cell::RefCell;
use std::fs;
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

    let resolved = {
        let borrowed = env.borrow();
        resolve_source_path(&path, &borrowed)
    };

    let content = match fs::read_to_string(&resolved) {
        Ok(c) => c,
        Err(e) => {
            return Err(format!(
                "Failed to read file '{}': {}",
                resolved.display(),
                e
            ));
        }
    };
    let canonical = fs::canonicalize(&resolved).unwrap_or_else(|_| resolved.clone());

    let mut result = Value::Nil;
    match parse(&content) {
        Ok(vals) => {
            if let Some(parent) = canonical.parent() {
                env.borrow_mut()
                    .push_source_dir(normalize_source_dir(parent));
            }
            let mut eval_error = None;
            for val in vals {
                match interpreter.eval(val, env).await {
                    Ok(value) => result = value,
                    Err(err) => {
                        eval_error = Some(err);
                        break;
                    }
                }
            }
            if canonical.parent().is_some() {
                env.borrow_mut().pop_source_dir();
            }
            if let Some(err) = eval_error {
                return Err(err);
            }
            Ok(Some(result))
        }
        Err(e) => Err(format!("Parse error in '{}': {:?}", resolved.display(), e)),
    }
}
