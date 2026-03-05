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

/// (append &rest lists)
/// Concatenates multiple lists or vectors into a single list.
pub fn append(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        let mut result_list = Vec::new();
        for arg in args {
            match arg {
                Value::List(l) => result_list.extend(l.into_iter()),
                Value::Vector(v) => result_list.extend(v.into_iter()),
                Value::Nil => {} // treat nil as empty list
                _ => return Err("append requires lists or vectors".to_string()),
            }
        }
        Ok(Value::List(result_list))
    })
}

/// (reverse list)
/// Returns a list with the elements in reverse order.
pub fn reverse(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("reverse requires exactly 1 argument".to_string());
        }
        match &args[0] {
            Value::List(l) => {
                let mut rev = l.clone();
                rev.reverse();
                Ok(Value::List(rev))
            }
            Value::Vector(v) => {
                let mut rev = v.clone();
                rev.reverse();
                Ok(Value::Vector(rev))
            }
            Value::Nil => Ok(Value::Nil),
            _ => Err("reverse requires a list or vector".to_string()),
        }
    })
}

/// (sort list)
/// Sorts a list or vector of comparable elements in ascending order.
pub fn sort(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("sort requires exactly 1 argument".to_string());
        }

        let do_sort = |mut items: Vec<Value>| -> Result<Vec<Value>, String> {
            items.sort_by(|a, b| {
                match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                    (Value::Float(x), Value::Float(y)) => {
                        x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Value::Integer(x), Value::Float(y)) => (*x as f64)
                        .partial_cmp(y)
                        .unwrap_or(std::cmp::Ordering::Equal),
                    (Value::Float(x), Value::Integer(y)) => x
                        .partial_cmp(&(*y as f64))
                        .unwrap_or(std::cmp::Ordering::Equal),
                    (Value::String(x), Value::String(y)) => x.cmp(y),
                    // If types are different or not comparable correctly, fallback to comparing formatted strings
                    _ => format!("{}", a).cmp(&format!("{}", b)),
                }
            });
            Ok(items)
        };

        match &args[0] {
            Value::List(l) => Ok(Value::List(do_sort(l.clone())?)),
            Value::Vector(v) => Ok(Value::Vector(do_sort(v.clone())?)),
            Value::Nil => Ok(Value::Nil),
            _ => Err("sort requires a list or vector".to_string()),
        }
    })
}

/// (last list)
/// Returns the last element of a list or vector, or nil if empty.
pub fn last(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() != 1 {
        return Box::pin(async { Err("last requires exactly 1 argument".to_string()) });
    }
    let result = match &args[0] {
        Value::List(l) => l.last().cloned().unwrap_or(Value::Nil),
        Value::Vector(v) => v.last().cloned().unwrap_or(Value::Nil),
        Value::Nil => Value::Nil,
        _ => return Box::pin(async { Err("last requires a list or vector argument".to_string()) }),
    };
    Box::pin(async move { Ok(result) })
}

/// (butlast list)
/// Returns a list of all elements except the last, or nil if empty.
pub fn butlast(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() != 1 {
        return Box::pin(async { Err("butlast requires exactly 1 argument".to_string()) });
    }
    let result = match &args[0] {
        Value::List(l) => {
            if l.is_empty() {
                Value::Nil
            } else {
                Value::List(l[..l.len() - 1].to_vec())
            }
        }
        Value::Vector(v) => {
            if v.is_empty() {
                Value::Nil
            } else {
                Value::Vector(v[..v.len() - 1].to_vec())
            }
        }
        Value::Nil => Value::Nil,
        _ => {
            return Box::pin(async {
                Err("butlast requires a list or vector argument".to_string())
            });
        }
    };
    Box::pin(async move { Ok(result) })
}

/// (flatten list)
/// Flattens a nested list structure into a single linear list.
pub fn flatten(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() != 1 {
        return Box::pin(async { Err("flatten requires exactly 1 argument".to_string()) });
    }
    fn do_flatten(val: &Value, out: &mut Vec<Value>) {
        match val {
            Value::List(l) | Value::Vector(l) => {
                for item in l {
                    do_flatten(item, out);
                }
            }
            Value::Nil => {}
            atom => out.push(atom.clone()),
        }
    }
    let mut out = Vec::new();
    do_flatten(&args[0], &mut out);
    let result = Value::List(out);
    Box::pin(async move { Ok(result) })
}

