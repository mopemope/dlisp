use crate::ast::Value;
use crate::environment::Environment;
use crate::eval_failure::EvalFailure;
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
            | "while"
            | "dotimes"
            | "dolist"
            | "loop"
            | "recur"
    )
}

/// Special forms registered in the interpreter that have no codegen
/// lowering. AOT compilation must reject them with an explicit error
/// instead of emitting unresolvable symbols.
///
/// `some`/`every`/`find`/`for-each`, `map`/`filter`/`reduce`,
/// `while`/`dotimes`/`dolist` are lowered as compiled builtins instead and
/// are listed in `crate::codegen::COMPILED_BUILTINS`.
pub(crate) fn is_interpreter_only_special_form(name: &str) -> bool {
    matches!(
        name,
        "try"
            | "throw"
            | "eval"
            | "apply"
            | "macroexpand"
            | "load"
            | "require"
            | "defmacro"
            | "map-indexed"
            | "update"
            | "map-keys"
            | "map-vals"
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
    find_uncompiled_call_inner(expr, current_func, env, pending_functions, &[])
}

/// Collects the variable names bound by a destructuring pattern.
fn collect_pattern_names(pattern: &Value, out: &mut Vec<String>) {
    match pattern {
        Value::Symbol(s) if s != "&rest" => out.push(s.clone()),
        Value::List(items) => {
            for item in items {
                collect_pattern_names(item, out);
            }
        }
        _ => {}
    }
}

fn find_uncompiled_call_inner(
    expr: &Value,
    current_func: &str,
    env: &Environment,
    pending_functions: &HashSet<String>,
    locals: &[String],
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

                // `let` / `let*`: binding heads are names, not calls. Init
                // expressions see the outer scope; the body sees the new
                // bindings as locals.
                if op == "let" || op == "let*" {
                    let mut inner_locals: Vec<String> = locals.to_vec();
                    if let Some(Value::List(bindings)) = items.get(1) {
                        for binding in bindings {
                            if let Value::List(pair) = binding {
                                if pair.is_empty() {
                                    return Some("let".to_string());
                                }
                                collect_pattern_names(&pair[0], &mut inner_locals);
                                if pair.len() != 2 {
                                    continue;
                                }
                                find_uncompiled_call_inner(
                                    &pair[1],
                                    current_func,
                                    env,
                                    pending_functions,
                                    locals,
                                )?;
                            }
                        }
                    }
                    for item in items.iter().skip(2) {
                        find_uncompiled_call_inner(
                            item,
                            current_func,
                            env,
                            pending_functions,
                            &inner_locals,
                        )?;
                    }
                    return None;
                }

                // `loop`: flat name/init bindings introduce locals for the body.
                if op == "loop" {
                    let mut inner_locals: Vec<String> = locals.to_vec();
                    match items.get(1) {
                        Some(Value::List(bs)) | Some(Value::Vector(bs)) => {
                            if bs.len() % 2 != 0 {
                                return Some("loop".to_string());
                            }
                            for name in bs.iter().step_by(2) {
                                collect_pattern_names(name, &mut inner_locals);
                            }
                        }
                        _ => return Some("loop".to_string()),
                    }
                    for item in items.iter().skip(2) {
                        find_uncompiled_call_inner(
                            item,
                            current_func,
                            env,
                            pending_functions,
                            &inner_locals,
                        )?;
                    }
                    return None;
                }

                // `lambda`: parameters are locals of the lambda body.
                if op == "lambda" {
                    let mut inner_locals: Vec<String> = locals.to_vec();
                    if let Some(Value::List(params)) = items.get(1) {
                        for param in params {
                            collect_pattern_names(param, &mut inner_locals);
                        }
                    }
                    for item in items.iter().skip(2) {
                        find_uncompiled_call_inner(
                            item,
                            current_func,
                            env,
                            pending_functions,
                            &inner_locals,
                        )?;
                    }
                    return None;
                }

                if op != current_func && !is_codegen_special_form(op) && !locals.contains(op) {
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
                        None => {
                            // Names lowered directly by codegen (e.g. the
                            // higher-order forms registered only in the
                            // interpreter registry) need no env binding.
                            if !crate::codegen::COMPILED_BUILTINS.contains(&op.as_str()) {
                                return Some(op.clone());
                            }
                        }
                        Some(_) => {}
                    }
                }
            }

            items.iter().find_map(|item| {
                find_uncompiled_call_inner(item, current_func, env, pending_functions, locals)
            })
        }
        Value::Vector(items) => items.iter().find_map(|item| {
            find_uncompiled_call_inner(item, current_func, env, pending_functions, locals)
        }),
        Value::Map(map) => map.iter().find_map(|(k, v)| {
            find_uncompiled_call_inner(k, current_func, env, pending_functions, locals).or_else(
                || find_uncompiled_call_inner(v, current_func, env, pending_functions, locals),
            )
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
) -> Result<Option<Value>, EvalFailure> {
    if args.len() < 3 {
        return Err("defun requires at least 3 arguments".to_string().into());
    }
    let func_name = match &args[0] {
        Value::Symbol(n) => n.clone(),
        _ => return Err("defun name must be a symbol".to_string().into()),
    };
    let params = match &args[1] {
        Value::List(l) => l,
        _ => return Err("defun args must be a list".to_string().into()),
    };
    let mut arg_names = Vec::new();
    let mut rest_param = None;
    let mut i = 0;
    while i < params.len() {
        match &params[i] {
            Value::Symbol(n) if n == "&rest" => {
                if i + 1 >= params.len() {
                    return Err("&rest requires a parameter name".to_string().into());
                }
                match &params[i + 1] {
                    Value::Symbol(rest_name) => {
                        rest_param = Some(rest_name.clone());
                    }
                    _ => return Err("&rest parameter must be a symbol".to_string().into()),
                }
                if i + 2 < params.len() {
                    return Err("&rest parameter must be last in the parameter list"
                        .to_string()
                        .into());
                }
                break;
            }
            Value::Symbol(n) => arg_names.push(n.clone()),
            _ => return Err("defun arg must be a symbol".to_string().into()),
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
