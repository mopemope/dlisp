use crate::ast::Value;

pub fn construct_macro(args: &[Value]) -> Result<Value, String> {
    // (defmacro name (args) body...)
    if args.len() < 3 {
        return Err("defmacro requires at least 3 arguments".to_string());
    }

    let _name = match &args[0] {
        Value::Symbol(n) => n.clone(),
        _ => return Err("defmacro name must be a symbol".to_string()),
    };

    let args_list_val = &args[1];
    let body = args[2..].to_vec();

    let args_vec = match args_list_val {
        Value::List(l) => {
            let mut arg_names = Vec::new();
            for arg in l {
                match arg {
                    Value::Symbol(s) => arg_names.push(s.clone()),
                    _ => return Err("defmacro arguments must be symbols".to_string()),
                }
            }
            arg_names
        }
        Value::Nil => Vec::new(),
        _ => return Err("defmacro arguments must be a list".to_string()),
    };

    Ok(Value::Macro {
        args: args_vec,
        body,
    })
}
