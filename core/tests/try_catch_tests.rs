use dlisp_core::ast::Value;
use dlisp_core::interpreter::{default_env, default_interpreter};

async fn run_code(code: &str) -> Result<Value, String> {
    let mut interpreter = default_interpreter();
    let env = default_env();

    // Evaluate in a test environment
    let mut last_val = Value::Nil;
    let stmts = dlisp_core::parser::parse(code).map_err(|e| format!("{:?}", e))?;
    for stmt in stmts {
        last_val = interpreter.eval(stmt, &mut env.clone()).await?;
    }
    Ok(last_val)
}

#[tokio::test]
async fn test_try_catch_success() {
    let code = r#"
        (try
            (+ 1 2)
            (catch e e))
    "#;
    let res = run_code(code).await.unwrap();
    assert_eq!(res, Value::Integer(3));
}

#[tokio::test]
async fn test_try_catch_error() {
    let code = r#"
        (try
            (throw "my-error")
            (+ 1 2)
            (catch e e))
    "#;
    let res = run_code(code).await.unwrap();
    match res {
        Value::Error(inner) => {
            assert_eq!(*inner, Value::String("my-error".to_string()));
        }
        _ => panic!("Expected Error value, got {:?}", res),
    }
}

#[tokio::test]
async fn test_type_of_error() {
    let code = r#"
        (try
            (throw 42)
            (catch e (type-of e)))
    "#;
    let res = run_code(code).await.unwrap();
    assert_eq!(res, Value::String("error".to_string()));
}

#[tokio::test]
async fn test_is_error() {
    let code = r#"
        (try
            (throw "err")
            (catch e (error? e)))
    "#;
    let res = run_code(code).await.unwrap();
    assert_eq!(res, Value::Bool(true));
}

#[tokio::test]
async fn test_error_value() {
    let code = r#"
        (try
            (throw "specific error")
            (catch e (error-value e)))
    "#;
    let res = run_code(code).await.unwrap();
    assert_eq!(res, Value::String("specific error".to_string()));
}

#[tokio::test]
async fn test_throw_integer() {
    let code = r#"
        (try
            (throw 42)
            (catch e e))
    "#;
    let res = run_code(code).await.unwrap();
    if let Value::Error(inner) = res {
        assert_eq!(*inner, Value::Integer(42));
    } else {
        panic!("Expected Error value");
    }
}

#[tokio::test]
async fn test_throw_list() {
    let code = r#"
        (try
            (throw '(1 2 3))
            (catch e e))
    "#;
    let res = run_code(code).await.unwrap();
    if let Value::Error(inner) = res {
        assert_eq!(
            *inner,
            Value::List(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3)
            ])
        );
    } else {
        panic!("Expected Error value, got {:?}", res);
    }
}

#[tokio::test]
async fn test_throw_keyword() {
    let code = r#"
        (try
            (throw :fatal-error)
            (catch e e))
    "#;
    let res = run_code(code).await.unwrap();
    if let Value::Error(inner) = res {
        assert_eq!(*inner, Value::Keyword("fatal-error".to_string()));
    } else {
        panic!("Expected Error value");
    }
}
