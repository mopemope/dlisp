use dlisp_core::ast::Value;
use dlisp_core::environment::Environment;
use dlisp_core::interpreter::default_interpreter;
use std::cell::RefCell;
use std::rc::Rc;

#[tokio::test]
async fn test_eval_vector_literal() {
    let mut interp = default_interpreter();
    let mut env = Rc::new(RefCell::new(Environment::new(None)));

    // Test [1 2 3]
    let vec_val = Value::Vector(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ]);

    let result = interp.eval(vec_val, &mut env).await.expect("Eval failed");
    if let Value::Vector(v) = result {
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], Value::Integer(1));
        assert_eq!(v[1], Value::Integer(2));
        assert_eq!(v[2], Value::Integer(3));
    } else {
        panic!("Expected Vector, got {:?}", result);
    }
}

#[tokio::test]
async fn test_eval_nested_vector() {
    let mut interp = default_interpreter();
    let mut env = Rc::new(RefCell::new(Environment::new(None)));

    // Test [1 [2]]
    let vec_val = Value::Vector(vec![
        Value::Integer(1),
        Value::Vector(vec![Value::Integer(2)]),
    ]);

    let result = interp.eval(vec_val, &mut env).await.expect("Eval failed");
    if let Value::Vector(v) = result {
        assert_eq!(v.len(), 2);
        if let Value::Vector(inner) = &v[1] {
            assert_eq!(inner[0], Value::Integer(2));
        } else {
            panic!("Expected nested vector");
        }
    } else {
        panic!("Expected Vector");
    }
}

async fn eval_str(src: &str) -> Value {
    let mut interp = default_interpreter();
    let mut env = dlisp_core::interpreter::default_env();
    let exprs = dlisp_core::parser::parse(src).unwrap();
    interp.eval(exprs[0].clone(), &mut env).await.unwrap()
}

#[tokio::test]
async fn test_vector_count() {
    assert_eq!(eval_str("(count [1 2 3])").await, Value::Integer(3));
    assert_eq!(eval_str("(count [])").await, Value::Integer(0));
}

#[tokio::test]
async fn test_vector_nth() {
    assert_eq!(eval_str("(nth [10 20 30] 1)").await, Value::Integer(20));
    assert_eq!(eval_str("(nth [10 20] 5)").await, Value::Nil);
}

#[tokio::test]
async fn test_vector_conj() {
    let val = eval_str("(conj [1 2] 3 4)").await;
    if let Value::Vector(v) = val {
        assert_eq!(v.len(), 4);
        assert_eq!(
            v,
            vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
                Value::Integer(4)
            ]
        );
    } else {
        panic!("Expected Vector");
    }
}
