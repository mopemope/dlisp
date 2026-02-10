use crate::ast::Value;

/// (str args...)
/// Converts all arguments to strings and concatenates them.
pub fn str_fn(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        let mut result = std::string::String::new();
        for arg in &args {
            match arg {
                Value::String(s) => result.push_str(s),
                other => result.push_str(&format!("{}", other)),
            }
        }
        Ok(Value::String(result))
    })
}

/// (string-length s)
/// Returns the length of a string in characters.
pub fn string_length(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("string-length requires exactly 1 argument".to_string());
        }
        match &args[0] {
            Value::String(s) => Ok(Value::Integer(s.chars().count() as i64)),
            _ => Err("string-length requires a string argument".to_string()),
        }
    })
}

/// (substring s start end)
/// Returns a substring from start (inclusive) to end (exclusive).
pub fn substring(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 3 {
            return Err("substring requires exactly 3 arguments (string start end)".to_string());
        }
        let s = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("substring first argument must be a string".to_string()),
        };
        let start = match &args[1] {
            Value::Integer(i) if *i >= 0 => *i as usize,
            Value::Integer(_) => return Err("substring start must be non-negative".to_string()),
            _ => return Err("substring start must be an integer".to_string()),
        };
        let end = match &args[2] {
            Value::Integer(i) if *i >= 0 => *i as usize,
            Value::Integer(_) => return Err("substring end must be non-negative".to_string()),
            _ => return Err("substring end must be an integer".to_string()),
        };

        let chars: Vec<char> = s.chars().collect();
        if start > chars.len() || end > chars.len() || start > end {
            return Err(format!(
                "substring index out of bounds: start={}, end={}, length={}",
                start,
                end,
                chars.len()
            ));
        }
        let result: std::string::String = chars[start..end].iter().collect();
        Ok(Value::String(result))
    })
}

/// (string-append s1 s2 ...)
/// Concatenates strings.
pub fn string_append(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        let mut result = std::string::String::new();
        for arg in &args {
            match arg {
                Value::String(s) => result.push_str(s),
                _ => return Err("string-append requires string arguments".to_string()),
            }
        }
        Ok(Value::String(result))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_str_fn() {
        assert_eq!(
            str_fn(&[Value::Integer(42)]).await,
            Ok(Value::String("42".to_string()))
        );
        assert_eq!(
            str_fn(&[Value::String("hello ".to_string()), Value::Integer(42)]).await,
            Ok(Value::String("hello 42".to_string()))
        );
    }

    #[tokio::test]
    async fn test_string_length() {
        assert_eq!(
            string_length(&[Value::String("hello".to_string())]).await,
            Ok(Value::Integer(5))
        );
        assert_eq!(
            string_length(&[Value::String("".to_string())]).await,
            Ok(Value::Integer(0))
        );
    }

    #[tokio::test]
    async fn test_substring() {
        assert_eq!(
            substring(&[
                Value::String("hello".to_string()),
                Value::Integer(1),
                Value::Integer(3)
            ])
            .await,
            Ok(Value::String("el".to_string()))
        );
    }

    #[tokio::test]
    async fn test_string_append() {
        assert_eq!(
            string_append(&[
                Value::String("foo".to_string()),
                Value::String("bar".to_string())
            ])
            .await,
            Ok(Value::String("foobar".to_string()))
        );
    }
}
