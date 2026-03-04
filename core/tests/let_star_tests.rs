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

#[tokio::test]
async fn test_let_star_basic() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(let* ((x 10) (y (* x 2))) (+ x y))", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(30));
}

#[tokio::test]
async fn test_let_star_chain() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        "(let* ((a 1) (b (+ a 1)) (c (+ b 1))) c)",
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(res, Value::Integer(3));
}

#[tokio::test]
async fn test_let_star_single_binding() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(let* ((x 42)) x)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(42));
}

#[tokio::test]
async fn test_let_star_empty_bindings() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(let* () 99)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(99));
}

#[tokio::test]
async fn test_let_star_shadowing() {
    let (mut interp, mut env) = setup();
    // x is 5 in the outer let, then x is shadowed to 10 in let*
    let res = eval_str(
        "(let ((x 5)) (let* ((x 10) (y x)) y))",
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(res, Value::Integer(10));
}

#[tokio::test]
async fn test_let_star_does_not_leak() {
    let (mut interp, mut env) = setup();
    // let* bindings should not be visible outside
    eval_str("(defvar outer 0)", &mut interp, &mut env).await;
    eval_str(
        "(let* ((inner 42)) (setq outer inner))",
        &mut interp,
        &mut env,
    )
    .await;
    let res = eval_str("outer", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(42));
}

#[tokio::test]
async fn test_let_star_multi_body() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        "(let* ((x 1)) (+ x 1) (+ x 2) (+ x 3))",
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(res, Value::Integer(4)); // last body form
}
