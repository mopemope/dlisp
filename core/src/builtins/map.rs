use crate::ast::Value;
use futures::future::{self, LocalBoxFuture};
use std::collections::HashMap;

pub fn hash_map(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if !args.len().is_multiple_of(2) {
        return Box::pin(future::ready(Err(
            "hash-map requires an even number of arguments".to_string(),
        )));
    }
    let mut m = HashMap::new();
    for chunk in args.chunks(2) {
        m.insert(chunk[0].clone(), chunk[1].clone());
    }
    Box::pin(future::ready(Ok(Value::Map(m))))
}

pub fn get(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() < 2 || args.len() > 3 {
        return Box::pin(future::ready(Err(
            "get requires 2 or 3 arguments".to_string()
        )));
    }
    match &args[0] {
        Value::Map(m) => {
            let key = &args[1];
            let default = if args.len() == 3 {
                &args[2]
            } else {
                &Value::Nil
            };
            let res = m.get(key).unwrap_or(default).clone();
            Box::pin(future::ready(Ok(res)))
        }
        Value::Vector(v) => {
            let default = if args.len() == 3 {
                &args[2]
            } else {
                &Value::Nil
            };
            if let Value::Integer(i) = args[1]
                && i >= 0
                && i < v.len() as i64
            {
                return Box::pin(future::ready(Ok(v[i as usize].clone())));
            }
            Box::pin(future::ready(Ok(default.clone())))
        }
        Value::Nil => {
            let default = if args.len() == 3 {
                &args[2]
            } else {
                &Value::Nil
            };
            Box::pin(future::ready(Ok(default.clone())))
        }
        _ => Box::pin(future::ready(Ok(Value::Nil))),
    }
}

pub fn assoc(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() < 3 || !(args.len() - 1).is_multiple_of(2) {
        return Box::pin(future::ready(Err(
            "assoc requires a map and even number of key/value pairs".to_string(),
        )));
    }
    match &args[0] {
        Value::Map(m) => {
            let mut new_m = m.clone();
            for chunk in args[1..].chunks(2) {
                new_m.insert(chunk[0].clone(), chunk[1].clone());
            }
            Box::pin(future::ready(Ok(Value::Map(new_m))))
        }
        Value::Nil => {
            let mut new_m = HashMap::new();
            for chunk in args[1..].chunks(2) {
                new_m.insert(chunk[0].clone(), chunk[1].clone());
            }
            Box::pin(future::ready(Ok(Value::Map(new_m))))
        }
        _ => Box::pin(future::ready(Err(
            "assoc first arg must be a map or nil".to_string()
        ))),
    }
}

pub fn dissoc(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.is_empty() {
        return Box::pin(future::ready(Err(
            "dissoc requires at least 1 argument".to_string()
        )));
    }
    match &args[0] {
        Value::Map(m) => {
            let mut new_m = m.clone();
            for key in &args[1..] {
                new_m.remove(key);
            }
            Box::pin(future::ready(Ok(Value::Map(new_m))))
        }
        Value::Nil => Box::pin(future::ready(Ok(Value::Nil))),
        _ => Box::pin(future::ready(Ok(args[0].clone()))),
    }
}

pub fn keys(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() != 1 {
        return Box::pin(future::ready(Err("keys requires 1 argument".to_string())));
    }
    match &args[0] {
        Value::Map(m) => {
            let keys: Vec<Value> = m.keys().cloned().collect();
            Box::pin(future::ready(Ok(Value::List(keys))))
        }
        Value::Nil => Box::pin(future::ready(Ok(Value::List(vec![])))),
        _ => Box::pin(future::ready(Err(
            "keys requires a map argument".to_string()
        ))),
    }
}

pub fn vals(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() != 1 {
        return Box::pin(future::ready(Err("vals requires 1 argument".to_string())));
    }
    match &args[0] {
        Value::Map(m) => {
            let vals: Vec<Value> = m.values().cloned().collect();
            Box::pin(future::ready(Ok(Value::List(vals))))
        }
        Value::Nil => Box::pin(future::ready(Ok(Value::List(vec![])))),
        _ => Box::pin(future::ready(Err(
            "vals requires a map argument".to_string()
        ))),
    }
}

pub fn contains_q(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() != 2 {
        return Box::pin(future::ready(Err(
            "contains? requires 2 arguments".to_string()
        )));
    }
    match &args[0] {
        Value::Map(m) => Box::pin(future::ready(Ok(Value::Bool(m.contains_key(&args[1]))))),
        Value::Nil => Box::pin(future::ready(Ok(Value::Bool(false)))),
        Value::Vector(v) => {
            if let Value::Integer(i) = args[1] {
                let valid = i >= 0 && i < v.len() as i64;
                Box::pin(future::ready(Ok(Value::Bool(valid))))
            } else {
                Box::pin(future::ready(Ok(Value::Bool(false))))
            }
        }
        _ => Box::pin(future::ready(Ok(Value::Bool(false)))),
    }
}

pub fn merge(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let mut m = HashMap::new();
    for arg in args {
        match arg {
            Value::Map(map) => {
                for (k, v) in map.iter() {
                    m.insert(k.clone(), v.clone());
                }
            }
            Value::Nil => {} // treat as empty map
            _ => {
                return Box::pin(future::ready(Err(
                    "merge arguments must be maps or nil".to_string()
                )));
            }
        }
    }
    Box::pin(future::ready(Ok(Value::Map(m))))
}

