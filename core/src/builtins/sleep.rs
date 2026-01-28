use crate::ast::Value;

use tokio::time::{Duration, sleep};

pub fn sleep_fn(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("sleep requires exactly one argument (milliseconds)".to_string());
        }

        let ms = match args[0] {
            Value::Integer(n) => n,
            _ => return Err("sleep argument must be an integer".to_string()),
        };

        if ms < 0 {
            return Err("sleep duration cannot be negative".to_string());
        }

        sleep(Duration::from_millis(ms as u64)).await;
        Ok(Value::Nil)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_sleep() {
        let start = Instant::now();
        let args = vec![Value::Integer(100)];
        let _ = sleep_fn(&args).await;
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() >= 100);
    }
}
