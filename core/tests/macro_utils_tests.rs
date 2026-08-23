use dlisp_core::ast::Value;
use dlisp_core::interpreter::{Interpreter, default_env};
use dlisp_core::parser::parse;

async fn run_code(code: &str) -> Result<Value, dlisp_core::eval_failure::EvalFailure> {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();
    let parsed = parse(code).map_err(|e| format!("{:?}", e))?;
    let mut result = Value::Nil;
    for stmt in parsed {
        result = interpreter.eval(stmt, &mut env).await?;
    }
    Ok(result)
}

// --- gensym tests ---

#[tokio::test]
async fn test_gensym_default_prefix() {
    let res = run_code("(gensym)").await.unwrap();
    if let Value::Symbol(s) = res {
        assert!(s.starts_with("G__"));
    } else {
        panic!("gensym should return a symbol");
    }
}

#[tokio::test]
async fn test_gensym_custom_prefix() {
    let res = run_code("(gensym \"MY-\")").await.unwrap();
    if let Value::Symbol(s) = res {
        assert!(s.starts_with("MY-"));
    } else {
        panic!("gensym should return a symbol");
    }
}

#[tokio::test]
async fn test_gensym_uniqueness() {
    let sym1 = run_code("(gensym)").await.unwrap();
    let sym2 = run_code("(gensym)").await.unwrap();
    assert_ne!(sym1, sym2);
}

#[tokio::test]
async fn test_gensym_too_many_args() {
    let result = run_code("(gensym \"a\" \"b\")").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_gensym_invalid_type() {
    let result = run_code("(gensym 42)").await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("must be a string or symbol")
    );
}

// --- macroexpand tests ---

#[tokio::test]
async fn test_macroexpand_basic() {
    let code = r#"
        (defmacro my-inc (x)
          (list '+ x 1))
        (macroexpand '(my-inc 5))
    "#;
    let res = run_code(code).await.unwrap();
    assert_eq!(
        res,
        Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Integer(5),
            Value::Integer(1),
        ])
    );
}

#[tokio::test]
async fn test_macroexpand_non_macro() {
    // Expanding a non-macro form should return it unchanged
    let code = "(macroexpand '(+ 1 2))";
    let res = run_code(code).await.unwrap();
    assert_eq!(
        res,
        Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Integer(1),
            Value::Integer(2),
        ])
    );
}

#[tokio::test]
async fn test_macroexpand_atom() {
    // Expanding an atom should return it unchanged
    let code = "(macroexpand '42)";
    let res = run_code(code).await.unwrap();
    assert_eq!(res, Value::Integer(42));
}

#[tokio::test]
async fn test_macroexpand_with_backquote_macro() {
    let code = r#"
        (defmacro when (cond body)
          `(if ,cond ,body nil))
        (macroexpand '(when true 42))
    "#;
    let res = run_code(code).await.unwrap();
    assert_eq!(
        res,
        Value::List(vec![
            Value::Symbol("if".to_string()),
            Value::Bool(true),
            Value::Integer(42),
            Value::Nil,
        ])
    );
}

#[tokio::test]
async fn test_macroexpand_with_rest_param() {
    let code = r#"
        (defmacro my-list (name &rest items)
          `(list ,name ,@items))
        (macroexpand '(my-list "nums" 1 2 3))
    "#;
    let res = run_code(code).await.unwrap();
    assert_eq!(
        res,
        Value::List(vec![
            Value::Symbol("list".to_string()),
            Value::String("nums".to_string()),
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
}

#[tokio::test]
async fn test_macroexpand_with_rest_param_empty() {
    let code = r#"
        (defmacro my-list (name &rest items)
          `(list ,name ,@items))
        (macroexpand '(my-list "empty"))
    "#;
    let res = run_code(code).await.unwrap();
    assert_eq!(
        res,
        Value::List(vec![
            Value::Symbol("list".to_string()),
            Value::String("empty".to_string()),
        ])
    );
}
