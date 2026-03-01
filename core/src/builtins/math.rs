use crate::ast::Value;

/// (abs x)
/// Returns the absolute value of x.
pub fn abs(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("abs requires exactly 1 argument".to_string());
        }
        match &args[0] {
            Value::Integer(i) => Ok(Value::Integer(i.abs())),
            Value::Float(f) => Ok(Value::Float(f.abs())),
            _ => Err("abs requires a number".to_string()),
        }
    })
}

/// (pow base exp)
/// Returns base raised to the power of exp.
pub fn pow(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("pow requires exactly 2 arguments (base exp)".to_string());
        }
        match (&args[0], &args[1]) {
            (Value::Integer(b), Value::Integer(e)) => {
                if *e < 0 {
                    Ok(Value::Float((*b as f64).powi(*e as i32)))
                } else if *e > u32::MAX as i64 {
                    Err(format!("pow: exponent {} is too large", e))
                } else {
                    match b.checked_pow(*e as u32) {
                        Some(result) => Ok(Value::Integer(result)),
                        None => Ok(Value::Float((*b as f64).powi(*e as i32))),
                    }
                }
            }
            (Value::Float(b), Value::Float(e)) => Ok(Value::Float(b.powf(*e))),
            (Value::Integer(b), Value::Float(e)) => Ok(Value::Float((*b as f64).powf(*e))),
            (Value::Float(b), Value::Integer(e)) => Ok(Value::Float(b.powi(*e as i32))),
            _ => Err("pow requires numbers".to_string()),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_abs() {
        assert_eq!(abs(&[Value::Integer(-5)]).await, Ok(Value::Integer(5)));
        assert_eq!(abs(&[Value::Float(-3.14)]).await, Ok(Value::Float(3.14)));
    }

    #[tokio::test]
    async fn test_pow() {
        assert_eq!(
            pow(&[Value::Integer(2), Value::Integer(3)]).await,
            Ok(Value::Integer(8))
        );
        assert_eq!(
            pow(&[Value::Float(2.0), Value::Float(3.0)]).await,
            Ok(Value::Float(8.0))
        );
        assert_eq!(
            pow(&[Value::Integer(2), Value::Integer(-1)]).await,
            Ok(Value::Float(0.5))
        );
    }

    #[tokio::test]
    async fn test_pow_overflow_fallback() {
        // Large exponent that overflows i64: should fall back to float.
        let result = pow(&[Value::Integer(2), Value::Integer(100)]).await;
        match result {
            Ok(Value::Float(f)) => assert!((f - (2.0f64).powi(100)).abs() < 1e10),
            Ok(Value::Integer(_)) => {} // also acceptable if platform supports it
            e => panic!("Unexpected result: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_abs_zero() {
        assert_eq!(abs(&[Value::Integer(0)]).await, Ok(Value::Integer(0)));
        assert_eq!(abs(&[Value::Float(0.0)]).await, Ok(Value::Float(0.0)));
    }

    #[tokio::test]
    async fn test_abs_error() {
        assert!(abs(&[Value::String("x".to_string())]).await.is_err());
        assert!(abs(&[]).await.is_err());
    }

    #[tokio::test]
    async fn test_pow_error() {
        assert!(
            pow(&[Value::String("x".to_string()), Value::Integer(2)])
                .await
                .is_err()
        );
        assert!(pow(&[Value::Integer(2)]).await.is_err());
    }
}
