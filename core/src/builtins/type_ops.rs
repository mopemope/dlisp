use crate::ast::Value;

/// (nil? x)
pub fn is_nil(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("nil? requires exactly 1 argument".to_string());
        }
        Ok(Value::Bool(matches!(args[0], Value::Nil)))
    })
}

/// (list? x)
pub fn is_list(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("list? requires exactly 1 argument".to_string());
        }
        Ok(Value::Bool(matches!(args[0], Value::List(_) | Value::Nil)))
    })
}

/// (number? x)
pub fn is_number(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("number? requires exactly 1 argument".to_string());
        }
        Ok(Value::Bool(matches!(
            args[0],
            Value::Integer(_) | Value::Float(_)
        )))
    })
}

/// (string? x)
pub fn is_string(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("string? requires exactly 1 argument".to_string());
        }
        Ok(Value::Bool(matches!(args[0], Value::String(_))))
    })
}

/// (symbol? x)
pub fn is_symbol(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("symbol? requires exactly 1 argument".to_string());
        }
        Ok(Value::Bool(matches!(args[0], Value::Symbol(_))))
    })
}

/// (keyword? x)
pub fn is_keyword(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("keyword? requires exactly 1 argument".to_string());
        }
        Ok(Value::Bool(matches!(args[0], Value::Keyword(_))))
    })
}

/// (vector? x)
pub fn is_vector(
    args: &[Value],
) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("vector? requires exactly 1 argument".to_string());
        }
        Ok(Value::Bool(matches!(args[0], Value::Vector(_))))
    })
}

/// (map? x)
pub fn is_map(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("map? requires exactly 1 argument".to_string());
        }
        Ok(Value::Bool(matches!(args[0], Value::Map(_))))
    })
}

/// (type-of x)
/// Returns a string describing the type of x.
pub fn type_of(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("type-of requires exactly 1 argument".to_string());
        }
        let type_name = match &args[0] {
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::Symbol(_) => "symbol",
            Value::Keyword(_) => "keyword",
            Value::String(_) => "string",
            Value::Error(_) => "error",
            Value::NativeFunc(_) => "function",
            Value::UserFunc { .. } => "function",
            Value::Macro { .. } => "macro",
            Value::List(_) => "list",
            Value::Vector(_) => "vector",
            Value::Map(_) => "map",
            Value::Nil => "nil",
        };
        Ok(Value::String(type_name.to_string()))
    })
}

/// (error? x)
pub fn is_error(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("error? requires exactly 1 argument".to_string());
        }
        Ok(Value::Bool(matches!(args[0], Value::Error(_))))
    })
}

/// (empty? x)
/// Returns true if the collection or string is empty, false otherwise.
pub fn is_empty(args: &[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("empty? requires exactly 1 argument".to_string());
        }
        match &args[0] {
            Value::List(l) => Ok(Value::Bool(l.is_empty())),
            Value::Vector(v) => Ok(Value::Bool(v.is_empty())),
            Value::Map(m) => Ok(Value::Bool(m.is_empty())),
            Value::String(s) => Ok(Value::Bool(s.is_empty())),
            Value::Nil => Ok(Value::Bool(true)),
            _ => Err("empty? requires a collection or string".to_string()),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_is_nil() {
        assert_eq!(is_nil(&[Value::Nil]).await, Ok(Value::Bool(true)));
        assert_eq!(is_nil(&[Value::Integer(1)]).await, Ok(Value::Bool(false)));
    }

    #[tokio::test]
    async fn test_is_list() {
        assert_eq!(is_list(&[Value::List(vec![])]).await, Ok(Value::Bool(true)));
        assert_eq!(is_list(&[Value::Nil]).await, Ok(Value::Bool(true)));
        assert_eq!(is_list(&[Value::Integer(1)]).await, Ok(Value::Bool(false)));
    }

    #[tokio::test]
    async fn test_is_number() {
        assert_eq!(is_number(&[Value::Integer(1)]).await, Ok(Value::Bool(true)));
        assert_eq!(is_number(&[Value::Float(1.0)]).await, Ok(Value::Bool(true)));
        assert_eq!(
            is_number(&[Value::String("hi".to_string())]).await,
            Ok(Value::Bool(false))
        );
    }

    #[tokio::test]
    async fn test_type_of() {
        assert_eq!(
            type_of(&[Value::Integer(42)]).await,
            Ok(Value::String("integer".to_string()))
        );
        assert_eq!(
            type_of(&[Value::String("hi".to_string())]).await,
            Ok(Value::String("string".to_string()))
        );
        assert_eq!(
            type_of(&[Value::Nil]).await,
            Ok(Value::String("nil".to_string()))
        );
    }

    #[tokio::test]
    async fn test_is_empty() {
        assert_eq!(is_empty(&[Value::Nil]).await, Ok(Value::Bool(true)));
        assert_eq!(
            is_empty(&[Value::List(vec![])]).await,
            Ok(Value::Bool(true))
        );
        assert_eq!(
            is_empty(&[Value::List(vec![Value::Integer(1)])]).await,
            Ok(Value::Bool(false))
        );
        assert_eq!(
            is_empty(&[Value::String("hi".to_string())]).await,
            Ok(Value::Bool(false))
        );
        assert_eq!(
            is_empty(&[Value::String("".to_string())]).await,
            Ok(Value::Bool(true))
        );
    }

    #[tokio::test]
    async fn test_is_error() {
        assert_eq!(
            is_error(&[Value::Error(Box::new(Value::String("err".to_string())))]).await,
            Ok(Value::Bool(true))
        );
        assert_eq!(is_error(&[Value::Nil]).await, Ok(Value::Bool(false)));
    }
}
