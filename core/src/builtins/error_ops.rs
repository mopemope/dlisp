use crate::ast::Value;

/// (error-value x)
/// Unwraps the value inside a Value::Error
pub fn error_value(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("error-value requires exactly 1 argument".to_string());
        }
        match &args[0] {
            Value::Error(inner) => Ok(*inner.clone()),
            _ => Err("error-value requires an Error type".to_string()),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_error_value() {
        let err = Value::Error(Box::new(Value::Integer(42)));
        assert_eq!(error_value(&[err]).await, Ok(Value::Integer(42)));

        let not_err = Value::Integer(42);
        assert!(error_value(&[not_err]).await.is_err());
    }
}
