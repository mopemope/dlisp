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

#[tokio::test]
async fn test_map_basic() {
    let (mut i, mut e) = setup();
    let res = eval_str("(map (lambda (x) (* x 2)) '(1 2 3))", &mut i, &mut e).await;
    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(2),
            Value::Integer(4),
            Value::Integer(6)
        ])
    );
}

#[tokio::test]
async fn test_map_empty() {
    let (mut i, mut e) = setup();
    let res = eval_str("(map (lambda (x) (* x 2)) '())", &mut i, &mut e).await;
    assert_eq!(res, Value::List(vec![]));

    let res_nil = eval_str("(map (lambda (x) (* x 2)) nil)", &mut i, &mut e).await;
    assert_eq!(res_nil, Value::Nil);
}

#[tokio::test]
async fn test_map_named_func() {
    let (mut i, mut e) = setup();
    eval_str("(defun double (x) (* x 2))", &mut i, &mut e).await;
    let res = eval_str("(map double '(10 20))", &mut i, &mut e).await;
    assert_eq!(
        res,
        Value::List(vec![Value::Integer(20), Value::Integer(40)])
    );
}

#[tokio::test]
async fn test_filter_basic() {
    let (mut i, mut e) = setup();
    let res = eval_str("(filter (lambda (x) (> x 2)) '(1 2 3 4 5))", &mut i, &mut e).await;
    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5)
        ])
    );
}

#[tokio::test]
async fn test_filter_all_out() {
    let (mut i, mut e) = setup();
    let res = eval_str("(filter (lambda (x) (> x 10)) '(1 2 3))", &mut i, &mut e).await;
    assert_eq!(res, Value::List(vec![]));
}

#[tokio::test]
async fn test_filter_empty() {
    let (mut i, mut e) = setup();
    let res = eval_str("(filter (lambda (x) (> x 0)) '())", &mut i, &mut e).await;
    assert_eq!(res, Value::List(vec![]));
}

#[tokio::test]
async fn test_reduce_basic() {
    let (mut i, mut e) = setup();
    let res = eval_str("(reduce + 0 '(1 2 3 4 5))", &mut i, &mut e).await;
    assert_eq!(res, Value::Integer(15));
}

#[tokio::test]
async fn test_reduce_mul() {
    let (mut i, mut e) = setup();
    let res = eval_str("(reduce * 1 '(1 2 3 4))", &mut i, &mut e).await;
    assert_eq!(res, Value::Integer(24));
}

#[tokio::test]
async fn test_reduce_lambda() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(reduce (lambda (acc x) (+ acc x)) 10 '(1 2 3))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Integer(16));
}

#[tokio::test]
async fn test_reduce_empty() {
    let (mut i, mut e) = setup();
    let res = eval_str("(reduce + 42 '())", &mut i, &mut e).await;
    assert_eq!(res, Value::Integer(42));
}

#[tokio::test]
async fn test_higher_order_composition() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(reduce + 0 (map (lambda (x) (* x x)) (filter (lambda (x) (> x 2)) '(1 2 3 4))))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Integer(25));
}

// --- Error Cases & Edge Cases ---

#[tokio::test]
async fn test_map_wrong_args() {
    let (mut i, mut e) = setup();
    let exprs = parse("(map (lambda (x) x))").unwrap(); // Missing list
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("exactly 2 arguments"));
}

#[tokio::test]
async fn test_map_not_a_list() {
    let (mut i, mut e) = setup();
    let exprs = parse("(map (lambda (x) x) 42)").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("expects a list"));
}

#[tokio::test]
async fn test_map_not_a_func() {
    let (mut i, mut e) = setup();
    let exprs = parse("(map 42 '(1 2 3))").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("not a function"));
}

#[tokio::test]
async fn test_filter_truthiness() {
    let (mut i, mut e) = setup();
    // In DLisp, everything except nil, false, 0 is truthy.
    // Let's test filter with 0 and nil.
    let res = eval_str(
        "(filter (lambda (x) (if (= x 2) 0 1)) '(1 2 3))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::List(vec![Value::Integer(1), Value::Integer(3)]));
}

#[tokio::test]
async fn test_reduce_error_in_func() {
    let (mut i, mut e) = setup();
    let exprs = parse("(reduce (lambda (acc x) (/ acc 0)) 10 '(1 2 3))").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("zero"));
}

#[tokio::test]
async fn test_filter_wrong_args() {
    let (mut i, mut e) = setup();
    let exprs = parse("(filter (lambda (x) x))").unwrap(); // Missing list
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("exactly 2 arguments"));
}

#[tokio::test]
async fn test_filter_not_a_list() {
    let (mut i, mut e) = setup();
    let exprs = parse("(filter (lambda (x) x) 42)").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("expects a list"));
}

#[tokio::test]
async fn test_filter_not_a_func() {
    let (mut i, mut e) = setup();
    let exprs = parse("(filter 42 '(1 2 3))").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("not a function"));
}

#[tokio::test]
async fn test_reduce_wrong_args() {
    let (mut i, mut e) = setup();
    let exprs = parse("(reduce + 0)").unwrap(); // Missing list
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("exactly 3 arguments"));
}

#[tokio::test]
async fn test_reduce_not_a_list() {
    let (mut i, mut e) = setup();
    let exprs = parse("(reduce + 0 42)").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("expects a list"));
}

#[tokio::test]
async fn test_reduce_not_a_func() {
    let (mut i, mut e) = setup();
    let exprs = parse("(reduce 42 0 '(1 2 3))").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("not a function"));
}

#[tokio::test]
async fn test_map_vector() {
    let (mut i, mut e) = setup();
    let res = eval_str("(map (lambda (x) (* x 2)) [1 2 3])", &mut i, &mut e).await;
    assert_eq!(
        res,
        Value::Vector(vec![
            Value::Integer(2),
            Value::Integer(4),
            Value::Integer(6)
        ])
    );
}

#[tokio::test]
async fn test_filter_vector() {
    let (mut i, mut e) = setup();
    let res = eval_str("(filter (lambda (x) (> x 2)) [1 2 3 4 5])", &mut i, &mut e).await;
    assert_eq!(
        res,
        Value::Vector(vec![
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5)
        ])
    );
}

#[tokio::test]
async fn test_reduce_vector() {
    let (mut i, mut e) = setup();
    let res = eval_str("(reduce + 0 [1 2 3 4 5])", &mut i, &mut e).await;
    assert_eq!(res, Value::Integer(15));
}
