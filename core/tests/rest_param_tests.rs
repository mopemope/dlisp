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

// --- defun with &rest ---

#[tokio::test]
async fn test_defun_rest_basic() {
    let (mut interp, mut env) = setup();
    eval_str("(defun my-list (&rest args) args)", &mut interp, &mut env).await;
    let res = eval_str("(my-list 1 2 3)", &mut interp, &mut env).await;
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
async fn test_defun_rest_no_extra_args() {
    let (mut interp, mut env) = setup();
    eval_str("(defun my-list (&rest args) args)", &mut interp, &mut env).await;
    let res = eval_str("(my-list)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_defun_mixed_and_rest() {
    let (mut interp, mut env) = setup();
    eval_str(
        "(defun my-format (fmt &rest args) (list fmt args))",
        &mut interp,
        &mut env,
    )
    .await;
    let res = eval_str("(my-format \"hello\" 1 2 3)", &mut interp, &mut env).await;
    assert_eq!(
        res,
        Value::List(vec![
            Value::String("hello".to_string()),
            Value::List(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3)
            ])
        ])
    );
}

#[tokio::test]
async fn test_defun_mixed_rest_empty() {
    let (mut interp, mut env) = setup();
    eval_str(
        "(defun my-format (fmt &rest args) (list fmt args))",
        &mut interp,
        &mut env,
    )
    .await;
    let res = eval_str("(my-format \"hello\")", &mut interp, &mut env).await;
    assert_eq!(
        res,
        Value::List(vec![Value::String("hello".to_string()), Value::Nil])
    );
}

// --- lambda with &rest ---

#[tokio::test]
async fn test_lambda_rest_basic() {
    let (mut interp, mut env) = setup();
    let res = eval_str("((lambda (&rest xs) xs) 10 20 30)", &mut interp, &mut env).await;
    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(10),
            Value::Integer(20),
            Value::Integer(30)
        ])
    );
}

#[tokio::test]
async fn test_lambda_mixed_and_rest() {
    let (mut interp, mut env) = setup();
    let res = eval_str(
        "((lambda (a b &rest cs) (list a b cs)) 1 2 3 4 5)",
        &mut interp,
        &mut env,
    )
    .await;
    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::List(vec![
                Value::Integer(3),
                Value::Integer(4),
                Value::Integer(5)
            ])
        ])
    );
}

// --- apply with &rest ---

#[tokio::test]
async fn test_apply_with_rest_func() {
    let (mut interp, mut env) = setup();
    eval_str(
        "(defun my-sum (&rest nums) (apply + nums))",
        &mut interp,
        &mut env,
    )
    .await;
    let res = eval_str("(my-sum 1 2 3 4 5)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(15));
}

// --- edge cases ---

#[tokio::test]
async fn test_rest_too_few_args_error() {
    let (mut interp, mut env) = setup();
    eval_str(
        "(defun needs-one (x &rest rest) (list x rest))",
        &mut interp,
        &mut env,
    )
    .await;
    // should error with 0 args
    let exprs = parse("(needs-one)").unwrap();
    let result = interp.eval(exprs[0].clone(), &mut env).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_rest_with_higher_order() {
    let (mut interp, mut env) = setup();
    eval_str(
        "(defun sum-all (&rest nums) (reduce + 0 nums))",
        &mut interp,
        &mut env,
    )
    .await;
    let res = eval_str("(sum-all 10 20 30)", &mut interp, &mut env).await;
    assert_eq!(res, Value::Integer(60));
}
