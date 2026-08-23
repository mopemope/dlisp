use crate::ast::Value;
use crate::environment::Environment;
use crate::jit::JIT;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

pub(crate) fn is_codegen_special_form(name: &str) -> bool {
    matches!(
        name,
        "if" | "let"
            | "let*"
            | "lambda"
            | "spawn"
            | "quote"
            | "setq"
            | "defvar"
            | "progn"
            | "do"
            | "when"
            | "unless"
            | "and"
            | "or"
            | "cond"
    )
}

fn can_jit_compile_expr(
    expr: &Value,
    current_func: &str,
    env: &Environment,
    pending_functions: &HashSet<String>,
) -> bool {
    find_uncompiled_call(expr, current_func, env, pending_functions).is_none()
}

/// Returns the offending call target if `expr` contains a call that codegen
/// cannot lower (an interpreter-only builtin or an unresolved name).
pub(crate) fn find_uncompiled_call(
    expr: &Value,
    current_func: &str,
    env: &Environment,
    pending_functions: &HashSet<String>,
) -> Option<String> {
    match expr {
        Value::List(items) => {
            if items.is_empty() {
                return None;
            }

            if let Value::Symbol(op) = &items[0] {
                if op == "quote" {
                    return None;
                }

                if op != current_func && !is_codegen_special_form(op) {
                    match env.get(op) {
                        Some(Value::UserFunc {
                            jit_code: Some(_), ..
                        })
                        | Some(Value::Macro { .. }) => {}
                        Some(Value::NativeFunc(_)) => {
                            if !crate::codegen::COMPILED_BUILTINS.contains(&op.as_str()) {
                                return Some(op.clone());
                            }
                        }
                        Some(Value::UserFunc { jit_code: None, .. }) => {
                            if !pending_functions.contains(op) {
                                return Some(op.clone());
                            }
                        }
                        None => return Some(op.clone()),
                        Some(_) => {}
                    }
                }
            }

            items
                .iter()
                .find_map(|item| find_uncompiled_call(item, current_func, env, pending_functions))
        }
        Value::Vector(items) => items
            .iter()
            .find_map(|item| find_uncompiled_call(item, current_func, env, pending_functions)),
        Value::Map(map) => map.iter().find_map(|(k, v)| {
            find_uncompiled_call(k, current_func, env, pending_functions)
                .or_else(|| find_uncompiled_call(v, current_func, env, pending_functions))
        }),
        _ => None,
    }
}

fn retry_pending_jit_functions(jit: &mut JIT, env: &mut Rc<RefCell<Environment>>) {
    let candidates: Vec<(crate::ast::FuncDef, Option<Rc<RefCell<Environment>>>)> = {
        let borrowed = env.borrow();
        borrowed
            .values
            .iter()
            .filter_map(|(name, value)| match value {
                Value::UserFunc {
                    args,
                    rest_param,
                    body,
                    jit_code: None,
                    env,
                } => Some((
                    (name.clone(), args.clone(), rest_param.clone(), body.clone()),
                    env.clone(),
                )),
                _ => None,
            })
            .collect()
    };

    let pending_names: HashSet<String> = candidates
        .iter()
        .map(|((name, _, _, _), _)| name.clone())
        .collect();

    for ((name, args, rest_param, _), _) in &candidates {
        jit.register_signature(name, args, rest_param.clone());
    }

    let compilable: Vec<_> = candidates
        .into_iter()
        .filter(|((name, _, _, body), _)| {
            let borrowed = env.borrow();
            body.iter()
                .all(|expr| can_jit_compile_expr(expr, name, &borrowed, &pending_names))
        })
        .collect();

    let defs: Vec<_> = compilable.iter().map(|(def, _)| def.clone()).collect();

    if let Ok(compiled) = jit.compile_batch_with_rest(&defs) {
        for (name, code_ptr) in compiled {
            if let Some(((name, args, rest_param, body), captured_env)) = compilable
                .iter()
                .find(|((candidate_name, _, _, _), _)| *candidate_name == *name)
            {
                env.borrow_mut().set(
                    name.clone(),
                    Value::UserFunc {
                        args: args.clone(),
                        rest_param: rest_param.clone(),
                        body: body.clone(),
                        jit_code: Some(code_ptr as usize),
                        env: captured_env.clone(),
                    },
                );
            }
        }
    }
}

pub fn defun(
    jit: &mut JIT,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, String> {
    if args.len() < 3 {
        return Err("defun requires at least 3 arguments".to_string());
    }
    let func_name = match &args[0] {
        Value::Symbol(n) => n.clone(),
        _ => return Err("defun name must be a symbol".to_string()),
    };
    let params = match &args[1] {
        Value::List(l) => l,
        _ => return Err("defun args must be a list".to_string()),
    };
    let mut arg_names = Vec::new();
    let mut rest_param = None;
    let mut i = 0;
    while i < params.len() {
        match &params[i] {
            Value::Symbol(n) if n == "&rest" => {
                if i + 1 >= params.len() {
                    return Err("&rest requires a parameter name".to_string());
                }
                match &params[i + 1] {
                    Value::Symbol(rest_name) => {
                        rest_param = Some(rest_name.clone());
                    }
                    _ => return Err("&rest parameter must be a symbol".to_string()),
                }
                if i + 2 < params.len() {
                    return Err("&rest parameter must be last in the parameter list".to_string());
                }
                break;
            }
            Value::Symbol(n) => arg_names.push(n.clone()),
            _ => return Err("defun arg must be a symbol".to_string()),
        }
        i += 1;
    }
    let body = args[2..].to_vec();

    jit.register_signature(&func_name, &arg_names, rest_param.clone());

    let jit_code = if body.iter().all(|expr| {
        can_jit_compile_expr(
            expr,
            &func_name,
            &env.borrow(),
            &HashSet::from([func_name.clone()]),
        )
    }) {
        match jit.compile_with_rest(&func_name, &arg_names, rest_param.clone(), &body) {
            Ok(code) => Some(code as usize),
            Err(_) => None,
        }
    } else {
        None
    };

    let func = Value::UserFunc {
        args: arg_names,
        rest_param,
        body,
        jit_code,
        env: Some(env.clone()),
    };
    env.borrow_mut().set(func_name.clone(), func);
    retry_pending_jit_functions(jit, env);
    Ok(Some(Value::Symbol(func_name)))
}
