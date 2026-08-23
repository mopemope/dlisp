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
async fn test_loop_counts_down() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(loop [n 5 acc 0] (if (= n 0) acc (recur (- n 1) (+ acc n))))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Integer(15));
}

#[tokio::test]
async fn test_loop_returns_last_body_value_without_recur() {
    let (mut i, mut e) = setup();
    let res = eval_str("(loop [x 1] (* x 10))", &mut i, &mut e).await;
    assert_eq!(res, Value::Integer(10));
}

#[tokio::test]
async fn test_loop_zero_iterations_body_value() {
    let (mut i, mut e) = setup();
    // Body runs once even with an immediately-false condition; recur never fires.
    let res = eval_str(
        "(loop [done true] (if done :finished (recur true)))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Keyword("finished".to_string()));
}

#[tokio::test]
async fn test_loop_nested_loops_inner_recur() {
    let (mut i, mut e) = setup();
    // Inner loop's recur must not disturb the outer loop.
    let res = eval_str(
        "(loop [outer 2 total 0]
           (if (= outer 0)
               total
               (recur (- outer 1)
                      (+ total (loop [inner 3 acc 0]
                                 (if (= inner 0) acc (recur (- inner 1) (+ acc 1))))))))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Integer(6));
}

#[tokio::test]
async fn test_loop_recur_from_inside_lambda_call() {
    let (mut i, mut e) = setup();
    // Dynamic escape: recur inside a called function still targets the loop.
    let res = eval_str(
        "(defun step (n) (recur (- n 1)))
         (loop [n 3] (if (= n 0) :done (step n)))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Keyword("done".to_string()));
}

#[tokio::test]
async fn test_recur_outside_loop_is_error() {
    let (mut i, mut e) = setup();
    let exprs = parse("(recur 1)").unwrap();
    let err = i.eval(exprs[0].clone(), &mut e).await.unwrap_err();
    assert!(err.to_string().contains("recur outside loop"));
}

#[tokio::test]
async fn test_loop_recur_arity_mismatch_is_error() {
    let (mut i, mut e) = setup();
    let exprs = parse("(loop [x 1 y 2] (recur x))").unwrap();
    let err = i.eval(exprs[0].clone(), &mut e).await.unwrap_err();
    assert!(err.to_string().contains("expects 2 argument(s), got 1"));
}

#[tokio::test]
async fn test_loop_binding_name_must_be_symbol() {
    let (mut i, mut e) = setup();
    let exprs = parse("(loop [1 2] 3)").unwrap();
    let err = i.eval(exprs[0].clone(), &mut e).await.unwrap_err();
    assert!(err.to_string().contains("must be a symbol"));
}

#[tokio::test]
async fn test_loop_deep_iteration_is_stack_safe() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(loop [n 100000 acc 0] (if (= n 0) acc (recur (- n 1) (+ acc 1))))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Integer(100000));
}

#[tokio::test]
async fn test_defun_with_loop_recur_is_jit_compilable() {
    let (mut i, mut e) = setup();
    eval_str(
        "(defun sum-to (n) (loop [k n acc 0] (if (= k 0) acc (recur (- k 1) (+ acc k)))))",
        &mut i,
        &mut e,
    )
    .await;
    let func = e.borrow().get("sum-to").expect("sum-to defined");
    match func {
        Value::UserFunc { jit_code, .. } => {
            assert!(
                jit_code.is_some(),
                "loop/recur function should pass the JIT gate"
            );
        }
        other => panic!("expected UserFunc, got {other:?}"),
    }

    // Invoking it executes the natively compiled loop.
    let res = eval_str("(sum-to 100)", &mut i, &mut e).await;
    assert_eq!(res, Value::Integer(5050));
}
