use crate::ast::Value;

pub fn print(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{}", arg);
        }
        println!();
        Ok(Value::Nil)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_print_success() {
        // (print 1 2) => nil
        // We can't easily capture stdout in unit tests without a crate like `subprocess` or redirecting,
        // but we can check the return value.
        let args = vec![Value::Integer(1), Value::Integer(2)];
        let result = print(&args).await;
        assert_eq!(result, Ok(Value::Nil));
    }
}
