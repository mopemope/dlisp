use dlisp_core::ast::Value;
use dlisp_core::interpreter::{default_env, default_interpreter};
use dlisp_core::parser::parse;
use std::cell::RefCell;
use std::rc::Rc;

fn setup() -> (
    dlisp_core::interpreter::Interpreter,
    Rc<RefCell<dlisp_core::environment::Environment>>,
) {
    let interpreter = default_interpreter();
    let env = default_env();
    (interpreter, env)
}

async fn eval_str(
    src: &str,
    interpreter: &mut dlisp_core::interpreter::Interpreter,
    env: &mut Rc<RefCell<dlisp_core::environment::Environment>>,
) -> Result<Value, String> {
    let exprs = parse(src).map_err(|e| format!("{:?}", e))?;
    let mut result = Value::Nil;
    for expr in exprs {
        result = interpreter.eval(expr, env).await?;
    }
    Ok(result)
}

// --- let destructuring ---

#[tokio::test]
async fn test_let_destructure_basic() {
    let (mut i, mut e) = setup();
    let res = eval_str("(let (((a b) '(1 2))) (+ a b))", &mut i, &mut e).await;
    assert_eq!(res.unwrap(), Value::Integer(3));
}

#[tokio::test]
async fn test_let_destructure_with_rest() {
    let (mut i, mut e) = setup();
    let res = eval_str("(let (((x &rest ys) '(10 20 30))) ys)", &mut i, &mut e).await;
    assert_eq!(
        res.unwrap(),
        Value::List(vec![Value::Integer(20), Value::Integer(30)])
    );
}

#[tokio::test]
async fn test_let_destructure_rest_empty() {
    let (mut i, mut e) = setup();
    let res = eval_str("(let (((x &rest ys) '(10))) ys)", &mut i, &mut e).await;
    assert_eq!(res.unwrap(), Value::Nil);
}

#[tokio::test]
async fn test_let_destructure_nested() {
    let (mut i, mut e) = setup();
    // Nested destructuring: ((a (b c)) '(1 (2 3)))
    let res = eval_str(
        "(let (((a (b c)) '(1 (2 3)))) (+ a (+ b c)))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res.unwrap(), Value::Integer(6));
}

#[tokio::test]
async fn test_let_destructure_fewer_values() {
    let (mut i, mut e) = setup();
    // Pattern has more variables than values -> extra binds to nil
    let res = eval_str("(let (((a b c) '(1 2))) c)", &mut i, &mut e).await;
    assert_eq!(res.unwrap(), Value::Nil);
}

#[tokio::test]
async fn test_let_destructure_non_list_error() {
    let (mut i, mut e) = setup();
    // Destructure pattern is list, but value is not a list -> error
    let res = eval_str("(let (((a b) 42)) a)", &mut i, &mut e).await;
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .contains("destructuring bind expects a list")
    );
}

#[tokio::test]
async fn test_let_destructure_mixed_simple_and_pattern() {
    let (mut i, mut e) = setup();
    // Mix of simple binding and destructuring in same let
    let res = eval_str(
        "(let ((x 100) ((a b) '(1 2))) (+ x (+ a b)))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res.unwrap(), Value::Integer(103));
}

// --- let* destructuring ---

#[tokio::test]
async fn test_let_star_destructure_basic() {
    let (mut i, mut e) = setup();
    let res = eval_str("(let* (((a b) '(10 20))) (+ a b))", &mut i, &mut e).await;
    assert_eq!(res.unwrap(), Value::Integer(30));
}

#[tokio::test]
async fn test_let_star_destructure_sequential() {
    let (mut i, mut e) = setup();
    // Sequential: first binding creates vals, second destructures using first
    let res = eval_str(
        "(let* ((vals '(3 4)) ((x y) vals)) (+ x y))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res.unwrap(), Value::Integer(7));
}

#[tokio::test]
async fn test_let_star_destructure_with_rest() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(let* (((head &rest tail) '(1 2 3 4))) (list head tail))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(
        res.unwrap(),
        Value::List(vec![
            Value::Integer(1),
            Value::List(vec![
                Value::Integer(2),
                Value::Integer(3),
                Value::Integer(4)
            ])
        ])
    );
}

// --- defmacro &rest edge cases ---

#[tokio::test]
async fn test_defmacro_rest_too_few_args() {
    let (mut i, mut e) = setup();
    eval_str(
        "(defmacro needs-name (name &rest body) `(list ,name ,@body))",
        &mut i,
        &mut e,
    )
    .await
    .unwrap();
    // Calling with 0 args should error (needs at least 1 for 'name')
    let res = eval_str("(needs-name)", &mut i, &mut e).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_defmacro_rest_invocation() {
    let (mut i, mut e) = setup();
    eval_str(
        r#"(defmacro with-logging (msg &rest body)
             `(do (println ,msg) ,@body))"#,
        &mut i,
        &mut e,
    )
    .await
    .unwrap();
    // Should expand and execute without error
    let res = eval_str("(with-logging \"start\" (+ 1 2))", &mut i, &mut e).await;
    assert_eq!(res.unwrap(), Value::Integer(3));
}

// --- setq edge cases ---

#[tokio::test]
async fn test_setq_returns_last_value() {
    let (mut i, mut e) = setup();
    let res = eval_str("(let ((a 0) (b 0)) (setq a 10 b 20))", &mut i, &mut e).await;
    assert_eq!(res.unwrap(), Value::Integer(20));
}

#[tokio::test]
async fn test_setq_empty_args_error() {
    let (mut i, mut e) = setup();
    let exprs = parse("(setq)").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("pairs of (symbol value)"));
}

#[tokio::test]
async fn test_setq_non_symbol_error() {
    let (mut i, mut e) = setup();
    let exprs = parse("(setq 42 10)").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("must be symbols"));
}

#[tokio::test]
async fn test_setq_sequential_eval() {
    let (mut i, mut e) = setup();
    // Second value expression can reference the updated first variable
    let res = eval_str("(let ((x 1) (y 0)) (setq x 10 y x) y)", &mut i, &mut e).await;
    assert_eq!(res.unwrap(), Value::Integer(10));
}