/// (range end)
/// (range start end)
/// (range start end step)
pub fn range_fn(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.is_empty() || args.len() > 3 {
            return Err("range requires 1 to 3 arguments".to_string());
        }

        let mut int_args = Vec::new();
        for arg in args {
            match arg {
                Value::Integer(i) => int_args.push(i),
                _ => return Err("range arguments must be integers".to_string()),
            }
        }

        let (start, end, step) = match int_args.len() {
            1 => (0, int_args[0], 1),
            2 => (int_args[0], int_args[1], 1),
            3 => (int_args[0], int_args[1], int_args[2]),
            _ => unreachable!(),
        };

        if step == 0 {
            return Err("range step cannot be zero".to_string());
        }

        let mut res = Vec::new();
        let mut curr = start;
        if step > 0 {
            while curr < end {
                res.push(Value::Integer(curr));
                curr += step;
            }
        } else {
            while curr > end {
                res.push(Value::Integer(curr));
                curr += step;
            }
        }

        Ok(Value::List(res))
    })
}

/// (take n list)
/// Returns a list of the first n elements of the list/vector.
pub fn take(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() != 2 {
        return Box::pin(async { Err("take requires exactly 2 arguments".to_string()) });
    }

    let n = match &args[0] {
        Value::Integer(i) if *i >= 0 => *i as usize,
        Value::Integer(_) => {
            return Box::pin(async {
                Err("take first argument must be a non-negative integer".to_string())
            });
        }
        _ => return Box::pin(async { Err("take first argument must be an integer".to_string()) }),
    };

    let result = match &args[1] {
        Value::List(l) => {
            let take_len = l.len().min(n);
            Value::List(l[..take_len].to_vec())
        }
        Value::Vector(v) => {
            let take_len = v.len().min(n);
            Value::Vector(v[..take_len].to_vec())
        }
        Value::Nil => Value::Nil,
        _ => {
            return Box::pin(async {
                Err("take second argument must be a list or vector".to_string())
            });
        }
    };
    Box::pin(async move { Ok(result) })
}

/// (drop n list)
/// Returns a list of all elements except the first n elements.
pub fn drop_fn(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    if args.len() != 2 {
        return Box::pin(async { Err("drop requires exactly 2 arguments".to_string()) });
    }

    let n = match &args[0] {
        Value::Integer(i) if *i >= 0 => *i as usize,
        Value::Integer(_) => {
            return Box::pin(async {
                Err("drop first argument must be a non-negative integer".to_string())
            });
        }
        _ => return Box::pin(async { Err("drop first argument must be an integer".to_string()) }),
    };

    let result = match &args[1] {
        Value::List(l) => {
            let drop_len = l.len().min(n);
            Value::List(l[drop_len..].to_vec())
        }
        Value::Vector(v) => {
            let drop_len = v.len().min(n);
            Value::Vector(v[drop_len..].to_vec())
        }
        Value::Nil => Value::Nil,
        _ => {
            return Box::pin(async {
                Err("drop second argument must be a list or vector".to_string())
            });
        }
    };
    Box::pin(async move { Ok(result) })
}

