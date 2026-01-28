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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_integers() {
        // (- 10 3 2) => 5
        let args = vec![Value::Integer(10), Value::Integer(3), Value::Integer(2)];
        let result = sub(&args);
        assert_eq!(result, Ok(Value::Integer(5)));
    }

    #[test]
    fn test_sub_floats() {
        // (- 10.5 2.5) => 8.0
        let args = vec![Value::Float(10.5), Value::Float(2.5)];
        let result = sub(&args);
        assert_eq!(result, Ok(Value::Float(8.0)));
    }

    #[test]
    fn test_sub_negation() {
        // (- 5) => -5
        let args = vec![Value::Integer(5)];
        let result = sub(&args);
        assert_eq!(result, Ok(Value::Integer(-5)));
    }

    #[test]
    fn test_sub_empty_error() {
        let args = vec![];
        let result = sub(&args);
        assert!(result.is_err());
    }
}
