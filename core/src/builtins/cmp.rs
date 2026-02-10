use crate::ast::Value;

/// (>= a b): greater than or equal
pub fn gte(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err(">= requires exactly 2 arguments".to_string());
        }
        match (&args[0], &args[1]) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a >= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) >= *b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Bool(*a >= (*b as f64))),
            _ => Err("Arguments must be numbers".to_string()),
        }
    })
}

/// (<= a b): less than or equal
pub fn lte(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("<= requires exactly 2 arguments".to_string());
        }
        match (&args[0], &args[1]) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a <= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) <= *b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Bool(*a <= (*b as f64))),
            _ => Err("Arguments must be numbers".to_string()),
        }
    })
}

/// (/= a b): not equal
pub fn neq(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("/= requires exactly 2 arguments".to_string());
        }
        let is_equal = args[0] == args[1];
        Ok(Value::Bool(!is_equal))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gte() {
        assert_eq!(
            gte(&[Value::Integer(3), Value::Integer(3)]).await,
            Ok(Value::Bool(true))
        );
        assert_eq!(
            gte(&[Value::Integer(4), Value::Integer(3)]).await,
            Ok(Value::Bool(true))
        );
        assert_eq!(
            gte(&[Value::Integer(2), Value::Integer(3)]).await,
            Ok(Value::Bool(false))
        );
    }

    #[tokio::test]
    async fn test_lte() {
        assert_eq!(
            lte(&[Value::Integer(3), Value::Integer(3)]).await,
            Ok(Value::Bool(true))
        );
        assert_eq!(
            lte(&[Value::Integer(2), Value::Integer(3)]).await,
            Ok(Value::Bool(true))
        );
        assert_eq!(
            lte(&[Value::Integer(4), Value::Integer(3)]).await,
            Ok(Value::Bool(false))
        );
    }

    #[tokio::test]
    async fn test_neq() {
        assert_eq!(
            neq(&[Value::Integer(1), Value::Integer(2)]).await,
            Ok(Value::Bool(true))
        );
        assert_eq!(
            neq(&[Value::Integer(1), Value::Integer(1)]).await,
            Ok(Value::Bool(false))
        );
    }
}
