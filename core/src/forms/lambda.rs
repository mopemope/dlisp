use crate::ast::Value;
use crate::environment::Environment;
use std::cell::RefCell;
use std::rc::Rc;

pub fn lambda(args: &[Value], env: &mut Rc<RefCell<Environment>>) -> Result<Option<Value>, String> {
    if args.is_empty() {
        // (lambda) is invalid? OR (lambda () ...) if args[0] is list
        // args[0] MUST be list of params.
        return Err("lambda requires argument list".to_string());
    }

    let params = match &args[0] {
        Value::List(l) => l,
        _ => return Err("lambda args must be a list".to_string()),
    };

    let mut arg_names = Vec::new();
    for arg in params {
        match arg {
            Value::Symbol(n) => arg_names.push(n.clone()),
            _ => return Err("lambda arg must be a symbol".to_string()),
        }
    }

    let body = args[1..].to_vec();

    // Capture environment
    let captured_env = Some(env.clone());

    // Create the function value
    let func = Value::UserFunc {
        args: arg_names,
        body,
        jit_code: None,
        env: captured_env,
    };

    Ok(Some(func))
}
