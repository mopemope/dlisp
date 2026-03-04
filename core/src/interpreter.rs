use crate::ast::Value;
use crate::environment::Environment;
use async_recursion::async_recursion;
use std::cell::RefCell;
use std::rc::Rc;

use crate::jit::JIT;

pub mod apply;

use crate::forms::registry::{FormRegistry, standard_registry};

pub struct Interpreter {
    pub jit: JIT,
    pub forms: FormRegistry,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            jit: JIT::new(),
            forms: standard_registry(),
        }
    }

    #[async_recursion(?Send)]
    pub async fn eval(
        &mut self,
        val: Value,
        env: &mut Rc<RefCell<Environment>>,
    ) -> Result<Value, String> {
        // Expand macros first
        let expanded = self.expand(val, env).await?;

        match expanded {
            Value::Symbol(s) => env
                .borrow()
                .get(&s)
                .ok_or_else(|| format!("Undefined symbol: {}", s)),
            Value::Keyword(_) => Ok(expanded),
            Value::List(list) => {
                if list.is_empty() {
                    return Ok(Value::Nil);
                }

                // Try special forms first
                #[allow(clippy::collapsible_if)]
                if let Value::Symbol(ref s) = list[0] {
                    if s == "defmacro" {
                        let macro_val = crate::macros::construct_macro(&list[1..])?;
                        let name = match &list[1] {
                            Value::Symbol(n) => n.clone(),
                            _ => unreachable!(),
                        };
                        env.borrow_mut().set(name.clone(), macro_val); // Store macro in env
                        return Ok(Value::Symbol(name));
                    }

                    if let Some(result) = self.eval_special_form(s, &list[1..], env).await? {
                        return Ok(result);
                    }
                }

                // Standard function call
                let func_val = self.eval(list[0].clone(), env).await?;
                let mut args = Vec::new();
                for arg in &list[1..] {
                    args.push(self.eval(arg.clone(), env).await?);
                }

                self.apply(func_val, args, env).await
            }
            Value::Vector(vec) => {
                let mut new_vec = Vec::with_capacity(vec.len());
                for item in vec {
                    new_vec.push(self.eval(item, env).await?);
                }
                Ok(Value::Vector(new_vec))
            }
            Value::Map(map) => {
                let mut new_map = std::collections::HashMap::with_capacity(map.len());
                for (k, v) in map {
                    let k_eval = self.eval(k, env).await?;
                    let v_eval = self.eval(v, env).await?;
                    new_map.insert(k_eval, v_eval);
                }
                Ok(Value::Map(new_map))
            }
            _ => Ok(expanded), // Self-evaluating
        }
    }

    #[async_recursion(?Send)]
    pub async fn expand(
        &mut self,
        val: Value,
        env: &mut Rc<RefCell<Environment>>,
    ) -> Result<Value, String> {
        match val {
            Value::List(ref list) => {
                if list.is_empty() {
                    return Ok(val);
                }

                if let Value::Symbol(ref s) = list[0] {
                    // Check for special forms that need custom expansion handling
                    match s.as_str() {
                        "quote" => return Ok(val),
                        "let" => {
                            // (let bindings body...)
                            // bindings: ((var val) ...)
                            // We must NOT expand the bindings list itself as a macro call, but we MUST expand the 'val's inside it.
                            if list.len() < 2 {
                                return Ok(val); // Malformed, just return
                            }

                            let mut new_list = Vec::new();
                            new_list.push(list[0].clone()); // 'let'

                            // Handle bindings
                            match &list[1] {
                                Value::List(bindings) => {
                                    let mut new_bindings = Vec::new();
                                    for b in bindings {
                                        if let Value::List(pair) = b {
                                            if pair.len() == 2 {
                                                // (var val) -> expand val only
                                                let val_expanded =
                                                    self.expand(pair[1].clone(), env).await?;
                                                new_bindings.push(Value::List(vec![
                                                    pair[0].clone(),
                                                    val_expanded,
                                                ]));
                                            } else {
                                                // Malformed binding, just push as is (or expand recursively if it's weird)
                                                new_bindings
                                                    .push(self.expand(b.clone(), env).await?);
                                            }
                                        } else {
                                            new_bindings.push(b.clone());
                                        }
                                    }
                                    new_list.push(Value::List(new_bindings));
                                }
                                _ => new_list.push(list[1].clone()), // Malformed bindings, keep as is
                            }

                            // Expand body
                            for item in list.iter().skip(2) {
                                new_list.push(self.expand(item.clone(), env).await?);
                            }

                            return Ok(Value::List(new_list));
                        }
                        "defun" | "defmacro" => {
                            // (defun name args body...)
                            // Args list should NOT be expanded
                            if list.len() < 3 {
                                return Ok(val);
                            }
                            let mut new_list = Vec::new();
                            new_list.push(list[0].clone()); // defun
                            new_list.push(list[1].clone()); // name
                            new_list.push(list[2].clone()); // args (unexpanded)

                            // Expand body
                            for item in list.iter().skip(3) {
                                new_list.push(self.expand(item.clone(), env).await?);
                            }
                            return Ok(Value::List(new_list));
                        }
                        "lambda" => {
                            // (lambda args body...)
                            if list.len() < 2 {
                                return Ok(val);
                            }
                            let mut new_list = Vec::new();
                            new_list.push(list[0].clone()); // lambda
                            new_list.push(list[1].clone()); // args (unexpanded)

                            // Expand body
                            for item in list.iter().skip(2) {
                                new_list.push(self.expand(item.clone(), env).await?);
                            }
                            return Ok(Value::List(new_list));
                        }
                        "backquote" => {
                            if list.len() < 2 {
                                return Ok(val);
                            }
                            let expanded = self.expand_backquote(list[1].clone(), env, 1).await?;
                            return self.expand(expanded, env).await;
                        }
                        _ => {} // Fall through to macro check or default expansion
                    }

                    // Check environment for macro definition
                    let resolved_opt = env.borrow().get(s);
                    if let Some(Value::Macro { args, body }) = resolved_opt {
                        // It IS a macro!
                        let macro_args_vals = list[1..].to_vec(); // Unevaluated args

                        if macro_args_vals.len() != args.len() {
                            return Err(format!(
                                "Macro {} expects {} arguments, got {}",
                                s,
                                args.len(),
                                macro_args_vals.len()
                            ));
                        }

                        // Execute macro body
                        // Create macro environment
                        let mut macro_env = Environment::new(Some(env.clone()));
                        for (name, val) in args.iter().zip(macro_args_vals.iter()) {
                            macro_env.set(name.clone(), val.clone());
                        }
                        let mut macro_env_rc = Rc::new(RefCell::new(macro_env));

                        // Eval body
                        let mut result = Value::Nil;
                        for stmt in body {
                            result = self.eval(stmt.clone(), &mut macro_env_rc).await?;
                        }

                        // Recursive expand the RESULT
                        return self.expand(result, env).await;
                    }
                }

                // Recursively expand list elements (default case)
                let mut new_list = Vec::with_capacity(list.len());
                for item in list {
                    new_list.push(self.expand(item.clone(), env).await?);
                }
                Ok(Value::List(new_list))
            }
            Value::Vector(vec) => {
                let mut new_vec = Vec::with_capacity(vec.len());
                for item in vec {
                    new_vec.push(self.expand(item.clone(), env).await?);
                }
                Ok(Value::Vector(new_vec))
            }
            Value::Map(map) => {
                let mut new_map = std::collections::HashMap::with_capacity(map.len());
                for (k, v) in map {
                    let k_expanded = self.expand(k.clone(), env).await?;
                    let v_expanded = self.expand(v.clone(), env).await?;
                    new_map.insert(k_expanded, v_expanded);
                }
                Ok(Value::Map(new_map))
            }
            _ => Ok(val),
        }
    }

    async fn eval_special_form(
        &mut self,
        name: &str,
        args: &[Value],
        env: &mut Rc<RefCell<Environment>>,
    ) -> Result<Option<Value>, String> {
        if let Some(form) = self.forms.get(name) {
            form.call(self, args, env).await
        } else {
            Ok(None)
        }
    }

    #[async_recursion(?Send)]
    pub async fn expand_backquote(
        &mut self,
        val: Value,
        env: &mut Rc<RefCell<Environment>>,
        depth: usize,
    ) -> Result<Value, String> {
        match val {
            Value::List(list) => {
                if list.is_empty() {
                    return Ok(Value::List(vec![
                        Value::Symbol("quote".to_string()),
                        Value::Nil,
                    ]));
                }

                let mut is_bq = false;
                let mut is_uq = false;

                if let Value::Symbol(ref s) = list[0] {
                    if s == "unquote" {
                        if depth == 1 {
                            if list.len() < 2 {
                                return Err("unquote expects 1 argument".to_string());
                            }
                            return self.expand(list[1].clone(), env).await;
                        }
                        is_uq = true;
                    } else if s == "unquote-splicing" {
                        if depth == 1 {
                            return Err(
                                "unquote-splicing invalid at top-level of backquote".to_string()
                            );
                        }
                        is_uq = true;
                    } else if s == "backquote" {
                        is_bq = true;
                    }
                }

                let mut append_args = Vec::new();
                let mut current_list = Vec::new();

                for (i, item) in list.iter().enumerate() {
                    let mut child_depth = depth;
                    if i > 0 {
                        if is_bq {
                            child_depth += 1;
                        } else if is_uq {
                            child_depth -= 1;
                        }
                    }

                    if depth == 1
                        && let Value::List(inner) = item
                        && !inner.is_empty()
                        && matches!(&inner[0], Value::Symbol(s) if s == "unquote-splicing")
                    {
                        if !current_list.is_empty() {
                            let mut list_call = vec![Value::Symbol("list".to_string())];
                            list_call.append(&mut current_list);
                            append_args.push(Value::List(list_call));
                        }
                        if inner.len() < 2 {
                            return Err("unquote-splicing expects 1 argument".to_string());
                        }
                        append_args.push(self.expand(inner[1].clone(), env).await?);
                        continue;
                    }

                    let expanded_item = self
                        .expand_backquote(item.clone(), env, child_depth)
                        .await?;
                    current_list.push(expanded_item);
                }

                if !current_list.is_empty() {
                    let mut list_call = vec![Value::Symbol("list".to_string())];
                    list_call.extend(current_list);
                    append_args.push(Value::List(list_call));
                }

                if append_args.is_empty() {
                    Ok(Value::List(vec![
                        Value::Symbol("quote".to_string()),
                        Value::Nil,
                    ]))
                } else if append_args.len() == 1 {
                    Ok(append_args[0].clone())
                } else {
                    let mut res = vec![Value::Symbol("append".to_string())];
                    res.extend(append_args);
                    Ok(Value::List(res))
                }
            }
            Value::Vector(vec) => {
                let list_val = Value::List(vec);
                let list_expansion = self.expand_backquote(list_val, env, depth).await?;
                Ok(Value::List(vec![
                    Value::Symbol("apply".to_string()),
                    Value::Symbol("vector".to_string()),
                    list_expansion,
                ]))
            }
            Value::Symbol(_) | Value::Map(_) => {
                Ok(Value::List(vec![Value::Symbol("quote".to_string()), val]))
            }
            _ => Ok(val),
        }
    }

    pub async fn apply(
        &mut self,
        func: Value,
        args: Vec<Value>,
        env: &mut Rc<RefCell<Environment>>,
    ) -> Result<Value, String> {
        crate::interpreter::apply::apply(self, func, args, env).await
    }
}

pub fn default_env() -> Rc<RefCell<Environment>> {
    let mut env = Environment::new(None);
    crate::builtins::install(&mut env);
    Rc::new(RefCell::new(env))
}

pub fn default_interpreter() -> Interpreter {
    Interpreter::new()
}