pub fn select_keys(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() != 2 {
        return Box::pin(future::ready(Err(
            "select-keys requires exactly 2 arguments".to_string(),
        )));
    }

    let map = match &args[0] {
        Value::Map(m) => m,
        Value::Nil => return Box::pin(future::ready(Ok(Value::Nil))),
        _ => {
            return Box::pin(future::ready(Err(
                "select-keys first argument must be a map or nil".to_string(),
            )));
        }
    };

    let keys = match &args[1] {
        Value::List(l) | Value::Vector(l) => l,
        Value::Nil => return Box::pin(future::ready(Ok(Value::Map(HashMap::new())))),
        _ => {
            return Box::pin(future::ready(Err(
                "select-keys second argument must be a list, vector or nil".to_string(),
            )));
        }
    };

    let mut new_m = HashMap::new();
    for key in keys {
        if let Some(val) = map.get(key) {
            new_m.insert(key.clone(), val.clone());
        }
    }

    Box::pin(future::ready(Ok(Value::Map(new_m))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: std::future::Future<Output = Result<Value, String>>>(f: F) -> Result<Value, String> {
        tokio::runtime::Runtime::new().unwrap().block_on(f)
    }

    fn make_map(pairs: Vec<(Value, Value)>) -> Value {
        let mut m = HashMap::new();
        for (k, v) in pairs {
            m.insert(k, v);
        }
        Value::Map(m)
    }

    #[test]
    fn test_merge_basic() {
        let m1 = make_map(vec![(Value::Keyword("a".to_string()), Value::Integer(1))]);
        let m2 = make_map(vec![
            (Value::Keyword("b".to_string()), Value::Integer(2)),
            (Value::Keyword("a".to_string()), Value::Integer(10)),
        ]);

        let merged = run(merge(&[m1, m2])).unwrap();

        if let Value::Map(m) = merged {
            assert_eq!(
                m.get(&Value::Keyword("a".to_string())),
                Some(&Value::Integer(10))
            );
            assert_eq!(
                m.get(&Value::Keyword("b".to_string())),
                Some(&Value::Integer(2))
            );
        } else {
            panic!("Expected Map");
        }
    }

    #[test]
    fn test_merge_with_nil() {
        let m1 = make_map(vec![(Value::Keyword("a".to_string()), Value::Integer(1))]);

        let merged = run(merge(&[m1.clone(), Value::Nil])).unwrap();
        assert_eq!(merged, m1);

        assert_eq!(
            run(merge(&[Value::Nil, Value::Nil])).unwrap(),
            Value::Map(HashMap::new())
        );
        assert_eq!(run(merge(&[])).unwrap(), Value::Map(HashMap::new()));
    }

    #[test]
    fn test_merge_wrong_args() {
        let m1 = make_map(vec![(Value::Keyword("a".to_string()), Value::Integer(1))]);
        assert!(run(merge(&[m1, Value::Integer(42)])).is_err());
    }

    #[test]
    fn test_select_keys_basic() {
        let m1 = make_map(vec![
            (Value::Keyword("a".to_string()), Value::Integer(1)),
            (Value::Keyword("b".to_string()), Value::Integer(2)),
            (Value::Keyword("c".to_string()), Value::Integer(3)),
        ]);
        let keys = Value::List(vec![
            Value::Keyword("a".to_string()),
            Value::Keyword("c".to_string()),
        ]);

        let selected = run(select_keys(&[m1, keys])).unwrap();
        if let Value::Map(m) = selected {
            assert_eq!(m.len(), 2);
            assert_eq!(
                m.get(&Value::Keyword("a".to_string())),
                Some(&Value::Integer(1))
            );
            assert_eq!(
                m.get(&Value::Keyword("c".to_string())),
                Some(&Value::Integer(3))
            );
            assert_eq!(m.get(&Value::Keyword("b".to_string())), None);
        } else {
            panic!("Expected Map");
        }
    }

    #[test]
    fn test_select_keys_missing_keys() {
        let m1 = make_map(vec![(Value::Keyword("a".to_string()), Value::Integer(1))]);
        let keys = Value::List(vec![
            Value::Keyword("a".to_string()),
            Value::Keyword("x".to_string()),
        ]);

        let selected = run(select_keys(&[m1, keys])).unwrap();
        if let Value::Map(m) = selected {
            assert_eq!(m.len(), 1);
            assert_eq!(
                m.get(&Value::Keyword("a".to_string())),
                Some(&Value::Integer(1))
            );
        } else {
            panic!("Expected Map");
        }
    }

    #[test]
    fn test_select_keys_nil_args() {
        let keys = Value::List(vec![Value::Keyword("a".to_string())]);
        assert_eq!(run(select_keys(&[Value::Nil, keys])).unwrap(), Value::Nil);

        let m1 = make_map(vec![(Value::Keyword("a".to_string()), Value::Integer(1))]);
        assert_eq!(
            run(select_keys(&[m1, Value::Nil])).unwrap(),
            Value::Map(HashMap::new())
        );
    }

    #[test]
    fn test_select_keys_wrong_args() {
        assert!(run(select_keys(&[Value::Integer(42), Value::List(vec![])])).is_err());
        let m1 = make_map(vec![]);
        assert!(run(select_keys(&[m1, Value::Integer(42)])).is_err());
    }
}
