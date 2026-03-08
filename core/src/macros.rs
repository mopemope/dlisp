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

    let mut args_vec = Vec::new();
    let mut rest_param = None;

    match args_list_val {
        Value::List(l) => {
            let mut iter = l.iter();
            while let Some(arg) = iter.next() {
                match arg {
                    Value::Symbol(s) => {
                        if s == "&rest" {
                            if let Some(Value::Symbol(rest_name)) = iter.next() {
                                rest_param = Some(rest_name.clone());
                                if iter.next().is_some() {
                                    return Err(
                                        "defmacro &rest must be the last parameter".to_string()
                                    );
                                }
                                break;
                            } else {
                                return Err(
                                    "defmacro &rest must be followed by a symbol".to_string()
                                );
                            }
                        } else {
                            args_vec.push(s.clone());
                        }
                    }
                    _ => return Err("defmacro arguments must be symbols".to_string()),
                }
            }
        }
        Value::Nil => {}
        _ => return Err("defmacro arguments must be a list".to_string()),
    };

    Ok(Value::Macro {
        args: args_vec,
        rest_param,
        body,
    })
}
