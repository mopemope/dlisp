use crate::ast::Value;
use futures::future::LocalBoxFuture;
use std::env;

pub fn getenv(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("getenv requires exactly 1 argument (name)".to_string());
        }
        let name = match &args[0] {
            Value::String(s) => s,
            _ => return Err("getenv argument must be a string".to_string()),
        };
        match env::var(name) {
            Ok(val) => Ok(Value::String(val)),
            Err(_) => Ok(Value::Nil),
        }
    })
}

pub fn setenv(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("setenv requires exactly 2 arguments (name value)".to_string());
        }
        let name = match &args[0] {
            Value::String(s) => s,
            _ => return Err("setenv first argument must be a string".to_string()),
        };
        let value = match &args[1] {
            Value::String(s) => s,
            _ => return Err("setenv second argument must be a string".to_string()),
        };
        unsafe {
            env::set_var(name, value);
        }
        Ok(Value::Bool(true))
    })
}

pub fn cwd(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if !args.is_empty() {
            return Err("cwd requires no arguments".to_string());
        }
        match env::current_dir() {
            Ok(path) => Ok(Value::String(path.to_string_lossy().to_string())),
            Err(e) => Err(format!("cwd error: {}", e)),
        }
    })
}

pub fn set_cwd(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("set-cwd requires exactly 1 argument (path)".to_string());
        }
        let path = match &args[0] {
            Value::String(s) => s,
            _ => return Err("set-cwd argument must be a string".to_string()),
        };
        match env::set_current_dir(path) {
            Ok(_) => Ok(Value::Bool(true)),
            Err(e) => Err(format!("set-cwd error: {}", e)),
        }
    })
}

pub fn exit(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        let code = if args.len() == 1 {
            match &args[0] {
                Value::Integer(i) => *i as i32,
                _ => return Err("exit argument must be an integer".to_string()),
            }
        } else if args.is_empty() {
            0
        } else {
            return Err("exit requires 0 or 1 arguments (code)".to_string());
        };
        std::process::exit(code);
    })
}

pub fn get_args(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if !args.is_empty() {
            return Err("args requires no arguments".to_string());
        }
        let arg_strings: Vec<Value> = env::args().map(Value::String).collect();
        Ok(Value::List(arg_strings))
    })
}
