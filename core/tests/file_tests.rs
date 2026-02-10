use dlisp_core::ast::Value;
use dlisp_core::interpreter::{Interpreter, default_env};
use dlisp_core::parser::parse;
use std::fs;
use std::path::PathBuf;

async fn run_file(name: &str) -> Result<Value, String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // root
    path.push("example-lisp");
    path.push(name);

    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut env = default_env();
    let mut interpreter = Interpreter::new();

    let vals = parse(&content).map_err(|e| format!("{:?}", e))?;
    let mut last_res = Value::Nil;
    for val in vals {
        last_res = interpreter.eval(val, &mut env).await?;
    }

    // Check for main
    let main_val = env.borrow().get("main");
    if let Some(main_val @ Value::UserFunc { .. }) = main_val {
        last_res = interpreter.apply(main_val, vec![], &mut env).await?;
    }

    Ok(last_res)
}

#[tokio::test]
async fn test_hello_file() {
    let res = run_file("hello.lisp").await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_factorial_file() {
    let res = run_file("factorial.lisp").await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_fib_file() {
    let res = run_file("fib.lisp").await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_macros_file() {
    let res = run_file("macros.lisp").await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_mutual_recursion_file() {
    let res = run_file("mutual_recursion.lisp").await;
    assert!(res.is_ok());
}
