use dlisp_core::ast::Value;
use dlisp_core::interpreter::{Interpreter, default_env};

#[tokio::test]
async fn test_eval_add() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (+ 1 2)
    let ast = Value::List(vec![
        Value::Symbol("+".to_string()),
        Value::Integer(1),
        Value::Integer(2),
    ]);
    let res = interpreter.eval(ast, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(3));
}

#[tokio::test]
async fn test_eval_nested() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (+ 1 (+ 2 3))
    let ast = Value::List(vec![
        Value::Symbol("+".to_string()),
        Value::Integer(1),
        Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Integer(2),
            Value::Integer(3),
        ]),
    ]);
    let res = interpreter.eval(ast, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(6));
}

#[tokio::test]
async fn test_eval_defun() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
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
async fn test_eval_defun_3args() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (defun add3 (x y z) (+ x (+ y z)))
    let body_expr = Value::List(vec![
        Value::Symbol("+".to_string()),
        Value::Symbol("x".to_string()),
        Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Symbol("y".to_string()),
            Value::Symbol("z".to_string()),
        ]),
    ]);

    let defun_expr = Value::List(vec![
        Value::Symbol("defun".to_string()),
        Value::Symbol("add3".to_string()),
        Value::List(vec![
            Value::Symbol("x".to_string()),
            Value::Symbol("y".to_string()),
            Value::Symbol("z".to_string()),
        ]),
        body_expr,
    ]);
    interpreter
        .eval(defun_expr, &mut env.clone())
        .await
        .unwrap();

    // (add3 1 2 3)
    let call_expr = Value::List(vec![
        Value::Symbol("add3".to_string()),
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ]);
    let res = interpreter.eval(call_expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(6));
}

#[tokio::test]
async fn test_eval_spawn() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            let env = default_env();
            let mut interpreter = Interpreter::new();

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

            // Allow the spawned task to execute
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        })
        .await;
}

#[tokio::test]
async fn test_eval_let() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (let ((x 10) (y 20)) (+ x y))
    let let_expr = Value::List(vec![
        Value::Symbol("let".to_string()),
        Value::List(vec![
            Value::List(vec![Value::Symbol("x".to_string()), Value::Integer(10)]),
            Value::List(vec![Value::Symbol("y".to_string()), Value::Integer(20)]),
        ]),
        Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Symbol("x".to_string()),
            Value::Symbol("y".to_string()),
        ]),
    ]);
    let res = interpreter.eval(let_expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(30));
}

#[tokio::test]
async fn test_eval_let_parallel_binding() {
    let env = default_env();
    // Define x = 100 in outer scope
    env.borrow_mut().set("x".to_string(), Value::Integer(100));

    let mut interpreter = Interpreter::new();
    // (let ((x 1) (y x)) y)
    // If sequential, y would be 1. If parallel, y should be 100.
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
async fn test_eval_let_destructuring() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (let (((x y) '(10 20))) (+ x y))
    let let_expr = Value::List(vec![
        Value::Symbol("let".to_string()),
        Value::List(vec![Value::List(vec![
            Value::List(vec![
                Value::Symbol("x".to_string()),
                Value::Symbol("y".to_string()),
            ]),
            Value::List(vec![
                Value::Symbol("quote".to_string()),
                Value::List(vec![Value::Integer(10), Value::Integer(20)]),
            ]),
        ])]),
        Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Symbol("x".to_string()),
            Value::Symbol("y".to_string()),
        ]),
    ]);
    let res = interpreter.eval(let_expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(30));
}

#[tokio::test]
async fn test_eval_let_destructuring_rest() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (let (((x &rest y) '(1 2 3))) y) => (2 3)
    let let_expr = Value::List(vec![
        Value::Symbol("let".to_string()),
        Value::List(vec![Value::List(vec![
            Value::List(vec![
                Value::Symbol("x".to_string()),
                Value::Symbol("&rest".to_string()),
                Value::Symbol("y".to_string()),
            ]),
            Value::List(vec![
                Value::Symbol("quote".to_string()),
                Value::List(vec![
                    Value::Integer(1),
                    Value::Integer(2),
                    Value::Integer(3),
                ]),
            ]),
        ])]),
        Value::Symbol("y".to_string()),
    ]);
    let res = interpreter.eval(let_expr, &mut env.clone()).await;
    assert_eq!(
        res.unwrap(),
        Value::List(vec![Value::Integer(2), Value::Integer(3)])
    );
}
#[tokio::test]
async fn test_eval_let_empty_bindings() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (let () 1)
    let let_expr = Value::List(vec![
        Value::Symbol("let".to_string()),
        Value::List(vec![]),
        Value::Integer(1),
    ]);
    let res = interpreter.eval(let_expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(1));
}

#[tokio::test]
async fn test_eval_let_shadowing() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
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
async fn test_eval_let_multiple_body() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (let ((x 10)) (+ x 1) (+ x 2)) -> 12
    let let_expr = Value::List(vec![
        Value::Symbol("let".to_string()),
        Value::List(vec![Value::List(vec![
            Value::Symbol("x".to_string()),
            Value::Integer(10),
        ])]),
        Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Symbol("x".to_string()),
            Value::Integer(1),
        ]),
        Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Symbol("x".to_string()),
            Value::Integer(2),
        ]),
    ]);
    let res = interpreter.eval(let_expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(12));
}

#[tokio::test]
async fn test_eval_let_parallel_undefined() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (let ((x 1) (y x)) y) -> Error because x is not defined in outer env
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
    assert!(res.is_err());
}

#[tokio::test]
async fn test_eval_if_true() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (if 1 2 3) -> 2
    let if_expr = Value::List(vec![
        Value::Symbol("if".to_string()),
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ]);
    let res = interpreter.eval(if_expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(2));
}

#[tokio::test]
async fn test_eval_if_false() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (if 0 2 3) -> 3 (Assuming 0 is false)
    let if_expr = Value::List(vec![
        Value::Symbol("if".to_string()),
        Value::Integer(0),
        Value::Integer(2),
        Value::Integer(3),
    ]);
    let res = interpreter.eval(if_expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(3));
}

#[tokio::test]
async fn test_eval_if_nil() {
    let env = default_env();
    let mut interpreter = Interpreter::new();
    // (if nil 2 3) -> 3
    let if_expr = Value::List(vec![
        Value::Symbol("if".to_string()),
        Value::Nil,
        Value::Integer(2),
        Value::Integer(3),
    ]);
    let res = interpreter.eval(if_expr, &mut env.clone()).await;
    assert_eq!(res.unwrap(), Value::Integer(3));
}
