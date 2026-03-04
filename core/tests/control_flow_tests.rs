use dlisp_core::ast::Value;
use dlisp_core::interpreter::{default_env, default_interpreter};
use dlisp_core::parser::parse;
use std::cell::RefCell;
use std::rc::Rc;

async fn eval_str(
    src: &str,
    interpreter: &mut dlisp_core::interpreter::Interpreter,
    env: &mut Rc<RefCell<dlisp_core::environment::Environment>>,
) -> Value {
    let exprs = parse(src).unwrap();
    let mut result = Value::Nil;
    for expr in exprs {
        result = interpreter.eval(expr, env).await.unwrap();
    }
    result
}

fn setup() -> (
    dlisp_core::interpreter::Interpreter,
    Rc<RefCell<dlisp_core::environment::Environment>>,
) {
    let interpreter = default_interpreter();
    let env = default_env();
    (interpreter, env)
}

// ===== when =====

#[tokio::test]
async fn test_when_true() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(when true 1 2 3)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(3));
}

#[tokio::test]
async fn test_when_false() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(when false 1 2 3)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_when_truthy_integer() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(when 42 \"yes\")", &mut interp, &mut env).await;
    assert_eq!(res, Value::String("yes".to_string()));
}

#[tokio::test]
async fn test_when_nil_is_falsy() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(when nil \"yes\")", &mut interp, &mut env).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_when_side_effects() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        "(defvar x 0) (when true (setq x 10)) x",
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(res, Value::Integer(10));
}

// ===== unless =====

#[tokio::test]
async fn test_unless_false() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(unless false 1 2 3)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(3));
}

#[tokio::test]
async fn test_unless_true() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(unless true 1 2 3)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_unless_nil() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(unless nil \"executed\")", &mut interp, &mut env).await;
    assert_eq!(res, Value::String("executed".to_string()));
}

// ===== dotimes =====

#[tokio::test]
async fn test_dotimes_basic() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"
        (defvar acc 0)
        (dotimes (i 5) (setq acc (+ acc i)))
        acc
        "#,
        &mut interp,
        &mut env,
    )
    .await;
    // 0 + 1 + 2 + 3 + 4 = 10
    assert_eq!(res, Value::Integer(10));
}

#[tokio::test]
async fn test_dotimes_zero() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"
        (defvar acc 0)
        (dotimes (i 0) (setq acc (+ acc 1)))
        acc
        "#,
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(res, Value::Integer(0));
}

#[tokio::test]
async fn test_dotimes_returns_nil() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(dotimes (i 3) (+ i 1))", &mut interp, &mut env).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_dotimes_with_expression_count() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"
        (defvar acc 0)
        (dotimes (i (+ 2 3)) (setq acc (+ acc 1)))
        acc
        "#,
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(res, Value::Integer(5));
}

// ===== dolist =====

#[tokio::test]
async fn test_dolist_basic() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"
        (defvar acc 0)
        (dolist (x '(1 2 3 4 5)) (setq acc (+ acc x)))
        acc
        "#,
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(res, Value::Integer(15));
}

#[tokio::test]
async fn test_dolist_empty_list() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"
        (defvar acc 0)
        (dolist (x '()) (setq acc (+ acc 1)))
        acc
        "#,
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(res, Value::Integer(0));
}

#[tokio::test]
async fn test_dolist_returns_nil() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(dolist (x '(1 2 3)) (+ x 1))", &mut interp, &mut env).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_dolist_with_vector() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"
        (defvar acc 0)
        (dolist (x [10 20 30]) (setq acc (+ acc x)))
        acc
        "#,
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(res, Value::Integer(60));
}

#[tokio::test]
async fn test_dolist_with_nil() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"
        (defvar acc 0)
        (dolist (x nil) (setq acc (+ acc 1)))
        acc
        "#,
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(res, Value::Integer(0));
}

#[tokio::test]
async fn test_dolist_string_elements() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        r#"
        (defvar result "")
        (dolist (s '("a" "b" "c")) (setq result (string-append result s)))
        result
        "#,
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(res, Value::String("abc".to_string()));
}
