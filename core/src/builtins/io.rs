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

pub fn file_exists(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("file-exists? requires exactly 1 argument (path)".to_string());
        }

        let path = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("file-exists? argument must be a string (path)".to_string()),
        };

        match fs::try_exists(&path).await {
            Ok(exists) => Ok(Value::Bool(exists)),
            Err(e) => Err(format!("file-exists? error: {}", e)),
        }
    })
}

pub fn is_dir(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("is-dir? requires exactly 1 argument (path)".to_string());
        }

        let path = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("is-dir? argument must be a string (path)".to_string()),
        };

        match fs::metadata(&path).await {
            Ok(metadata) => Ok(Value::Bool(metadata.is_dir())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Bool(false)),
            Err(e) => Err(format!("is-dir? error: {}", e)),
        }
    })
}

pub fn is_file(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("is-file? requires exactly 1 argument (path)".to_string());
        }

        let path = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("is-file? argument must be a string (path)".to_string()),
        };

        match fs::metadata(&path).await {
            Ok(metadata) => Ok(Value::Bool(metadata.is_file())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Bool(false)),
            Err(e) => Err(format!("is-file? error: {}", e)),
        }
    })
}

pub fn delete_file(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("delete-file requires exactly 1 argument (path)".to_string());
        }

        let path = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("delete-file argument must be a string (path)".to_string()),
        };

        match fs::metadata(&path).await {
            Ok(metadata) => {
                let res = if metadata.is_dir() {
                    fs::remove_dir_all(&path).await
                } else {
                    fs::remove_file(&path).await
                };
                match res {
                    Ok(_) => Ok(Value::Bool(true)),
                    Err(e) => Err(format!("delete-file error: {}", e)),
                }
            }
            // If it doesn't exist, return false or nil to indicate it wasn't there
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Bool(false)),
            Err(e) => Err(format!("delete-file error: {}", e)),
        }
    })
}

pub fn list_dir(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("list-dir requires exactly 1 argument (path)".to_string());
        }

        let path = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("list-dir argument must be a string (path)".to_string()),
        };

        match fs::read_dir(&path).await {
            Ok(mut entries) => {
                let mut results = Vec::new();
                loop {
                    match entries.next_entry().await {
                        Ok(Some(entry)) => {
                            if let Ok(name) = entry.file_name().into_string() {
                                results.push(Value::String(name));
                            }
                        }
                        Ok(None) => break,
                        Err(e) => return Err(format!("list-dir error reading entry: {}", e)),
                    }
                }
                Ok(Value::List(results))
            }
            Err(e) => Err(format!("list-dir error for path '{}': {}", path, e)),
        }
    })
}
