use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use futures::future::LocalBoxFuture;
use std::cell::RefCell;
use std::rc::Rc;

pub fn map_form<'a>(
    interpreter: &'a mut Interpreter,
    args: &'a [Value],
    env: &'a mut Rc<RefCell<Environment>>,
) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
    Box::pin(async move {
        if args.len() != 2 {
            return Err("map requires exactly 2 arguments: (map func list)".to_string());
        }

        let func = interpreter.eval(args[0].clone(), env).await?;
        let list_val = interpreter.eval(args[1].clone(), env).await?;

        match list_val {
            Value::List(list) => {
                let mut result = Vec::with_capacity(list.len());
                for item in list {
                    let val = interpreter.apply(func.clone(), vec![item], env).await?;
                    result.push(val);
                }
                Ok(Some(Value::List(result)))
            }
            Value::Vector(list) => {
                let mut result = Vec::with_capacity(list.len());
                for item in list {
                    let val = interpreter.apply(func.clone(), vec![item], env).await?;
                    result.push(val);
                }
                Ok(Some(Value::Vector(result)))
            }
            Value::Nil => Ok(Some(Value::Nil)),
            val => Err(format!(
                "map expects a list or vector as the second argument, got: {}",
                val
            )),
        }
    })
}

pub fn filter_form<'a>(
    interpreter: &'a mut Interpreter,
    args: &'a [Value],
    env: &'a mut Rc<RefCell<Environment>>,
) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
    Box::pin(async move {
        if args.len() != 2 {
            return Err("filter requires exactly 2 arguments: (filter func list)".to_string());
        }

        let func = interpreter.eval(args[0].clone(), env).await?;
        let list_val = interpreter.eval(args[1].clone(), env).await?;

        match list_val {
            Value::List(list) => {
                let mut result = Vec::with_capacity(list.len());
                for item in list {
                    let val = interpreter
                        .apply(func.clone(), vec![item.clone()], env)
                        .await?;
                    if val.is_truthy() {
                        result.push(item);
                    }
                }
                Ok(Some(Value::List(result)))
            }
            Value::Vector(list) => {
                let mut result = Vec::with_capacity(list.len());
                for item in list {
                    let val = interpreter
                        .apply(func.clone(), vec![item.clone()], env)
                        .await?;
                    if val.is_truthy() {
                        result.push(item);
                    }
                }
                Ok(Some(Value::Vector(result)))
            }
            Value::Nil => Ok(Some(Value::Nil)),
            val => Err(format!(
                "filter expects a list or vector as the second argument, got: {}",
                val
            )),
        }
    })
}

pub fn reduce_form<'a>(
    interpreter: &'a mut Interpreter,
    args: &'a [Value],
    env: &'a mut Rc<RefCell<Environment>>,
) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
    Box::pin(async move {
        if args.len() != 3 {
            return Err("reduce requires exactly 3 arguments: (reduce func init list)".to_string());
        }

        let func = interpreter.eval(args[0].clone(), env).await?;
        let mut acc = interpreter.eval(args[1].clone(), env).await?;
        let list_val = interpreter.eval(args[2].clone(), env).await?;

        match list_val {
            Value::List(list) | Value::Vector(list) => {
                for item in list {
                    acc = interpreter
                        .apply(func.clone(), vec![acc, item], env)
                        .await?;
                }
                Ok(Some(acc))
            }
            Value::Nil => Ok(Some(acc)),
            val => Err(format!(
                "reduce expects a list or vector as the third argument, got: {}",
                val
            )),
        }
    })
}

pub fn some_form<'a>(
    interpreter: &'a mut Interpreter,
    args: &'a [Value],
    env: &'a mut Rc<RefCell<Environment>>,
) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
    Box::pin(async move {
        if args.len() != 2 {
            return Err("some requires exactly 2 arguments: (some func list)".to_string());
        }

        let func = interpreter.eval(args[0].clone(), env).await?;
        let list_val = interpreter.eval(args[1].clone(), env).await?;

        match list_val {
            Value::List(list) | Value::Vector(list) => {
                for item in list {
                    let val = interpreter.apply(func.clone(), vec![item], env).await?;
                    if val.is_truthy() {
                        return Ok(Some(val));
                    }
                }
                Ok(Some(Value::Nil))
            }
            Value::Nil => Ok(Some(Value::Nil)),
            val => Err(format!(
                "some expects a list or vector as the second argument, got: {}",
                val
            )),
        }
    })
}

