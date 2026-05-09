use dlisp_core::ast::Value;
use dlisp_core::interpreter::{default_env, default_interpreter};
use dlisp_core::parser::parse;
use std::cell::RefCell;
use std::rc::Rc;

async fn eval_str(
    src: &str,
    interpreter: &mut dlisp_core::interpreter::Interpreter,
    env: &mut Rc<RefCell<dlisp_core::environment::Environment>>,
) -> Result<Value, String> {
    let exprs = parse(src).unwrap();
    let mut result = Value::Nil;
    for expr in exprs {
        result = interpreter.eval(expr, env).await?;
    }
    Ok(result)
}

fn setup() -> (
    dlisp_core::interpreter::Interpreter,
    Rc<RefCell<dlisp_core::environment::Environment>>,
) {
    (default_interpreter(), default_env())
}

#[tokio::test]
async fn test_require_core_loads_stdlib_functions_and_macros() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"
        (require "core")
        (list
          (inc 1)
          (dec 5)
          (second '(10 20 30))
          (when-let (x 41) (inc x)))
        "#,
        &mut interp,
        &mut env,
    )
    .await
    .unwrap();

    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(2),
            Value::Integer(4),
            Value::Integer(20),
            Value::Integer(42),
        ])
    );
}

#[tokio::test]
async fn test_require_core_is_loaded_once_per_environment() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"
        (require "core")
        (defun inc (x) (+ x 100))
        (require "core")
        (inc 1)
        "#,
        &mut interp,
        &mut env,
    )
    .await
    .unwrap();

    assert_eq!(res, Value::Integer(101));
}

#[tokio::test]
async fn test_require_in_child_scope_does_not_mark_parent_loaded() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"
        (let ()
          (require "core"))
        (require "core")
        (inc 1)
        "#,
        &mut interp,
        &mut env,
    )
    .await
    .unwrap();

    assert_eq!(res, Value::Integer(2));
}

#[tokio::test]
async fn test_require_unknown_module_errors() {
    let (mut interp, mut env) = setup();
    let exprs = parse(r#"(require "missing")"#).unwrap();
    let err = interp.eval(exprs[0].clone(), &mut env).await.unwrap_err();

    assert!(err.contains("Unknown stdlib module 'missing'"));
    assert!(err.contains("core"));
}

#[tokio::test]
async fn test_require_requires_string_module_name() {
    let (mut interp, mut env) = setup();
    let exprs = parse("(require 42)").unwrap();
    let err = interp.eval(exprs[0].clone(), &mut env).await.unwrap_err();

    assert!(err.contains("require requires a string module name"));
}
