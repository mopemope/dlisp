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
async fn test_defvar_basic() {
    let (mut interpreter, mut env) = setup();

    // Define x = 10
    eval_str("(defvar x 10)", &mut interpreter, &mut env).await;
    assert_eq!(
        eval_str("x", &mut interpreter, &mut env).await,
        Value::Integer(10)
    );

    // Try to redefine x = 20, should stay 10
    eval_str("(defvar x 20)", &mut interpreter, &mut env).await;
    assert_eq!(
        eval_str("x", &mut interpreter, &mut env).await,
        Value::Integer(10)
    );
}

#[tokio::test]
async fn test_defvar_no_init() {
    let (mut interpreter, mut env) = setup();

    // Declare y without init
    eval_str("(defvar y)", &mut interpreter, &mut env).await;

    // In DLisp current env, getting an unbound symbol is an error.
    // But defvar with no init essentially does nothing if unbound.
    // So "y" should still be undefined.
    let exprs = parse("y").unwrap();
    let res = interpreter.eval(exprs[0].clone(), &mut env).await;
    assert!(res.is_err()); // Undefined symbol
}

#[tokio::test]
async fn test_defvar_eval_init() {
    let (mut interpreter, mut env) = setup();

    // Init value should be evaluated
    eval_str("(defvar z (+ 1 2))", &mut interpreter, &mut env).await;
    assert_eq!(
        eval_str("z", &mut interpreter, &mut env).await,
        Value::Integer(3)
    );
}

#[tokio::test]
async fn test_defvar_docstring() {
    let (mut interpreter, mut env) = setup();

    // With docstring
    eval_str(r#"(defvar w 100 "This is w")"#, &mut interpreter, &mut env).await;
    assert_eq!(
        eval_str("w", &mut interpreter, &mut env).await,
        Value::Integer(100)
    );
}

#[tokio::test]
async fn test_defvar_arg_counts() {
    let (mut interpreter, mut env) = setup();

    // Too few
    let res = try_eval_str("(defvar)", &mut interpreter, &mut env).await;
    assert!(res.is_err());

    // Too many
    let res = try_eval_str("(defvar a 1 \"doc\" :extra)", &mut interpreter, &mut env).await;
    assert!(res.is_err());
}

async fn try_eval_str(
    src: &str,
    interpreter: &mut dlisp_core::interpreter::Interpreter,
    env: &mut Rc<RefCell<dlisp_core::environment::Environment>>,
) -> Result<Value, dlisp_core::eval_failure::EvalFailure> {
    let exprs = parse(src).map_err(|e| format!("{:?}", e))?;
    interpreter.eval(exprs[0].clone(), env).await
}
