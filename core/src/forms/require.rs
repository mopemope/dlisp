use crate::ast::Value;
use crate::environment::Environment;
use crate::eval_failure::EvalFailure;
use crate::interpreter::Interpreter;
use crate::parser::parse;
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub fn module_forms(module: &str) -> Result<Vec<Value>, String> {
    let source = dlisp_stdlib::get_module(module).ok_or_else(|| {
        let available = dlisp_stdlib::module_names().join(", ");
        format!(
            "Unknown stdlib module '{}'. Available modules: {}",
            module, available
        )
    })?;

    parse(source).map_err(|e| format!("Parse error in stdlib module '{}': {:?}", module, e))
}

pub fn is_file_module_name(module: &str) -> bool {
    let path = Path::new(module);
    path.is_absolute()
        || module.starts_with("./")
        || module.starts_with("../")
        || module.contains(".lisp")
}

pub fn normalize_source_dir(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };

    fs::canonicalize(&absolute).unwrap_or(absolute)
}

pub fn resolve_source_path(path: &str, env: &Environment) -> PathBuf {
    let requested = Path::new(path);
    if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        env.current_source_dir().join(requested)
    }
}

pub fn file_module_key(path: &Path) -> String {
    format!("file:{}", path.display())
}

pub fn file_module_forms(
    module: &str,
    env: &Environment,
) -> Result<(PathBuf, String, Vec<Value>), String> {
    let resolved = resolve_source_path(module, env);
    let canonical = fs::canonicalize(&resolved).map_err(|e| {
        format!(
            "Failed to resolve required file '{}': {}",
            resolved.display(),
            e
        )
    })?;
    let content = fs::read_to_string(&canonical).map_err(|e| {
        format!(
            "Failed to read required file '{}': {}",
            canonical.display(),
            e
        )
    })?;
    let forms = parse(&content).map_err(|e| {
        format!(
            "Parse error in required file '{}': {:?}",
            canonical.display(),
            e
        )
    })?;
    let key = file_module_key(&canonical);
    Ok((canonical, key, forms))
}

pub fn literal_module_name(args: &[Value]) -> Result<String, String> {
    if args.len() != 1 {
        return Err("require requires exactly 1 module name argument".to_string());
    }

    match &args[0] {
        Value::String(module) => Ok(module.clone()),
        _ => Err("require requires a string module name".to_string()),
    }
}

pub fn required_module_from_expr(expr: &Value) -> Result<Option<String>, String> {
    let Value::List(list) = expr else {
        return Ok(None);
    };
    if !matches!(list.first(), Some(Value::Symbol(s)) if s == "require") {
        return Ok(None);
    }
    literal_module_name(&list[1..]).map(Some)
}

pub async fn require_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, EvalFailure> {
    if args.len() != 1 {
        return Err(EvalFailure::message(
            "require requires exactly 1 module name argument",
        ));
    }

    let module_val = interpreter.eval(args[0].clone(), env).await?;
    let module = match module_val {
        Value::String(module) => module,
        _ => return Err("require requires a string module name".to_string().into()),
    };

    if is_file_module_name(&module) {
        let (canonical, key, forms) = {
            let borrowed = env.borrow();
            file_module_forms(&module, &borrowed)?
        };

        if env.borrow().has_loaded_module(&key) {
            return Ok(Some(Value::String(module)));
        }

        env.borrow_mut().mark_loaded_module(&key);
        if let Some(parent) = canonical.parent() {
            env.borrow_mut()
                .push_source_dir(normalize_source_dir(parent));
        }

        let mut result = Value::Nil;
        let mut eval_error = None;
        for form in forms {
            match interpreter.eval(form, env).await {
                Ok(value) => result = value,
                Err(err) => {
                    eval_error = Some(err);
                    break;
                }
            }
        }

        if canonical.parent().is_some() {
            env.borrow_mut().pop_source_dir();
        }

        if let Some(err) = eval_error {
            env.borrow_mut().unmark_loaded_module(&key);
            return Err(err);
        }

        return Ok(Some(result));
    }

    if env.borrow().has_loaded_module(&module) {
        return Ok(Some(Value::String(module)));
    }

    let forms = module_forms(&module)?;
    let mut result = Value::Nil;
    for form in forms {
        result = interpreter.eval(form, env).await?;
    }

    env.borrow_mut().mark_loaded_module(&module);
    Ok(Some(result))
}