pub fn every_form<'a>(
    interpreter: &'a mut Interpreter,
    args: &'a [Value],
    env: &'a mut Rc<RefCell<Environment>>,
) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
    Box::pin(async move {
        if args.len() != 2 {
            return Err("every requires exactly 2 arguments: (every func list)".to_string());
        }

        let func = interpreter.eval(args[0].clone(), env).await?;
        let list_val = interpreter.eval(args[1].clone(), env).await?;

        match list_val {
            Value::List(list) | Value::Vector(list) => {
                for item in list {
                    let val = interpreter.apply(func.clone(), vec![item], env).await?;
                    if !val.is_truthy() {
                        return Ok(Some(Value::Bool(false)));
                    }
                }
                Ok(Some(Value::Bool(true)))
            }
            Value::Nil => Ok(Some(Value::Bool(true))),
            val => Err(format!(
                "every expects a list or vector as the second argument, got: {}",
                val
            )),
        }
    })
}

pub fn find_form<'a>(
    interpreter: &'a mut Interpreter,
    args: &'a [Value],
    env: &'a mut Rc<RefCell<Environment>>,
) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
    Box::pin(async move {
        if args.len() != 2 {
            return Err("find requires exactly 2 arguments: (find func list)".to_string());
        }

        let func = interpreter.eval(args[0].clone(), env).await?;
        let list_val = interpreter.eval(args[1].clone(), env).await?;

        match list_val {
            Value::List(list) | Value::Vector(list) => {
                for item in list {
                    let val = interpreter
                        .apply(func.clone(), vec![item.clone()], env)
                        .await?;
                    if val.is_truthy() {
                        return Ok(Some(item));
                    }
                }
                Ok(Some(Value::Nil))
            }
            Value::Nil => Ok(Some(Value::Nil)),
            val => Err(format!(
                "find expects a list or vector as the second argument, got: {}",
                val
            )),
        }
    })
}

pub fn for_each_form<'a>(
    interpreter: &'a mut Interpreter,
    args: &'a [Value],
    env: &'a mut Rc<RefCell<Environment>>,
) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
    Box::pin(async move {
        if args.len() != 2 {
            return Err("for-each requires exactly 2 arguments: (for-each func list)".to_string());
        }

        let func = interpreter.eval(args[0].clone(), env).await?;
        let list_val = interpreter.eval(args[1].clone(), env).await?;

        match list_val {
            Value::List(list) | Value::Vector(list) => {
                for item in list {
                    interpreter.apply(func.clone(), vec![item], env).await?;
                }
                Ok(Some(Value::Nil))
            }
            Value::Nil => Ok(Some(Value::Nil)),
            val => Err(format!(
                "for-each expects a list or vector as the second argument, got: {}",
                val
            )),
        }
    })
}

pub fn map_indexed_form<'a>(
    interpreter: &'a mut Interpreter,
    args: &'a [Value],
    env: &'a mut Rc<RefCell<Environment>>,
) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
    Box::pin(async move {
        if args.len() != 2 {
            return Err(
                "map-indexed requires exactly 2 arguments: (map-indexed func list)".to_string(),
            );
        }

        let func = interpreter.eval(args[0].clone(), env).await?;
        let list_val = interpreter.eval(args[1].clone(), env).await?;

        match list_val {
            Value::List(list) => {
                let mut result = Vec::with_capacity(list.len());
                for (i, item) in list.into_iter().enumerate() {
                    let val = interpreter
                        .apply(func.clone(), vec![Value::Integer(i as i64), item], env)
                        .await?;
                    result.push(val);
                }
                Ok(Some(Value::List(result)))
            }
            Value::Vector(list) => {
                let mut result = Vec::with_capacity(list.len());
                for (i, item) in list.into_iter().enumerate() {
                    let val = interpreter
                        .apply(func.clone(), vec![Value::Integer(i as i64), item], env)
                        .await?;
                    result.push(val);
                }
                Ok(Some(Value::Vector(result)))
            }
            Value::Nil => Ok(Some(Value::Nil)),
            val => Err(format!(
                "map-indexed expects a list or vector as the second argument, got: {}",
                val
            )),
        }
    })
}

