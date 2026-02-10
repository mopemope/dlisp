use dlisp_core::ast::Value;
use dlisp_core::interpreter::default_interpreter;
use dlisp_core::parser::parse;
use std::cell::RefCell;
use std::rc::Rc;

fn setup() -> (
    dlisp_core::interpreter::Interpreter,
    Rc<RefCell<dlisp_core::environment::Environment>>,
) {
    let interpreter = default_interpreter();
    let env = dlisp_core::interpreter::default_env();
    (interpreter, env)
}

async fn eval_str(
    src: &str,
    interpreter: &mut dlisp_core::interpreter::Interpreter,
    env: &mut Rc<RefCell<dlisp_core::environment::Environment>>,
) -> Value {
    let exprs = parse(src).unwrap();
    interpreter.eval(exprs[0].clone(), env).await.unwrap()
}

#[tokio::test]
async fn test_setq_local() {
    let (mut interpreter, mut env) = setup();

    // (let ((x 1)) (setq x 2) x) -> 2
    let src = "(let ((x 1)) (setq x 2) x)";
    let res = eval_str(src, &mut interpreter, &mut env).await;
    assert_eq!(res, Value::Integer(2));
}

#[tokio::test]
async fn test_setq_closure() {
    let (mut interpreter, mut env) = setup();

    // (let ((x 1)) ((lambda () (setq x 2))) x) -> 2
    let src = "(let ((x 1)) ((lambda () (setq x 2))) x)";
    let res = eval_str(src, &mut interpreter, &mut env).await;
    assert_eq!(res, Value::Integer(2));
}

#[tokio::test]
async fn test_setq_global() {
    let (mut interpreter, mut env) = setup();

    // (defvar g 10) (setq g 20) g -> 20
    eval_str("(defvar g 10)", &mut interpreter, &mut env).await;
    eval_str("(setq g 20)", &mut interpreter, &mut env).await;
    let res = eval_str("g", &mut interpreter, &mut env).await;
    assert_eq!(res, Value::Integer(20));
}

#[tokio::test]
async fn test_setq_undefined_error() {
    let (mut interpreter, mut env) = setup();

    // (setq undefined 1) -> Error
    let src = "(setq undefined 1)";
    let exprs = parse(src).unwrap();
    let res = interpreter.eval(exprs[0].clone(), &mut env).await;
    assert!(res.is_err());
}
