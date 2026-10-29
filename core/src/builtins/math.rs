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
            Value::Integer(i) => i
                .checked_abs()
                .map(Value::Integer)
                .ok_or_else(|| "Integer overflow".to_string()),
            Value::Float(f) => Ok(Value::Float(f.abs())),
            _ => Err("abs requires a number".to_string()),
        }
    })
}

/// Keep the full i64 exponent, including its parity and i64::MIN magnitude.
fn pow_integer_exponent(mut base: f64, exponent: i64) -> f64 {
    if let Ok(exponent) = i32::try_from(exponent) {
        return base.powi(exponent);
    }
    if exponent < 0 {
        base = base.recip();
    }
    let mut exponent = exponent.unsigned_abs();
    let mut result = 1.0;
    while exponent != 0 {
        if exponent & 1 != 0 {
            result *= base;
        }
        exponent >>= 1;
        if exponent != 0 {
            base *= base;
        }
    }
    result
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
                    Ok(Value::Float(pow_integer_exponent(*b as f64, *e)))
                } else if *e > u32::MAX as i64 {
                    Err(format!("pow: exponent {} is too large", e))
                } else {
                    match b.checked_pow(*e as u32) {
                        Some(result) => Ok(Value::Integer(result)),
                        None => Ok(Value::Float(pow_integer_exponent(*b as f64, *e))),
                    }
                }
            }
            (Value::Float(b), Value::Float(e)) => Ok(Value::Float(b.powf(*e))),
            (Value::Integer(b), Value::Float(e)) => Ok(Value::Float((*b as f64).powf(*e))),
            (Value::Float(b), Value::Integer(e)) => Ok(Value::Float(pow_integer_exponent(*b, *e))),
            _ => Err("pow requires numbers".to_string()),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_abs_min_integer_is_catchable_overflow() {
        assert_eq!(
            abs(&[Value::Integer(i64::MIN)]).await,
            Err("Integer overflow".into())
        );
    }

    #[tokio::test]
    async fn test_pow_does_not_truncate_integer_exponents() {
        for (base, exponent, expected) in [
            (Value::Integer(2), -4_294_967_296, 0.0),
            (Value::Integer(2), 2_147_483_648, f64::INFINITY),
            (Value::Float(2.0), 4_294_967_296, f64::INFINITY),
            (Value::Float(2.0), -4_294_967_296, 0.0),
            (Value::Float(-1.0), i64::MAX, -1.0),
            (Value::Float(-1.0), i64::MIN, 1.0),
        ] {
            assert_eq!(
                pow(&[base, Value::Integer(exponent)]).await,
                Ok(Value::Float(expected)),
                "exponent={exponent}"
            );
        }
    }

    #[tokio::test]
    async fn test_abs() {
        assert_eq!(abs(&[Value::Integer(-5)]).await, Ok(Value::Integer(5)));
        assert_eq!(abs(&[Value::Float(-1.23)]).await, Ok(Value::Float(1.23)));
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
