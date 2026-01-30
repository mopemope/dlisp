use crate::ast::Value;
use futures::future::LocalBoxFuture;
use tokio::fs;

pub fn read_file(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("read-file requires exactly 1 argument".to_string());
        }

        let path = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("read-file requires a string argument".to_string()),
        };

        match fs::read_to_string(&path).await {
            Ok(content) => Ok(Value::String(content)),
            Err(_) => Ok(Value::Nil),
        }
    })
}
