use crate::ast::Value;

pub fn eq(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("= requires exactly 2 arguments".to_string());
        }

        match (&args[0], &args[1]) {
            (Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Integer(if a == b { 1 } else { 0 }))
            }
            (Value::Float(a), Value::Float(b)) => {
                Ok(Value::Integer(if (a - b).abs() < f64::EPSILON {
                    1
                } else {
                    0
                }))
            }
            (Value::Integer(a), Value::Float(b)) => {
                Ok(Value::Integer(if (*a as f64 - *b).abs() < f64::EPSILON {
                    1
                } else {
                    0
                }))
            }
            (Value::Float(a), Value::Integer(b)) => {
                Ok(Value::Integer(if (*a - *b as f64).abs() < f64::EPSILON {
                    1
                } else {
                    0
                }))
            }
            // Add other equality checks if needed, e.g. strings, symbols
            (Value::Symbol(a), Value::Symbol(b)) => Ok(Value::Integer(if a == b { 1 } else { 0 })),
            (Value::String(a), Value::String(b)) => Ok(Value::Integer(if a == b { 1 } else { 0 })),
            (Value::Nil, Value::Nil) => Ok(Value::Integer(1)),
            _ => Ok(Value::Integer(0)), // Different types or values are not equal
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_eq_integers() {
        assert_eq!(
            eq(&[Value::Integer(1), Value::Integer(1)]).await,
            Ok(Value::Integer(1))
        );
        assert_eq!(
            eq(&[Value::Integer(1), Value::Integer(2)]).await,
            Ok(Value::Integer(0))
        );
    }

    #[tokio::test]
    async fn test_eq_floats() {
        assert_eq!(
            eq(&[Value::Float(1.0), Value::Float(1.0)]).await,
            Ok(Value::Integer(1))
        );
        assert_eq!(
            eq(&[Value::Float(1.0), Value::Float(2.0)]).await,
            Ok(Value::Integer(0))
        );
    }
}
