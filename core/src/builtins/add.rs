use crate::ast::Value;

pub fn add(args: &[Value]) -> Result<Value, String> {
    let mut i_sum = 0;
    let mut f_sum = 0.0;
    let mut is_float = false;

    for arg in args {
        match arg {
            Value::Integer(n) => {
                if is_float {
                    f_sum += *n as f64;
                } else {
                    i_sum += n;
                }
            }
            Value::Float(n) => {
                if !is_float {
                    is_float = true;
                    f_sum = i_sum as f64;
                }
                f_sum += n;
            }
            _ => return Err(format!("Expected number, got {}", arg)),
        }
    }

    if is_float {
        Ok(Value::Float(f_sum))
    } else {
        Ok(Value::Integer(i_sum))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_integers() {
        let args = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        let result = add(&args);
        assert_eq!(result, Ok(Value::Integer(6)));
    }

    #[test]
    fn test_add_floats() {
        let args = vec![Value::Float(1.5), Value::Float(2.5)];
        let result = add(&args);
        assert_eq!(result, Ok(Value::Float(4.0)));
    }

    #[test]
    fn test_add_mixed() {
        let args = vec![Value::Integer(1), Value::Float(2.5)];
        let result = add(&args);
        // 1 + 2.5 = 3.5
        assert_eq!(result, Ok(Value::Float(3.5)));
    }
}
