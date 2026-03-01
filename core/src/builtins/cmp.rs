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

/// (max a b ...)
pub fn max(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.is_empty() {
            return Err("max requires at least 1 argument".to_string());
        }
        let mut max_val = args[0].clone();
        for arg in args.into_iter().skip(1) {
            match (&max_val, &arg) {
                (Value::Integer(a), Value::Integer(b)) => {
                    if b > a {
                        max_val = arg;
                    }
                }
                (Value::Float(a), Value::Float(b)) => {
                    if b > a {
                        max_val = arg;
                    }
                }
                (Value::Integer(a), Value::Float(b)) => {
                    if *b > (*a as f64) {
                        max_val = arg;
                    }
                }
                (Value::Float(a), Value::Integer(b)) => {
                    if (*b as f64) > *a {
                        max_val = arg;
                    }
                }
                _ => return Err("Arguments to max must be numbers".to_string()),
            }
        }
        Ok(max_val)
    })
}

/// (min a b ...)
pub fn min(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.is_empty() {
            return Err("min requires at least 1 argument".to_string());
        }
        let mut min_val = args[0].clone();
        for arg in args.into_iter().skip(1) {
            match (&min_val, &arg) {
                (Value::Integer(a), Value::Integer(b)) => {
                    if b < a {
                        min_val = arg;
                    }
                }
                (Value::Float(a), Value::Float(b)) => {
                    if b < a {
                        min_val = arg;
                    }
                }
                (Value::Integer(a), Value::Float(b)) => {
                    if *b < (*a as f64) {
                        min_val = arg;
                    }
                }
                (Value::Float(a), Value::Integer(b)) => {
                    if (*b as f64) < *a {
                        min_val = arg;
                    }
                }
                _ => return Err("Arguments to min must be numbers".to_string()),
            }
        }
        Ok(min_val)
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

    #[tokio::test]
    async fn test_max() {
        assert_eq!(
            max(&[Value::Integer(1), Value::Integer(5), Value::Integer(3)]).await,
            Ok(Value::Integer(5))
        );
        assert_eq!(
            max(&[Value::Integer(1), Value::Float(5.5), Value::Integer(3)]).await,
            Ok(Value::Float(5.5))
        );
    }

    #[tokio::test]
    async fn test_min() {
        assert_eq!(
            min(&[Value::Integer(1), Value::Integer(5), Value::Integer(-3)]).await,
            Ok(Value::Integer(-3))
        );
        assert_eq!(
            min(&[Value::Float(1.1), Value::Float(5.5), Value::Float(0.5)]).await,
            Ok(Value::Float(0.5))
        );
    }
}
