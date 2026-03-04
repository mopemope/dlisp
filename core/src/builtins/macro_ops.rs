use crate::ast::Value;
use futures::future::LocalBoxFuture;
use std::sync::atomic::{AtomicUsize, Ordering};

static GENSYM_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// (gensym) or (gensym prefix)
/// Generates a unique symbol. The optional prefix must be a string or symbol.
/// Returns an error if more than one argument is passed, or if the argument
/// is not a string or symbol.
pub fn gensym(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() > 1 {
        return Box::pin(async { Err("gensym expects 0 or 1 arguments".to_string()) });
    }

    let prefix = if args.len() == 1 {
        match &args[0] {
            Value::String(s) => s.clone(),
            Value::Symbol(s) => s.clone(),
            other => {
                let msg = format!("gensym prefix must be a string or symbol, got: {}", other);
                return Box::pin(async move { Err(msg) });
            }
        }
    } else {
        "G__".to_string()
    };

    let count = GENSYM_COUNTER.fetch_add(1, Ordering::SeqCst);
    let sym = format!("{}{}", prefix, count);

    Box::pin(async move { Ok(Value::Symbol(sym)) })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: std::future::Future<Output = Result<Value, String>>>(f: F) -> Result<Value, String> {
        tokio::runtime::Runtime::new().unwrap().block_on(f)
    }

    #[test]
    fn test_gensym_default_prefix() {
        let result = run(gensym(&[])).unwrap();
        if let Value::Symbol(s) = result {
            assert!(s.starts_with("G__"));
        } else {
            panic!("Expected Symbol, got {:?}", result);
        }
    }

    #[test]
    fn test_gensym_custom_string_prefix() {
        let result = run(gensym(&[Value::String("MY-".to_string())])).unwrap();
        if let Value::Symbol(s) = result {
            assert!(s.starts_with("MY-"));
        } else {
            panic!("Expected Symbol");
        }
    }

    #[test]
    fn test_gensym_custom_symbol_prefix() {
        let result = run(gensym(&[Value::Symbol("SYM-".to_string())])).unwrap();
        if let Value::Symbol(s) = result {
            assert!(s.starts_with("SYM-"));
        } else {
            panic!("Expected Symbol");
        }
    }

    #[test]
    fn test_gensym_uniqueness() {
        let a = run(gensym(&[])).unwrap();
        let b = run(gensym(&[])).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn test_gensym_too_many_args() {
        let result = run(gensym(&[
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ]));
        assert!(result.is_err());
    }

    #[test]
    fn test_gensym_invalid_prefix_type() {
        let result = run(gensym(&[Value::Integer(42)]));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("must be a string or symbol"));
    }
}
