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
        _ => Box::pin(future::ready(Ok(Value::List(vec![])))),
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
        _ => Box::pin(future::ready(Ok(Value::List(vec![])))),
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
