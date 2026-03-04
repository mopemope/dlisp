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

/// (string-split s sep)
/// Splits string `s` using separator `sep` and returns a list of strings.
pub fn string_split(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("string-split requires exactly 2 arguments (string separator)".to_string());
        }
        let s = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("string-split first arg must be a string".to_string()),
        };
        let sep = match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err("string-split second arg must be a string".to_string()),
        };

        let parts = s
            .split(&sep)
            .map(|part| Value::String(part.to_string()))
            .collect::<Vec<_>>();
        Ok(Value::List(parts))
    })
}

/// (string-replace s from to)
/// Replaces all occurrences of `from` with `to` in string `s`.
pub fn string_replace(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 3 {
            return Err("string-replace requires exactly 3 arguments (string from to)".to_string());
        }
        let s = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("string-replace first arg must be a string".to_string()),
        };
        let from = match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err("string-replace second arg must be a string".to_string()),
        };
        let to = match &args[2] {
            Value::String(s) => s.clone(),
            _ => return Err("string-replace third arg must be a string".to_string()),
        };

        Ok(Value::String(s.replace(&from, &to)))
    })
}

/// (string-upper s)
/// Converts a string to uppercase.
pub fn string_upper(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("string-upper requires exactly 1 argument".to_string());
        }
        match &args[0] {
            Value::String(s) => Ok(Value::String(s.to_uppercase())),
            _ => Err("string-upper requires a string argument".to_string()),
        }
    })
}

/// (string-lower s)
/// Converts a string to lowercase.
pub fn string_lower(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("string-lower requires exactly 1 argument".to_string());
        }
        match &args[0] {
            Value::String(s) => Ok(Value::String(s.to_lowercase())),
            _ => Err("string-lower requires a string argument".to_string()),
        }
    })
}

/// (string-trim s)
/// Removes leading and trailing whitespace.
pub fn string_trim(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("string-trim requires exactly 1 argument".to_string());
        }
        match &args[0] {
            Value::String(s) => Ok(Value::String(s.trim().to_string())),
            _ => Err("string-trim requires a string argument".to_string()),
        }
    })
}

/// (string-trim-left s)
/// Removes leading whitespace.
pub fn string_trim_left(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("string-trim-left requires exactly 1 argument".to_string());
        }
        match &args[0] {
            Value::String(s) => Ok(Value::String(s.trim_start().to_string())),
            _ => Err("string-trim-left requires a string argument".to_string()),
        }
    })
}

/// (string-trim-right s)
/// Removes trailing whitespace.
pub fn string_trim_right(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("string-trim-right requires exactly 1 argument".to_string());
        }
        match &args[0] {
            Value::String(s) => Ok(Value::String(s.trim_end().to_string())),
            _ => Err("string-trim-right requires a string argument".to_string()),
        }
    })
}

/// (string-starts-with? s prefix)
/// Returns true if string starts with the given prefix.
pub fn string_starts_with(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("string-starts-with? requires exactly 2 arguments".to_string());
        }
        let s = match &args[0] {
            Value::String(s) => s,
            _ => return Err("string-starts-with? first arg must be a string".to_string()),
        };
        let prefix = match &args[1] {
            Value::String(s) => s,
            _ => return Err("string-starts-with? second arg must be a string".to_string()),
        };
        Ok(Value::Bool(s.starts_with(prefix.as_str())))
    })
}

/// (string-ends-with? s suffix)
/// Returns true if string ends with the given suffix.
pub fn string_ends_with(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("string-ends-with? requires exactly 2 arguments".to_string());
        }
        let s = match &args[0] {
            Value::String(s) => s,
            _ => return Err("string-ends-with? first arg must be a string".to_string()),
        };
        let suffix = match &args[1] {
            Value::String(s) => s,
            _ => return Err("string-ends-with? second arg must be a string".to_string()),
        };
        Ok(Value::Bool(s.ends_with(suffix.as_str())))
    })
}

/// (string-contains? s substr)
/// Returns true if string contains the given substring.
pub fn string_contains(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("string-contains? requires exactly 2 arguments".to_string());
        }
        let s = match &args[0] {
            Value::String(s) => s,
            _ => return Err("string-contains? first arg must be a string".to_string()),
        };
        let substr = match &args[1] {
            Value::String(s) => s,
            _ => return Err("string-contains? second arg must be a string".to_string()),
        };
        Ok(Value::Bool(s.contains(substr.as_str())))
    })
}

