use crate::ast::Value;
use crate::environment::Environment;
use crate::jit::JIT;
use std::cell::RefCell;
use std::rc::Rc;

pub fn defun(
    _jit: &mut JIT,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, String> {
    if args.len() < 3 {
        return Err("defun requires at least 3 arguments".to_string());
    }
    let func_name = match &args[0] {
        Value::Symbol(n) => n.clone(),
        _ => return Err("defun name must be a symbol".to_string()),
    };
    let params = match &args[1] {
        Value::List(l) => l,
        _ => return Err("defun args must be a list".to_string()),
    };
    let mut arg_names = Vec::new();
    for arg in params {
        match arg {
            Value::Symbol(n) => arg_names.push(n.clone()),
            _ => return Err("defun arg must be a symbol".to_string()),
        }
    }
    let body = args[2..].to_vec();

    /* JIT compilation in the interpreter is currently disabled due to ABI mismatch with the new boxed value strategy.
    let jit_code = match jit.compile(&func_name, &arg_names, &body) {
        Ok(code) => Some(code as usize),
        Err(_) => None,
    };
    */
    let jit_code = None;

    let func = Value::UserFunc {
        args: arg_names,
        body,
        jit_code,
        env: Some(env.clone()),
    };
    env.borrow_mut().set(func_name.clone(), func);
    Ok(Some(Value::Symbol(func_name)))
}
