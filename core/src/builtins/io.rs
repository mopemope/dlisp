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
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Nil),
            Err(e) => Err(format!("read-file error: {}", e)),
        }
    })
}

/// (write-file path content)
/// Writes `content` string to the file at `path`.
pub fn write_file(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("write-file requires exactly 2 arguments (path content)".to_string());
        }

        let path = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("write-file first argument must be a string (path)".to_string()),
        };

        let content = match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err("write-file second argument must be a string (content)".to_string()),
        };

        match fs::write(&path, content).await {
            Ok(_) => Ok(Value::Bool(true)),
            Err(e) => Err(format!("write-file error: {}", e)),
        }
    })
}