/// (string-index-of s substr)
/// Returns the index of the first occurrence of substr in s, or nil if not found.
pub fn string_index_of(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("string-index-of requires exactly 2 arguments".to_string());
        }
        let s = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("string-index-of first arg must be a string".to_string()),
        };
        let substr = match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err("string-index-of second arg must be a string".to_string()),
        };
        match s.find(&substr) {
            Some(idx) => Ok(Value::Integer(idx as i64)),
            None => Ok(Value::Nil),
        }
    })
}

/// (string->number s)
/// Parses a string into a number (integer or float). Returns nil on failure.
pub fn string_to_number(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("string->number requires exactly 1 argument".to_string());
        }
        let s = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("string->number requires a string argument".to_string()),
        };
        if let Ok(i) = s.parse::<i64>() {
            Ok(Value::Integer(i))
        } else if let Ok(f) = s.parse::<f64>() {
            Ok(Value::Float(f))
        } else {
            Ok(Value::Nil)
        }
    })
}

/// (number->string n)
/// Converts a number to its string representation.
pub fn number_to_string(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("number->string requires exactly 1 argument".to_string());
        }
        match &args[0] {
            Value::Integer(n) => Ok(Value::String(n.to_string())),
            Value::Float(n) => Ok(Value::String(n.to_string())),
            _ => Err("number->string requires a number argument".to_string()),
        }
    })
}

/// (char-at s index)
/// Returns the character at the given index as a string, or nil if out of bounds.
pub fn char_at(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("char-at requires exactly 2 arguments (string index)".to_string());
        }
        let s = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("char-at first arg must be a string".to_string()),
        };
        let idx = match &args[1] {
            Value::Integer(i) if *i >= 0 => *i as usize,
            Value::Integer(_) => return Err("char-at index must be non-negative".to_string()),
            _ => return Err("char-at second arg must be an integer".to_string()),
        };
        let chars: Vec<char> = s.chars().collect();
        if idx < chars.len() {
            Ok(Value::String(chars[idx].to_string()))
        } else {
            Ok(Value::Nil)
        }
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

    #[tokio::test]
    async fn test_string_split() {
        assert_eq!(
            string_split(&[
                Value::String("a,b,c".to_string()),
                Value::String(",".to_string())
            ])
            .await,
            Ok(Value::List(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
                Value::String("c".to_string())
            ]))
        );
    }

    #[tokio::test]
    async fn test_string_replace() {
        assert_eq!(
            string_replace(&[
                Value::String("hello world".to_string()),
                Value::String("world".to_string()),
                Value::String("dlisp".to_string())
            ])
            .await,
            Ok(Value::String("hello dlisp".to_string()))
        );
    }

    #[tokio::test]
    async fn test_string_upper_lower() {
        assert_eq!(
            string_upper(&[Value::String("hello".to_string())]).await,
            Ok(Value::String("HELLO".to_string()))
        );
        assert_eq!(
            string_lower(&[Value::String("WORLD".to_string())]).await,
            Ok(Value::String("world".to_string()))
        );
    }

    #[tokio::test]
    async fn test_string_split_no_match() {
        // When separator is not found, the whole string should be the only element.
        assert_eq!(
            string_split(&[
                Value::String("hello".to_string()),
                Value::String(",".to_string())
            ])
            .await,
            Ok(Value::List(vec![Value::String("hello".to_string())]))
        );
    }

    #[tokio::test]
    async fn test_string_split_empty_string() {
        assert_eq!(
            string_split(&[
                Value::String("".to_string()),
                Value::String(",".to_string())
            ])
            .await,
            Ok(Value::List(vec![Value::String("".to_string())]))
        );
    }

    #[tokio::test]
    async fn test_string_replace_no_match() {
        // When the from-substring is not found, the original string should be returned.
        assert_eq!(
            string_replace(&[
                Value::String("hello".to_string()),
                Value::String("xyz".to_string()),
                Value::String("abc".to_string())
            ])
            .await,
            Ok(Value::String("hello".to_string()))
        );
    }

    #[tokio::test]
    async fn test_string_ops_error_paths() {
        assert!(
            string_split(&[Value::Integer(1), Value::String(",".to_string())])
                .await
                .is_err()
        );
        assert!(string_replace(&[Value::Integer(1)]).await.is_err());
        assert!(string_upper(&[Value::Integer(1)]).await.is_err());
        assert!(string_lower(&[]).await.is_err());
    }
}
