use crate::ast::Value;

pub fn sub(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("Expected at least one argument".to_string());
    }
    // Simplistic implementation: (- a b c) => a - b - c
    // (- a) => -a
    // ...
    // For brevity, implement basic binary or generic subtraction
    // Let's defer full implementation or do a simple one.
    // Supporting just binary for now or multi-arg.
    let first = &args[0];
    let mut i_acc = 0;
    let mut f_acc = 0.0;
    let mut is_float = false;

    match first {
        Value::Integer(n) => i_acc = *n,
        Value::Float(n) => {
            is_float = true;
            f_acc = *n;
        }
        _ => return Err("Expected number".to_string()),
    }

    if args.len() == 1 {
        // Negation
        return if is_float {
            Ok(Value::Float(-f_acc))
        } else {
            Ok(Value::Integer(-i_acc))
        };
    }

    for arg in &args[1..] {
        match arg {
            Value::Integer(n) => {
                if is_float {
                    f_acc -= *n as f64;
                } else {
                    i_acc -= n;
                }
            }
            Value::Float(n) => {
                if !is_float {
                    is_float = true;
                    f_acc = i_acc as f64;
                }
                f_acc -= n;
            }
            _ => return Err(format!("Expected number, got {}", arg)),
        }
    }

    if is_float {
        Ok(Value::Float(f_acc))
    } else {
        Ok(Value::Integer(i_acc))
    }
}
