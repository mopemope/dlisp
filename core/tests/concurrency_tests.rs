//! Interpreter-semantics tests for the concurrency builtins
//! (`chan` / `send` / `recv` / `try-recv` / `close` and `atom` family).
//!
//! Compiled-path parity is pinned by `example-lisp/concurrency.lisp` in
//! `cli/tests/parity_golden_tests.rs`.

use dlisp_core::ast::Value;
use dlisp_core::interpreter::default_interpreter;
use dlisp_core::parser::parse;
use std::cell::RefCell;
use std::rc::Rc;

fn setup() -> (
    dlisp_core::interpreter::Interpreter,
    Rc<RefCell<dlisp_core::environment::Environment>>,
) {
    (
        default_interpreter(),
        dlisp_core::interpreter::default_env(),
    )
}

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

#[tokio::test]
async fn test_chan_send_returns_true_and_recv_delivers() {
    let (mut i, mut e) = setup();
    let res = eval_str("(let ((ch (chan))) (send ch 42) (recv ch))", &mut i, &mut e).await;
    assert_eq!(res, Value::Integer(42));
}

#[tokio::test]
async fn test_try_recv_on_empty_channel_is_nil() {
    let (mut i, mut e) = setup();
    let res = eval_str("(try-recv (chan))", &mut i, &mut e).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_try_recv_returns_pending_value() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(let ((ch (chan))) (send ch :kw) (try-recv ch))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Keyword("kw".to_string()));
}

#[tokio::test]
async fn test_recv_preserves_fifo_order() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(let ((ch (chan))) (send ch 1) (send ch 2) (+ (* (recv ch) 10) (recv ch)))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Integer(12));
}

#[tokio::test]
async fn test_close_makes_send_return_false() {
    let (mut i, mut e) = setup();
    let res = eval_str("(let ((ch (chan))) (close ch) (send ch 1))", &mut i, &mut e).await;
    assert_eq!(res, Value::Bool(false));
}

#[tokio::test]
async fn test_close_is_idempotent() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(let ((ch (chan))) (close ch) (close ch) (send ch 1))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Bool(false));
}

#[tokio::test]
async fn test_recv_drains_then_returns_nil_after_close() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(let ((ch (chan))) (send ch 7) (close ch) (list (recv ch) (recv ch)))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::List(vec![Value::Integer(7), Value::Nil]));
}

#[tokio::test]
async fn test_spawned_task_communicates_via_channel() {
    let local = tokio::task::LocalSet::new();
    let res = local
        .run_until(async move {
            let (mut i, mut e) = setup();
            // The spawned task runs on the same thread; recv polls
            // cooperatively until the value arrives. The channel is passed
            // as an argument because compiled functions cannot reference
            // defvar globals.
            eval_str(
                "(require \"core\")\
                 (defun fill (out) (send out 99))\
                 (let ((ch (chan)))\
                   (spawn fill ch)\
                   (recv ch))",
                &mut i,
                &mut e,
            )
            .await
        })
        .await;
    assert_eq!(res, Value::Integer(99));
}

#[tokio::test]
async fn test_channel_values_are_identical_handles() {
    let (mut i, mut e) = setup();
    let res = eval_str("(let ((ch (chan))) (= ch ch))", &mut i, &mut e).await;
    assert_eq!(res, Value::Bool(true));
}

#[tokio::test]
async fn test_atom_deref_reset() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(let ((a (atom 10))) (reset! a 20) (deref a))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Integer(20));
}

#[tokio::test]
async fn test_reset_returns_new_value() {
    let (mut i, mut e) = setup();
    let res = eval_str("(reset! (atom 1) 5)", &mut i, &mut e).await;
    assert_eq!(res, Value::Integer(5));
}

#[tokio::test]
async fn test_atoms_are_independent() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(let ((a (atom 1)) (b (atom 2))) (reset! a 100) (list (deref a) (deref b)))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(
        res,
        Value::List(vec![Value::Integer(100), Value::Integer(2)])
    );
}

#[tokio::test]
async fn test_type_predicates() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(list \
         (channel? (chan)) (channel? (atom 1)) (channel? 1) \
         (atom? (atom 1)) (atom? (chan)) (atom? \"s\"))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(
        res,
        Value::List(vec![
            Value::Bool(true),
            Value::Bool(false),
            Value::Bool(false),
            Value::Bool(true),
            Value::Bool(false),
            Value::Bool(false),
        ])
    );
}

#[tokio::test]
async fn test_type_of_names() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(list (type-of (chan)) (type-of (atom nil)))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(
        res,
        Value::List(vec![
            Value::String("channel".to_string()),
            Value::String("atom".to_string()),
        ])
    );
}

#[tokio::test]
async fn test_send_requires_channel() {
    let (mut i, mut e) = setup();
    let exprs = parse("(send 1 2)").unwrap();
    let result = i.eval(exprs[0].clone(), &mut e).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_channels_flow_through_collections() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(require \"core\")\
         (defvar a (atom 0))\
         (defvar ch (chan))\
         (send ch {:x 1})\
         (defvar m (recv ch))\
         (reset! a (get m :x))\
         (deref a)",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Integer(1));
}
