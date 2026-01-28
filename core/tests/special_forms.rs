use dlisp_core::ast::Value;
use dlisp_core::interpreter::{default_env, default_interpreter};

#[tokio::test]
async fn test_defun_basic() {
    let env = default_env();
    let mut interpreter = default_interpreter();

    // (defun add2 (x) (+ x 2))
    let defun_expr = Value::List(vec![
        Value::Symbol("defun".to_string()),
        Value::Symbol("add2".to_string()),
        Value::List(vec![Value::Symbol("x".to_string())]),
        Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Symbol("x".to_string()),
            Value::Integer(2),
        ]),
    ]);

    interpreter
        .eval(defun_expr, &mut env.clone())
        .await
        .unwrap();

    // (add2 3)
    let call_expr = Value::List(vec![Value::Symbol("add2".to_string()), Value::Integer(3)]);
    let res = interpreter.eval(call_expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(5));
}

#[tokio::test]
async fn test_defun_error_arg_count() {
    let env = default_env();
    let mut interpreter = default_interpreter();

    // (defun func (x) x)
    let defun_expr = Value::List(vec![
        Value::Symbol("defun".to_string()),
        Value::Symbol("func".to_string()),
        Value::List(vec![Value::Symbol("x".to_string())]),
        Value::Symbol("x".to_string()),
    ]);
    interpreter
        .eval(defun_expr, &mut env.clone())
        .await
        .unwrap();

    // (func 1 2) -> Should fail
    let call_expr = Value::List(vec![
        Value::Symbol("func".to_string()),
        Value::Integer(1),
        Value::Integer(2),
    ]);
    let res = interpreter.eval(call_expr, &mut env.clone()).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_let_sequential_vs_parallel() {
    let env = default_env();
    // Define x = 100 in outer scope
    env.borrow_mut().set("x".to_string(), Value::Integer(100));

    let mut interpreter = default_interpreter();
    // (let ((x 1) (y x)) y)
    // If parallel, y gets value of x from OUTER scope (100).
    // If sequential, y gets value of x from CURRENT binding (1).
    // Current impl is parallel.
    let let_expr = Value::List(vec![
        Value::Symbol("let".to_string()),
        Value::List(vec![
            Value::List(vec![Value::Symbol("x".to_string()), Value::Integer(1)]),
            Value::List(vec![
                Value::Symbol("y".to_string()),
                Value::Symbol("x".to_string()),
            ]),
        ]),
        Value::Symbol("y".to_string()),
    ]);
    let res = interpreter.eval(let_expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(100));
}

#[tokio::test]
async fn test_let_shadowing() {
    let env = default_env();
    let mut interpreter = default_interpreter();
    // (let ((x 10)) (let ((x 20)) x)) -> 20
    let let_expr = Value::List(vec![
        Value::Symbol("let".to_string()),
        Value::List(vec![Value::List(vec![
            Value::Symbol("x".to_string()),
            Value::Integer(10),
        ])]),
        Value::List(vec![
            Value::Symbol("let".to_string()),
            Value::List(vec![Value::List(vec![
                Value::Symbol("x".to_string()),
                Value::Integer(20),
            ])]),
            Value::Symbol("x".to_string()),
        ]),
    ]);
    let res = interpreter.eval(let_expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(20));
}

#[tokio::test]
async fn test_spawn_execution() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            let env = default_env();
            let mut interpreter = default_interpreter();

            // (defun task () (+ 1 2))
            let defun_expr = Value::List(vec![
                Value::Symbol("defun".to_string()),
                Value::Symbol("task".to_string()),
                Value::List(vec![]),
                Value::List(vec![
                    Value::Symbol("+".to_string()),
                    Value::Integer(1),
                    Value::Integer(2),
                ]),
            ]);
            interpreter
                .eval(defun_expr, &mut env.clone())
                .await
                .unwrap();

            // (spawn task)
            let spawn_expr = Value::List(vec![
                Value::Symbol("spawn".to_string()),
                Value::Symbol("task".to_string()),
            ]);

            let res = interpreter.eval(spawn_expr, &mut env.clone()).await;
            assert_eq!(res.unwrap(), Value::Nil);

            // Give it a moment to run
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        })
        .await;
}
