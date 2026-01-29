use crate::ast::Value;

pub fn lt(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("< requires exactly 2 arguments".to_string());
        }

        match (&args[0], &args[1]) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(if a < b { 1 } else { 0 })),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Integer(if a < b { 1 } else { 0 })),
            (Value::Integer(a), Value::Float(b)) => {
                Ok(Value::Integer(if (*a as f64) < *b { 1 } else { 0 }))
            }
            (Value::Float(a), Value::Integer(b)) => {
                Ok(Value::Integer(if *a < (*b as f64) { 1 } else { 0 }))
            }
            _ => Err("Arguments must be numbers".to_string()),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lt_integers() {
        assert_eq!(
            lt(&[Value::Integer(1), Value::Integer(2)]).await,
            Ok(Value::Integer(1))
        );
        assert_eq!(
            lt(&[Value::Integer(2), Value::Integer(1)]).await,
            Ok(Value::Integer(0))
        );
        assert_eq!(
            lt(&[Value::Integer(1), Value::Integer(1)]).await,
            Ok(Value::Integer(0))
        );
    }

    #[tokio::test]
    async fn test_lt_floats() {
        assert_eq!(
            lt(&[Value::Float(1.0), Value::Float(2.0)]).await,
            Ok(Value::Integer(1))
        );
        assert_eq!(
            lt(&[Value::Float(2.0), Value::Float(1.0)]).await,
            Ok(Value::Integer(0))
        );
    }

    #[tokio::test]
    async fn test_lt_mixed() {
        assert_eq!(
            lt(&[Value::Integer(1), Value::Float(1.5)]).await,
            Ok(Value::Integer(1))
        );
        assert_eq!(
            lt(&[Value::Float(2.5), Value::Integer(2)]).await,
            Ok(Value::Integer(0))
        );
    }
}
