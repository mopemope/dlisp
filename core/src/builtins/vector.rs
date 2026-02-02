use crate::ast::Value;
use futures::future::LocalBoxFuture;

pub fn vector(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args_vec = args.to_vec();
    Box::pin(async move { Ok(Value::Vector(args_vec)) })
}

pub fn count(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() != 1 {
        return Box::pin(async move { Err("count requires exactly 1 argument".to_string()) });
    }
    let arg = args[0].clone();
    Box::pin(async move {
        match arg {
            Value::List(l) => Ok(Value::Integer(l.len() as i64)),
            Value::Vector(v) => Ok(Value::Integer(v.len() as i64)),
            Value::Nil => Ok(Value::Integer(0)),
            _ => Err(format!("count: expected collection, got {}", arg)),
        }
    })
}

pub fn nth(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() != 2 {
        return Box::pin(async move { Err("nth requires exactly 2 arguments".to_string()) });
    }
    let col = args[0].clone();
    let idx_val = args[1].clone();
    Box::pin(async move {
        let index = match idx_val {
            Value::Integer(i) => {
                if i < 0 {
                    return Err("nth: index cannot be negative".to_string());
                }
                i as usize
            }
            _ => return Err(format!("nth: index must be integer, got {}", idx_val)),
        };

        match col {
            Value::List(l) => {
                if index < l.len() {
                    Ok(l[index].clone())
                } else {
                    Ok(Value::Nil)
                }
            }
            Value::Vector(v) => {
                if index < v.len() {
                    Ok(v[index].clone())
                } else {
                    Ok(Value::Nil)
                }
            }
            Value::Nil => Ok(Value::Nil),
            _ => Err(format!("nth: expected collection, got {}", col)),
        }
    })
}

pub fn conj(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() < 2 {
        return Box::pin(async move { Err("conj requires at least 2 arguments".to_string()) });
    }
    let col = args[0].clone();
    let items = args[1..].to_vec();

    Box::pin(async move {
        match col {
            Value::List(l) => {
                // List: conj adds to front (cons)
                // For multiple items, we create a new list with items prepended in reverse order
                // Clojure: (conj '(1 2) 3 4) -> (4 3 1 2)
                let mut new_list = l;
                for item in items {
                    new_list.insert(0, item);
                }
                Ok(Value::List(new_list))
            }
            Value::Vector(v) => {
                // Vector: conj adds to end
                // Clojure: (conj [1 2] 3 4) -> [1 2 3 4]
                let mut new_vec = v;
                for item in items {
                    new_vec.push(item);
                }
                Ok(Value::Vector(new_vec))
            }
            Value::Nil => {
                // Treat nil as empty list
                let mut new_list = Vec::new();
                for item in items {
                    new_list.insert(0, item);
                }
                Ok(Value::List(new_list))
            }
            _ => Err(format!("conj: expected collection, got {}", col)),
        }
    })
}
