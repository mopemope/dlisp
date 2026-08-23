use dlisp_core::ast::Value;
use dlisp_core::interpreter::{default_env, default_interpreter};
use dlisp_core::parser::parse;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

async fn eval_str(
    src: &str,
    interpreter: &mut dlisp_core::interpreter::Interpreter,
    env: &mut Rc<RefCell<dlisp_core::environment::Environment>>,
) -> Result<Value, dlisp_core::eval_failure::EvalFailure> {
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

fn temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "dlisp_require_{}_{}_{}",
        name,
        std::process::id(),
        unique
    ))
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

    assert!(err.to_string().contains("Unknown stdlib module 'missing'"));
    assert!(err.to_string().contains("core"));
}

#[tokio::test]
async fn test_require_requires_string_module_name() {
    let (mut interp, mut env) = setup();
    let exprs = parse("(require 42)").unwrap();
    let err = interp.eval(exprs[0].clone(), &mut env).await.unwrap_err();

    assert!(
        err.to_string()
            .contains("require requires a string module name")
    );
}

#[tokio::test]
async fn test_require_file_loads_once_by_canonical_path() {
    let dir = temp_dir("once");
    fs::create_dir_all(&dir).unwrap();
    let module = dir.join("counter.lisp");
    fs::write(
        &module,
        r#"
        (defvar loaded-count 0)
        (setq loaded-count (+ loaded-count 1))
        (defun add-ten (x) (+ x 10))
        "#,
    )
    .unwrap();

    let (mut interp, mut env) = setup();
    let res = eval_str(
        &format!(
            r#"
            (require "{}")
            (require "{}")
            (list loaded-count (add-ten 5))
            "#,
            module.display(),
            module.display()
        ),
        &mut interp,
        &mut env,
    )
    .await
    .unwrap();

    assert_eq!(
        res,
        Value::List(vec![Value::Integer(1), Value::Integer(15)])
    );

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_require_file_failure_rolls_back_loaded_marker() {
    let dir = temp_dir("retry");
    fs::create_dir_all(&dir).unwrap();
    let module = dir.join("flaky.lisp");
    fs::write(
        &module,
        r#"
        (defvar partial-load 1)
        missing-symbol
        "#,
    )
    .unwrap();

    let (mut interp, mut env) = setup();
    let first = eval_str(
        &format!(r#"(require "{}")"#, module.display()),
        &mut interp,
        &mut env,
    )
    .await;
    assert!(first.is_err());

    fs::write(
        &module,
        r#"
        (defun recovered () 42)
        "#,
    )
    .unwrap();
    let res = eval_str(
        &format!(
            r#"
            (require "{}")
            (recovered)
            "#,
            module.display()
        ),
        &mut interp,
        &mut env,
    )
    .await
    .unwrap();

    assert_eq!(res, Value::Integer(42));

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_require_file_resolves_nested_relative_to_required_file() {
    let dir = temp_dir("nested");
    let nested = dir.join("nested");
    fs::create_dir_all(&nested).unwrap();
    fs::write(
        nested.join("math.lisp"),
        r#"
        (defun triple (x) (* x 3))
        "#,
    )
    .unwrap();
    fs::write(
        dir.join("main.lisp"),
        r#"
        (require "./nested/math.lisp")
        (defun from-main (x) (triple x))
        "#,
    )
    .unwrap();

    let (mut interp, mut env) = setup();
    env.borrow_mut().push_source_dir(dir.clone());
    let res = eval_str(
        r#"
        (require "./main.lisp")
        (from-main 7)
        "#,
        &mut interp,
        &mut env,
    )
    .await
    .unwrap();

    assert_eq!(res, Value::Integer(21));

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_require_missing_file_error_contains_resolved_path() {
    let dir = temp_dir("missing");
    fs::create_dir_all(&dir).unwrap();

    let (mut interp, mut env) = setup();
    env.borrow_mut().push_source_dir(dir.clone());
    let exprs = parse(r#"(require "./missing.lisp")"#).unwrap();
    let err = interp.eval(exprs[0].clone(), &mut env).await.unwrap_err();

    assert!(err.to_string().contains("Failed to resolve required file"));
    assert!(err.to_string().contains(&dir.display().to_string()));
    assert!(err.to_string().contains("missing.lisp"));

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_require_core_threading_macros() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"
        (require "core")
        (list
          (-> 5 inc)
          (-> '(1 2 3) (cdr) (car))
          (-> 5)
          (->> '(1 2 3) (map inc))
          (->> '(1 2 3 4) (filter (lambda (x) (> x 2))))
          (as-> 10 x (+ x 5) (* x 2))
          (as-> 100 y)
          (some-> nil inc)
          (some-> 5 inc)
          (-> "hello" (string-upper)))
        "#,
        &mut interp,
        &mut env,
    )
    .await
    .unwrap();

    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(6),
            Value::Integer(2),
            Value::Integer(5),
            Value::List(vec![
                Value::Integer(2),
                Value::Integer(3),
                Value::Integer(4)
            ]),
            Value::List(vec![Value::Integer(3), Value::Integer(4)]),
            Value::Integer(30),
            Value::Integer(100),
            Value::Nil,
            Value::Integer(6),
            Value::String("HELLO".to_string()),
        ])
    );
}

#[tokio::test]
async fn test_require_core_threading_macros_in_defun() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"
        (require "core")
        (defun process (xs)
          (->> xs (map inc) (filter (lambda (x) (= x 0)))))
        (process '(-1 0 1))
        "#,
        &mut interp,
        &mut env,
    )
    .await
    .unwrap();

    assert_eq!(res, Value::List(vec![Value::Integer(0)]));
}
