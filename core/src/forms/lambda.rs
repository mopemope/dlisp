use crate::ast::Value;
use crate::environment::Environment;
use crate::eval_failure::EvalFailure;
use std::cell::RefCell;
use std::rc::Rc;

pub fn lambda(
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, EvalFailure> {
    if args.is_empty() {
        // (lambda) is invalid? OR (lambda () ...) if args[0] is list
        // args[0] MUST be list of params.
        return Err("lambda requires argument list".to_string().into());
    }

    let params = match &args[0] {
        Value::List(l) => l,
        _ => return Err("lambda args must be a list".to_string().into()),
    };

    let mut arg_names = Vec::new();
    let mut rest_param = None;
    let mut i = 0;
    while i < params.len() {
        match &params[i] {
            Value::Symbol(n) if n == "&rest" => {
                if i + 1 >= params.len() {
                    return Err("&rest requires a parameter name".to_string().into());
                }
                match &params[i + 1] {
                    Value::Symbol(rest_name) => {
                        rest_param = Some(rest_name.clone());
                    }
                    _ => return Err("&rest parameter must be a symbol".to_string().into()),
                }
                if i + 2 < params.len() {
                    return Err("&rest parameter must be last in the parameter list"
                        .to_string()
                        .into());
                }
                break;
            }
            Value::Symbol(n) => arg_names.push(n.clone()),
            _ => return Err("lambda arg must be a symbol".to_string().into()),
        }
        i += 1;
    }

    let body = args[1..].to_vec();

    // Capture environment
    let captured_env = Some(env.clone());

    // Create the function value
    let func = Value::UserFunc {
        args: arg_names,
        rest_param,
        body,
        jit_code: None,
        env: captured_env,
    };

    Ok(Some(func))
}
