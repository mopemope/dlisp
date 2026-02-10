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
    let mut result = Value::Nil;
    for expr in exprs {
        result = interpreter.eval(expr, env).await.unwrap();
    }
    result
}

// --- progn ---

#[tokio::test]
async fn test_progn_basic() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(progn 1 2 3)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(3));
}

#[tokio::test]
async fn test_progn_empty() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(progn)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_progn_side_effects() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        "(let ((x 0)) (progn (setq x 1) (setq x (+ x 10)) x))",
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(res, Value::Integer(11));
}

#[tokio::test]
async fn test_do_alias() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(do 1 2 42)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(42));
}

// --- cond ---

#[tokio::test]
async fn test_cond_first_true() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(cond ((= 1 1) 10) ((= 1 2) 20))", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(10));
}

#[tokio::test]
async fn test_cond_second_true() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(cond ((= 1 2) 10) ((= 1 1) 20))", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(20));
}

#[tokio::test]
async fn test_cond_none_true() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(cond ((= 1 2) 10) ((= 3 4) 20))", &mut interp, &mut env).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_cond_default_clause() {
    let (mut interp, mut env) = setup();
    // true as the test always matches
    let res = eval_str("(cond ((= 1 2) 10) (1 99))", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(99));
}

#[tokio::test]
async fn test_cond_multi_body() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        "(let ((x 0)) (cond ((= 1 1) (setq x 5) (+ x 10))))",
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(res, Value::Integer(15));
}

// --- and ---

#[tokio::test]
async fn test_and_all_truthy() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(and 1 2 3)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(3));
}

#[tokio::test]
async fn test_and_short_circuit() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(and 1 nil 3)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_and_empty() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(and)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Bool(true));
}

#[tokio::test]
async fn test_and_zero() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(and 1 0 3)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(0));
}

// --- or ---

#[tokio::test]
async fn test_or_first_truthy() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(or nil 0 42)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(42));
}

#[tokio::test]
async fn test_or_all_falsy() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(or nil 0)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(0));
}

#[tokio::test]
async fn test_or_empty() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(or)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_or_short_circuit() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(or 42 nil)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(42));
}

// --- not ---

#[tokio::test]
async fn test_not_nil() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(not nil)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Bool(true));
}

#[tokio::test]
async fn test_not_zero() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(not 0)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Bool(true));
}

#[tokio::test]
async fn test_not_truthy() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(not 42)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Bool(false));
}

// --- car ---

#[tokio::test]
async fn test_car_basic() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(car '(1 2 3))", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(1));
}

#[tokio::test]
async fn test_car_nil() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(car nil)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_first_alias() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(first '(10 20 30))", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(10));
}

// --- cdr ---

#[tokio::test]
async fn test_cdr_basic() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(cdr '(1 2 3))", &mut interp, &mut env).await;
    assert_eq!(res, Value::List(vec![Value::Integer(2), Value::Integer(3)]));
}

#[tokio::test]
async fn test_cdr_single() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(cdr '(1))", &mut interp, &mut env).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_rest_alias() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(rest '(10 20))", &mut interp, &mut env).await;
    assert_eq!(res, Value::List(vec![Value::Integer(20)]));
}

// --- cons ---

#[tokio::test]
async fn test_cons_to_list() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(cons 1 '(2 3))", &mut interp, &mut env).await;
    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3)
        ])
    );
}

#[tokio::test]
async fn test_cons_to_nil() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(cons 1 nil)", &mut interp, &mut env).await;
    assert_eq!(res, Value::List(vec![Value::Integer(1)]));
}

#[tokio::test]
async fn test_cons_to_non_list() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(cons 1 2)", &mut interp, &mut env).await;
    assert_eq!(res, Value::List(vec![Value::Integer(1), Value::Integer(2)]));
}

// --- Combined usage ---

#[tokio::test]
async fn test_car_cdr_cons_roundtrip() {
    let (mut interp, mut env) = setup();
    // (cons (car '(1 2 3)) (cdr '(10 20 30))) => (1 20 30)
    let res = eval_str(
        "(cons (car '(1 2 3)) (cdr '(10 20 30)))",
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(1),
            Value::Integer(20),
            Value::Integer(30)
        ])
    );
}

#[tokio::test]
async fn test_progn_with_when_macro() {
    let (mut interp, mut env) = setup();
    // when macro using progn for multiple body forms
    eval_str(
        "(defmacro when2 (cond body1 body2) (list 'if cond (list 'progn body1 body2) 'nil))",
        &mut interp,
        &mut env,
    )
    .await;
    let res = eval_str(
        "(let ((x 0)) (when2 (> 10 5) (setq x 1) (+ x 100)))",
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(res, Value::Integer(101));
}

// --- Bool(false) edge cases ---

#[tokio::test]
async fn test_and_bool_false() {
    let (mut interp, mut env) = setup();
    // (and false 42) should short-circuit on false
    let res = eval_str("(and false 42)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Bool(false));
}

#[tokio::test]
async fn test_or_bool_false() {
    let (mut interp, mut env) = setup();
    // (or false 42) should skip false and return 42
    let res = eval_str("(or false 42)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(42));
}

#[tokio::test]
async fn test_cond_bool_false_condition() {
    let (mut interp, mut env) = setup();
    // cond with false as condition should skip to next clause
    let res = eval_str(
        "(cond (false \"wrong\") (true \"right\"))",
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(res, Value::String("right".to_string()));
}

#[tokio::test]
async fn test_not_bool_false() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(not false)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Bool(true));
}

#[tokio::test]
async fn test_not_bool_true() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(not true)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Bool(false));
}

#[tokio::test]
async fn test_if_bool_false() {
    let (mut interp, mut env) = setup();
    let res = eval_str("(if false \"then\" \"else\")", &mut interp, &mut env).await;
    assert_eq!(res, Value::String("else".to_string()));
}