/// (zip &rest lists)
/// Returns a list of lists, where the i-th sublist contains the i-th element from each argument list.
pub fn zip(args: &[Value]) -> LocalBoxFuture<'static, Result<Value, String>> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.is_empty() {
            return Ok(Value::List(vec![]));
        }

        let mut all_lists = Vec::new();
        for arg in args {
            match arg {
                Value::List(l) => all_lists.push(l),
                Value::Vector(v) => all_lists.push(v),
                Value::Nil => all_lists.push(vec![]),
                _ => return Err("zip requires all arguments to be lists or vectors".to_string()),
            }
        }

        let min_len = all_lists.iter().map(|l| l.len()).min().unwrap_or(0);
        let mut result = Vec::with_capacity(min_len);

        for i in 0..min_len {
            let mut sublist = Vec::with_capacity(all_lists.len());
            for list in &all_lists {
                sublist.push(list[i].clone());
            }
            result.push(Value::List(sublist));
        }

        Ok(Value::List(result))
    })
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

    #[test]
    fn test_append() {
        let list1 = Value::List(vec![Value::Integer(1), Value::Integer(2)]);
        let list2 = Value::List(vec![Value::Integer(3), Value::Integer(4)]);
        assert_eq!(
            run(append(&[list1, list2])).unwrap(),
            Value::List(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
                Value::Integer(4)
            ])
        );
        assert_eq!(run(append(&[])).unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_reverse() {
        let list = Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        assert_eq!(
            run(reverse(&[list])).unwrap(),
            Value::List(vec![
                Value::Integer(3),
                Value::Integer(2),
                Value::Integer(1)
            ])
        );
    }

    #[test]
    fn test_sort() {
        let list = Value::List(vec![
            Value::Integer(3),
            Value::Integer(1),
            Value::Integer(2),
        ]);
        assert_eq!(
            run(sort(&[list])).unwrap(),
            Value::List(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3)
            ])
        );

        let str_list = Value::List(vec![
            Value::String("c".to_string()),
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ]);
        assert_eq!(
            run(sort(&[str_list])).unwrap(),
            Value::List(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
                Value::String("c".to_string())
            ])
        );
    }

    #[test]
    fn test_last_basic() {
        let list = Value::List(vec![Value::Integer(1), Value::Integer(2)]);
        assert_eq!(run(last(&[list])).unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_last_empty() {
        assert_eq!(run(last(&[Value::List(vec![])])).unwrap(), Value::Nil);
        assert_eq!(run(last(&[Value::Nil])).unwrap(), Value::Nil);
    }

    #[test]
    fn test_butlast_basic() {
        let list = Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        assert_eq!(
            run(butlast(&[list])).unwrap(),
            Value::List(vec![Value::Integer(1), Value::Integer(2)])
        );
    }

    #[test]
    fn test_butlast_empty_or_single() {
        assert_eq!(run(butlast(&[Value::List(vec![])])).unwrap(), Value::Nil);
        let single = Value::List(vec![Value::Integer(1)]);
        assert_eq!(run(butlast(&[single])).unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_flatten_basic() {
        let nested = Value::List(vec![
            Value::Integer(1),
            Value::List(vec![Value::Integer(2), Value::Integer(3)]),
            Value::List(vec![Value::List(vec![Value::Integer(4)])]),
        ]);
        assert_eq!(
            run(flatten(&[nested])).unwrap(),
            Value::List(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
                Value::Integer(4)
            ])
        );
    }

    #[test]
    fn test_range_basic() {
        assert_eq!(
            run(range_fn(&[Value::Integer(3)])).unwrap(),
            Value::List(vec![
                Value::Integer(0),
                Value::Integer(1),
                Value::Integer(2)
            ])
        );
        assert_eq!(
            run(range_fn(&[Value::Integer(1), Value::Integer(4)])).unwrap(),
            Value::List(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3)
            ])
        );
        assert_eq!(
            run(range_fn(&[
                Value::Integer(10),
                Value::Integer(0),
                Value::Integer(-3)
            ]))
            .unwrap(),
            Value::List(vec![
                Value::Integer(10),
                Value::Integer(7),
                Value::Integer(4),
                Value::Integer(1)
            ])
        );
    }

    #[test]
    fn test_take_basic() {
        let list = Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        assert_eq!(
            run(take(&[Value::Integer(2), list.clone()])).unwrap(),
            Value::List(vec![Value::Integer(1), Value::Integer(2)])
        );
        assert_eq!(
            run(take(&[Value::Integer(5), list.clone()])).unwrap(),
            Value::List(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3)
            ])
        );
    }

    #[test]
    fn test_drop_basic() {
        let list = Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        assert_eq!(
            run(drop_fn(&[Value::Integer(2), list.clone()])).unwrap(),
            Value::List(vec![Value::Integer(3)])
        );
        assert_eq!(
            run(drop_fn(&[Value::Integer(5), list.clone()])).unwrap(),
            Value::List(vec![])
        );
    }

    #[test]
    fn test_zip_basic() {
        let l1 = Value::List(vec![Value::Integer(1), Value::Integer(2)]);
        let l2 = Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ]);
        assert_eq!(
            run(zip(&[l1, l2])).unwrap(),
            Value::List(vec![
                Value::List(vec![Value::Integer(1), Value::String("a".to_string())]),
                Value::List(vec![Value::Integer(2), Value::String("b".to_string())])
            ])
        );
    }

    // --- Additional edge-case tests ---

    #[test]
    fn test_last_vector() {
        let v = Value::Vector(vec![Value::Integer(10), Value::Integer(20)]);
        assert_eq!(run(last(&[v])).unwrap(), Value::Integer(20));
    }

    #[test]
    fn test_last_wrong_type() {
        assert!(run(last(&[Value::Integer(42)])).is_err());
    }

    #[test]
    fn test_last_wrong_arity() {
        assert!(run(last(&[])).is_err());
        assert!(run(last(&[Value::Nil, Value::Nil])).is_err());
    }

    #[test]
    fn test_butlast_vector() {
        let v = Value::Vector(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        assert_eq!(
            run(butlast(&[v])).unwrap(),
            Value::Vector(vec![Value::Integer(1), Value::Integer(2)])
        );
    }

    #[test]
    fn test_butlast_wrong_type() {
        assert!(run(butlast(&[Value::String("x".to_string())])).is_err());
    }

    #[test]
    fn test_flatten_empty() {
        assert_eq!(
            run(flatten(&[Value::List(vec![])])).unwrap(),
            Value::List(vec![])
        );
    }

    #[test]
    fn test_flatten_nil() {
        assert_eq!(run(flatten(&[Value::Nil])).unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_flatten_atom() {
        // Flatten on a non-list atom: just wraps it in a list
        assert_eq!(
            run(flatten(&[Value::Integer(42)])).unwrap(),
            Value::List(vec![Value::Integer(42)])
        );
    }

    #[test]
    fn test_flatten_vector() {
        let v = Value::Vector(vec![
            Value::Integer(1),
            Value::Vector(vec![Value::Integer(2)]),
        ]);
        assert_eq!(
            run(flatten(&[v])).unwrap(),
            Value::List(vec![Value::Integer(1), Value::Integer(2)])
        );
    }

    #[test]
    fn test_range_zero() {
        assert_eq!(
            run(range_fn(&[Value::Integer(0)])).unwrap(),
            Value::List(vec![])
        );
    }

    #[test]
    fn test_range_negative_end() {
        // (range -3) → empty because 0 >= -3
        assert_eq!(
            run(range_fn(&[Value::Integer(-3)])).unwrap(),
            Value::List(vec![])
        );
    }

    #[test]
    fn test_range_step_zero_error() {
        assert!(
            run(range_fn(&[
                Value::Integer(0),
                Value::Integer(5),
                Value::Integer(0)
            ]))
            .is_err()
        );
    }

    #[test]
    fn test_range_wrong_type() {
        assert!(run(range_fn(&[Value::String("a".to_string())])).is_err());
    }

    #[test]
    fn test_take_zero() {
        let list = Value::List(vec![Value::Integer(1), Value::Integer(2)]);
        assert_eq!(
            run(take(&[Value::Integer(0), list])).unwrap(),
            Value::List(vec![])
        );
    }

    #[test]
    fn test_take_negative_error() {
        let list = Value::List(vec![Value::Integer(1)]);
        assert!(run(take(&[Value::Integer(-1), list])).is_err());
    }

    #[test]
    fn test_take_nil() {
        assert_eq!(
            run(take(&[Value::Integer(3), Value::Nil])).unwrap(),
            Value::Nil
        );
    }

    #[test]
    fn test_take_vector() {
        let v = Value::Vector(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        assert_eq!(
            run(take(&[Value::Integer(2), v])).unwrap(),
            Value::Vector(vec![Value::Integer(1), Value::Integer(2)])
        );
    }

    #[test]
    fn test_drop_zero() {
        let list = Value::List(vec![Value::Integer(1), Value::Integer(2)]);
        assert_eq!(
            run(drop_fn(&[Value::Integer(0), list])).unwrap(),
            Value::List(vec![Value::Integer(1), Value::Integer(2)])
        );
    }

    #[test]
    fn test_drop_negative_error() {
        let list = Value::List(vec![Value::Integer(1)]);
        assert!(run(drop_fn(&[Value::Integer(-1), list])).is_err());
    }

    #[test]
    fn test_drop_nil() {
        assert_eq!(
            run(drop_fn(&[Value::Integer(3), Value::Nil])).unwrap(),
            Value::Nil
        );
    }

    #[test]
    fn test_drop_vector() {
        let v = Value::Vector(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        assert_eq!(
            run(drop_fn(&[Value::Integer(1), v])).unwrap(),
            Value::Vector(vec![Value::Integer(2), Value::Integer(3)])
        );
    }

    #[test]
    fn test_zip_unequal_lengths() {
        let l1 = Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let l2 = Value::List(vec![Value::Integer(10)]);
        assert_eq!(
            run(zip(&[l1, l2])).unwrap(),
            Value::List(vec![Value::List(vec![
                Value::Integer(1),
                Value::Integer(10)
            ])])
        );
    }

    #[test]
    fn test_zip_empty() {
        assert_eq!(run(zip(&[])).unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_zip_single_list() {
        let l1 = Value::List(vec![Value::Integer(1), Value::Integer(2)]);
        assert_eq!(
            run(zip(&[l1])).unwrap(),
            Value::List(vec![
                Value::List(vec![Value::Integer(1)]),
                Value::List(vec![Value::Integer(2)])
            ])
        );
    }

    #[test]
    fn test_zip_wrong_type() {
        assert!(run(zip(&[Value::Integer(1)])).is_err());
    }

    #[test]
    fn test_zip_with_nil() {
        let l1 = Value::List(vec![Value::Integer(1), Value::Integer(2)]);
        // Nil is treated as empty list, so min_len = 0
        assert_eq!(run(zip(&[l1, Value::Nil])).unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_zip_three_lists() {
        let l1 = Value::List(vec![Value::Integer(1), Value::Integer(2)]);
        let l2 = Value::List(vec![Value::Integer(3), Value::Integer(4)]);
        let l3 = Value::List(vec![Value::Integer(5), Value::Integer(6)]);
        assert_eq!(
            run(zip(&[l1, l2, l3])).unwrap(),
            Value::List(vec![
                Value::List(vec![
                    Value::Integer(1),
                    Value::Integer(3),
                    Value::Integer(5)
                ]),
                Value::List(vec![
                    Value::Integer(2),
                    Value::Integer(4),
                    Value::Integer(6)
                ])
            ])
        );
    }
}
