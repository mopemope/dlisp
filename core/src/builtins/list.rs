use crate::ast::Value;
use futures::future::LocalBoxFuture;

pub fn list(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args_vec = args.to_vec();
    Box::pin(async move { Ok(Value::List(args_vec)) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Value;

    #[tokio::test]
    async fn test_list_empty() {
        let args = vec![];
        let result = list(&args).await.unwrap();
        assert_eq!(result, Value::List(vec![]));
    }

    #[tokio::test]
    async fn test_list_elements() {
        let args = vec![Value::Integer(1), Value::Integer(2)];
        let result = list(&args).await.unwrap();
        assert_eq!(
            result,
            Value::List(vec![Value::Integer(1), Value::Integer(2)])
        );
    }
}
