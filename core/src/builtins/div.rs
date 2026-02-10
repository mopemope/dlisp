use crate::ast::Value;

/// (/ a b ...)
/// Division. With integers, performs integer division. If any argument is float, result is float.
pub fn div(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() < 2 {
            return Err("/ requires at least 2 arguments".to_string());
        }

        let mut is_float = false;
        let mut int_acc: i64 = 0;
        let mut float_acc: f64 = 0.0;

        for (i, arg) in args.iter().enumerate() {
            if i == 0 {
                match arg {
                    Value::Integer(v) => {
                        int_acc = *v;
                    }
                    Value::Float(v) => {
                        is_float = true;
                        float_acc = *v;
                    }
                    _ => return Err("Arguments must be numbers".to_string()),
                }
            } else {
                match arg {
                    Value::Integer(v) => {
                        if *v == 0 {
                            return Err("Division by zero".to_string());
                        }
                        if is_float {
                            float_acc /= *v as f64;
                        } else {
                            int_acc /= v;
                        }
                    }
                    Value::Float(v) => {
                        if *v == 0.0 {
                            return Err("Division by zero".to_string());
                        }
                        if !is_float {
                            is_float = true;
                            float_acc = int_acc as f64;
                        }
                        float_acc /= v;
                    }
                    _ => return Err("Arguments must be numbers".to_string()),
                }
            }
        }

        if is_float {
            Ok(Value::Float(float_acc))
        } else {
            Ok(Value::Integer(int_acc))
        }
    })
}

/// (% a b) or (mod a b)
/// Modulo operation. Integer only.
pub fn modulo(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("% requires exactly 2 arguments".to_string());
        }
        match (&args[0], &args[1]) {
            (Value::Integer(a), Value::Integer(b)) => {
                if *b == 0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Value::Integer(a % b))
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Value::Float(a % b))
            }
            (Value::Integer(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Value::Float((*a as f64) % b))
            }
            (Value::Float(a), Value::Integer(b)) => {
                if *b == 0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Value::Float(a % (*b as f64)))
            }
            _ => Err("Arguments must be numbers".to_string()),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_div_integers() {
        assert_eq!(
            div(&[Value::Integer(10), Value::Integer(3)]).await,
            Ok(Value::Integer(3))
        );
    }

    #[tokio::test]
    async fn test_div_floats() {
        assert_eq!(
            div(&[Value::Float(10.0), Value::Float(4.0)]).await,
            Ok(Value::Float(2.5))
        );
    }

    #[tokio::test]
    async fn test_div_mixed() {
        assert_eq!(
            div(&[Value::Integer(10), Value::Float(4.0)]).await,
            Ok(Value::Float(2.5))
        );
    }

    #[tokio::test]
    async fn test_div_by_zero() {
        assert!(div(&[Value::Integer(10), Value::Integer(0)]).await.is_err());
    }

    #[tokio::test]
    async fn test_div_chain() {
        // (/ 100 2 5) => 10
        assert_eq!(
            div(&[Value::Integer(100), Value::Integer(2), Value::Integer(5)]).await,
            Ok(Value::Integer(10))
        );
    }

    #[tokio::test]
    async fn test_mod_integers() {
        assert_eq!(
            modulo(&[Value::Integer(10), Value::Integer(3)]).await,
            Ok(Value::Integer(1))
        );
    }

    #[tokio::test]
    async fn test_mod_by_zero() {
        assert!(
            modulo(&[Value::Integer(10), Value::Integer(0)])
                .await
                .is_err()
        );
    }
}