// Map higher-order functions

pub fn update_form<'a>(
    interpreter: &'a mut Interpreter,
    args: &'a [Value],
    env: &'a mut Rc<RefCell<Environment>>,
) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
    Box::pin(async move {
        if args.len() < 3 {
            return Err(
                "update requires at least 3 arguments: (update map key func arg1 ...)".to_string(),
            );
        }

        let map_val = interpreter.eval(args[0].clone(), env).await?;
        let key = interpreter.eval(args[1].clone(), env).await?;
        let func = interpreter.eval(args[2].clone(), env).await?;

        // Build func_args: [old_value, extra_arg1, extra_arg2, ...]
        let old_val = match &map_val {
            Value::Map(m) => m.get(&key).cloned().unwrap_or(Value::Nil),
            Value::Nil => Value::Nil,
            _ => return Err("update first argument must be a map or nil".to_string()),
        };
        let mut func_args = vec![old_val];

        for arg in &args[3..] {
            func_args.push(interpreter.eval(arg.clone(), env).await?);
        }

        let new_val = interpreter.apply(func, func_args, env).await?;

        match map_val {
            Value::Map(mut m) => {
                m.insert(key, new_val);
                Ok(Some(Value::Map(m)))
            }
            Value::Nil => {
                let mut m = std::collections::HashMap::new();
                m.insert(key, new_val);
                Ok(Some(Value::Map(m)))
            }
            _ => unreachable!(),
        }
    })
}

pub fn map_keys_form<'a>(
    interpreter: &'a mut Interpreter,
    args: &'a [Value],
    env: &'a mut Rc<RefCell<Environment>>,
) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
    Box::pin(async move {
        if args.len() != 2 {
            return Err("map-keys requires exactly 2 arguments: (map-keys func map)".to_string());
        }

        let func = interpreter.eval(args[0].clone(), env).await?;
        let map_val = interpreter.eval(args[1].clone(), env).await?;

        match map_val {
            Value::Map(m) => {
                let mut new_m = std::collections::HashMap::new();
                for (k, v) in m {
                    let new_k = interpreter.apply(func.clone(), vec![k], env).await?;
                    new_m.insert(new_k, v);
                }
                Ok(Some(Value::Map(new_m)))
            }
            Value::Nil => Ok(Some(Value::Nil)),
            val => Err(format!(
                "map-keys expects a map as the second argument, got: {}",
                val
            )),
        }
    })
}

pub fn map_vals_form<'a>(
    interpreter: &'a mut Interpreter,
    args: &'a [Value],
    env: &'a mut Rc<RefCell<Environment>>,
) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
    Box::pin(async move {
        if args.len() != 2 {
            return Err("map-vals requires exactly 2 arguments: (map-vals func map)".to_string());
        }

        let func = interpreter.eval(args[0].clone(), env).await?;
        let map_val = interpreter.eval(args[1].clone(), env).await?;

        match map_val {
            Value::Map(m) => {
                let mut new_m = std::collections::HashMap::new();
                for (k, v) in m {
                    let new_v = interpreter.apply(func.clone(), vec![v], env).await?;
                    new_m.insert(k, new_v);
                }
                Ok(Some(Value::Map(new_m)))
            }
            Value::Nil => Ok(Some(Value::Nil)),
            val => Err(format!(
                "map-vals expects a map as the second argument, got: {}",
                val
            )),
        }
    })
}
