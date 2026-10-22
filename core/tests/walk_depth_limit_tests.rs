use std::time::Duration;

use dlisp_core::ast::Value;
use dlisp_core::eval_failure::EvalFailure;
use dlisp_core::interpreter::{Interpreter, default_env};
use dlisp_core::parser::parse;

/// Run `code` with the evaluation-depth limit forced to `limit` so the
/// guard trips well before a 2 MiB test thread's stack would overflow.
async fn run_with_limit(code: &str, limit: usize) -> Result<Value, EvalFailure> {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();
    interpreter.max_eval_depth = limit;
    let mut result = Value::Nil;
    for stmt in parse(code).unwrap() {
        result = interpreter.eval(stmt, &mut env).await?;
    }
    Ok(result)
}

async fn run_expect_depth_error(code: &str) -> EvalFailure {
    tokio::time::timeout(Duration::from_secs(10), run_with_limit(code, 64))
        .await
        .expect("evaluation must terminate, not loop or crash")
        .expect_err("program must fail with a catchable depth error")
}

#[tokio::test]
async fn test_self_expanding_macro_is_catchable_error() {
    let failure = run_expect_depth_error("(defmacro selfloop () '(selfloop))\n(selfloop)").await;
    assert!(
        failure.to_string().contains("Maximum evaluation depth"),
        "self-expanding macro must raise the depth error, got: {failure}"
    );
}

#[tokio::test]
async fn test_deep_interpreted_recursion_is_catchable_error() {
    // `format` is interpreter-only, so the JIT gate keeps `spin` on the
    // tree-walking interpreter and the recursion goes through `eval`.
    let code = "(defun spin (n) (if (= n 0) 0 (format \"~a\" (spin (- n 1)))))\n(spin 1000)";
    let failure = run_expect_depth_error(code).await;
    assert!(
        failure.to_string().contains("Maximum evaluation depth"),
        "deep recursion must raise the depth error, got: {failure}"
    );
}

#[tokio::test]
async fn test_normal_recursion_still_works_below_limit() {
    // Legitimate small-depth recursion must be unaffected by the guard.
    let value = run_with_limit(
        "(defun sumto (n) (if (= n 0) 0 (+ n (sumto (- n 1)))))\n(sumto 40)",
        256,
    )
    .await
    .unwrap();
    assert_eq!(value, Value::Integer(820));
}
