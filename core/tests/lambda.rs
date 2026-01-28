use dlisp_core::ast::Value;
use dlisp_core::interpreter::{Interpreter, default_env};

#[tokio::test]
async fn test_lambda_identity() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // ((lambda (x) x) 42)
    let expr = Value::List(vec![
        Value::List(vec![
            Value::Symbol("lambda".to_string()),
            Value::List(vec![Value::Symbol("x".to_string())]),
            Value::Symbol("x".to_string()),
        ]),
        Value::Integer(42),
    ]);
    let res = interpreter.eval(expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(42));
}

#[tokio::test]
async fn test_lambda_closure_capture() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (let ((x 10)) ((lambda (y) (+ x y)) 5))
    let expr = Value::List(vec![
        Value::Symbol("let".to_string()),
        Value::List(vec![Value::List(vec![
            Value::Symbol("x".to_string()),
            Value::Integer(10),
        ])]),
        Value::List(vec![
            Value::List(vec![
                Value::Symbol("lambda".to_string()),
                Value::List(vec![Value::Symbol("y".to_string())]),
                Value::List(vec![
                    Value::Symbol("+".to_string()),
                    Value::Symbol("x".to_string()),
                    Value::Symbol("y".to_string()),
                ]),
            ]),
            Value::Integer(5),
        ]),
    ]);
    let res = interpreter.eval(expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(15));
}

#[tokio::test]
async fn test_lambda_shadowing() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (let ((x 10)) ((lambda (x) x) 20)) -> 20 (inner x shadows outer x)
    let expr = Value::List(vec![
        Value::Symbol("let".to_string()),
        Value::List(vec![Value::List(vec![
            Value::Symbol("x".to_string()),
            Value::Integer(10),
        ])]),
        Value::List(vec![
            Value::List(vec![
                Value::Symbol("lambda".to_string()),
                Value::List(vec![Value::Symbol("x".to_string())]),
                Value::Symbol("x".to_string()),
            ]),
            Value::Integer(20),
        ]),
    ]);
    let res = interpreter.eval(expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(20));
}

#[tokio::test]
async fn test_lambda_nested_closure() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (let ((x 1)) (let ((y 2)) ((lambda (z) (+ x (+ y z))) 3))) -> 6
    let expr = Value::List(vec![
        Value::Symbol("let".to_string()),
        Value::List(vec![Value::List(vec![
            Value::Symbol("x".to_string()),
            Value::Integer(1),
        ])]),
        Value::List(vec![
            Value::Symbol("let".to_string()),
            Value::List(vec![Value::List(vec![
                Value::Symbol("y".to_string()),
                Value::Integer(2),
            ])]),
            Value::List(vec![
                Value::List(vec![
                    Value::Symbol("lambda".to_string()),
                    Value::List(vec![Value::Symbol("z".to_string())]),
                    Value::List(vec![
                        Value::Symbol("+".to_string()),
                        Value::Symbol("x".to_string()),
                        Value::List(vec![
                            Value::Symbol("+".to_string()),
                            Value::Symbol("y".to_string()),
                            Value::Symbol("z".to_string()),
                        ]),
                    ]),
                ]),
                Value::Integer(3),
            ]),
        ]),
    ]);
    let res = interpreter.eval(expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(6));
}

#[tokio::test]
async fn test_high_order_function() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (defun apply-func (f x) (f x))
    let defun = Value::List(vec![
        Value::Symbol("defun".to_string()),
        Value::Symbol("apply-func".to_string()),
        Value::List(vec![
            Value::Symbol("f".to_string()),
            Value::Symbol("x".to_string()),
        ]),
        Value::List(vec![
            Value::Symbol("f".to_string()),
            Value::Symbol("x".to_string()),
        ]),
    ]);
    interpreter.eval(defun, &mut env.clone()).await.unwrap();

    // (apply-func (lambda (a) (+ a 1)) 10)
    let expr = Value::List(vec![
        Value::Symbol("apply-func".to_string()),
        Value::List(vec![
            Value::Symbol("lambda".to_string()),
            Value::List(vec![Value::Symbol("a".to_string())]),
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Symbol("a".to_string()),
                Value::Integer(1),
            ]),
        ]),
        Value::Integer(10),
    ]);
    let res = interpreter.eval(expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(11));
}
