use dlisp_core::ast::Value;
use dlisp_core::interpreter::{default_env, default_interpreter};
use dlisp_core::parser::parse;
use std::cell::RefCell;
use std::rc::Rc;

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
    let interpreter = default_interpreter();
    let env = default_env();
    (interpreter, env)
}

// --- format integration tests ---

#[tokio::test]
async fn test_format_simple() {
    let (mut interp, mut env) = setup();
    let res = eval_str(r#"(format "Hello, {}!" "world")"#, &mut interp, &mut env)
        .await
        .unwrap();
    assert_eq!(res, Value::String("Hello, world!".to_string()));
}

#[tokio::test]
async fn test_format_multiple_args() {
    let (mut interp, mut env) = setup();
    let res = eval_str(r#"(format "{} + {} = {}" 1 2 3)"#, &mut interp, &mut env)
        .await
        .unwrap();
    assert_eq!(res, Value::String("1 + 2 = 3".to_string()));
}

#[tokio::test]
async fn test_format_with_computed_values() {
    let (mut interp, mut env) = setup();
    let res = eval_str(r#"(format "result: {}" (+ 10 20))"#, &mut interp, &mut env)
        .await
        .unwrap();
    assert_eq!(res, Value::String("result: 30".to_string()));
}

#[tokio::test]
async fn test_format_escaped_braces() {
    let (mut interp, mut env) = setup();
    let res = eval_str(r#"(format "Use {{}} for braces")"#, &mut interp, &mut env)
        .await
        .unwrap();
    assert_eq!(res, Value::String("Use {} for braces".to_string()));
}

#[tokio::test]
async fn test_format_not_enough_args_error() {
    let (mut interp, mut env) = setup();
    let res = eval_str(r#"(format "{} and {}" 1)"#, &mut interp, &mut env).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_format_with_string_concat() {
    let (mut interp, mut env) = setup();
    // Use format result in string-append
    let res = eval_str(
        r#"(string-append (format "Hello {}") (format " {}!" "world"))"#,
        &mut interp,
        &mut env,
    )
    .await;
    // format "Hello {}" has 1 placeholder but 0 args → error
    assert!(res.is_err());
}

#[tokio::test]
async fn test_format_with_let() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"(let ((name "Alice") (age 30)) (format "{} is {} years old" name age))"#,
        &mut interp,
        &mut env,
    )
    .await
    .unwrap();
    assert_eq!(res, Value::String("Alice is 30 years old".to_string()));
}

#[tokio::test]
async fn test_format_with_list() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"(format "items: {}" (list 1 2 3))"#,
        &mut interp,
        &mut env,
    )
    .await
    .unwrap();
    assert_eq!(res, Value::String("items: (1 2 3)".to_string()));
}

#[tokio::test]
async fn test_format_no_placeholders() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"(format "no placeholders at all")"#,
        &mut interp,
        &mut env,
    )
    .await
    .unwrap();
    assert_eq!(res, Value::String("no placeholders at all".to_string()));
}

#[tokio::test]
async fn test_println_returns_nil() {
    let (mut interp, mut env) = setup();
    let res = eval_str(r#"(println "hello" "world")"#, &mut interp, &mut env)
        .await
        .unwrap();
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_print_str_returns_nil() {
    let (mut interp, mut env) = setup();
    let res = eval_str(r#"(print-str "hello")"#, &mut interp, &mut env)
        .await
        .unwrap();
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_eprintln_returns_nil() {
    let (mut interp, mut env) = setup();
    let res = eval_str(r#"(eprintln "error message")"#, &mut interp, &mut env)
        .await
        .unwrap();
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_format_in_defun() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"
        (defun greet (name)
          (format "Hello, {}!" name))
        (greet "Bob")
        "#,
        &mut interp,
        &mut env,
    )
    .await
    .unwrap();
    assert_eq!(res, Value::String("Hello, Bob!".to_string()));
}

#[tokio::test]
async fn test_format_with_rest_params() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"
        (defun log-msg (level &rest parts)
          (format "[{}] {}" level (car parts)))
        (log-msg "INFO" "Server started")
        "#,
        &mut interp,
        &mut env,
    )
    .await
    .unwrap();
    assert_eq!(res, Value::String("[INFO] Server started".to_string()));
}
