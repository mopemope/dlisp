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
