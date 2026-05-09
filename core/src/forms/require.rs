use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use crate::parser::parse;
use std::cell::RefCell;
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
) -> Result<Option<Value>, String> {
    if args.len() != 1 {
        return Err("require requires exactly 1 module name argument".to_string());
    }

    let module_val = interpreter.eval(args[0].clone(), env).await?;
    let module = match module_val {
        Value::String(module) => module,
        _ => return Err("require requires a string module name".to_string()),
    };

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
