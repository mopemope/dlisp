use crate::ast::Value;
use futures::future::LocalBoxFuture;

/// (not expr)
/// Returns true if expr is falsy (nil, false, or 0), false otherwise.
pub fn not(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() != 1 {
        return Box::pin(async { Err("not requires exactly 1 argument".to_string()) });
    }
    let result = Value::Bool(!args[0].is_truthy());
    Box::pin(async move { Ok(result) })
}

/// (car list)
/// Returns the first element of a list, or nil if the list is empty.
pub fn car(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() != 1 {
        return Box::pin(async { Err("car requires exactly 1 argument".to_string()) });
    }
    let result = match &args[0] {
        Value::List(l) => {
            if l.is_empty() {
                Value::Nil
            } else {
                l[0].clone()
            }
        }
        Value::Nil => Value::Nil,
        _ => {
            return Box::pin(async { Err("car requires a list argument".to_string()) });
        }
    };
    Box::pin(async move { Ok(result) })
}

/// (cdr list)
/// Returns a list of all elements except the first, or nil if the list is empty.
pub fn cdr(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() != 1 {
        return Box::pin(async { Err("cdr requires exactly 1 argument".to_string()) });
    }
    let result = match &args[0] {
        Value::List(l) => {
            if l.len() <= 1 {
                Value::Nil
            } else {
                Value::List(l[1..].to_vec())
            }
        }
        Value::Nil => Value::Nil,
        _ => {
            return Box::pin(async { Err("cdr requires a list argument".to_string()) });
        }
    };
    Box::pin(async move { Ok(result) })
}

/// (cons element list)
/// Prepends an element to a list. If the second argument is nil, creates a single-element list.
pub fn cons(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() != 2 {
        return Box::pin(async { Err("cons requires exactly 2 arguments".to_string()) });
    }
    let elem = args[0].clone();
    let result = match &args[1] {
        Value::List(l) => {
            let mut new_list = vec![elem];
            new_list.extend(l.iter().cloned());
            Value::List(new_list)
        }
        Value::Nil => Value::List(vec![elem]),
        _ => {
            // Cons pair: create a list of the two elements
            Value::List(vec![elem, args[1].clone()])
        }
    };
    Box::pin(async move { Ok(result) })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: std::future::Future<Output = Result<Value, String>>>(f: F) -> Result<Value, String> {
        tokio::runtime::Runtime::new().unwrap().block_on(f)
    }

    #[test]
    fn test_not_nil() {
        assert_eq!(run(not(&[Value::Nil])).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_not_zero() {
        assert_eq!(run(not(&[Value::Integer(0)])).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_not_truthy() {
        assert_eq!(run(not(&[Value::Integer(42)])).unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_car_basic() {
        let list = Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        assert_eq!(run(car(&[list])).unwrap(), Value::Integer(1));
    }

    #[test]
    fn test_car_empty() {
        assert_eq!(run(car(&[Value::List(vec![])])).unwrap(), Value::Nil);
    }

    #[test]
    fn test_car_nil() {
        assert_eq!(run(car(&[Value::Nil])).unwrap(), Value::Nil);
    }

    #[test]
    fn test_cdr_basic() {
        let list = Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        assert_eq!(
            run(cdr(&[list])).unwrap(),
            Value::List(vec![Value::Integer(2), Value::Integer(3)])
        );
    }

    #[test]
    fn test_cdr_single() {
        let list = Value::List(vec![Value::Integer(1)]);
        assert_eq!(run(cdr(&[list])).unwrap(), Value::Nil);
    }

    #[test]
    fn test_cdr_nil() {
        assert_eq!(run(cdr(&[Value::Nil])).unwrap(), Value::Nil);
    }

    #[test]
    fn test_cons_to_list() {
        let list = Value::List(vec![Value::Integer(2), Value::Integer(3)]);
        assert_eq!(
            run(cons(&[Value::Integer(1), list])).unwrap(),
            Value::List(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3)
            ])
        );
    }

    #[test]
    fn test_cons_to_nil() {
        assert_eq!(
            run(cons(&[Value::Integer(1), Value::Nil])).unwrap(),
            Value::List(vec![Value::Integer(1)])
        );
    }
}
