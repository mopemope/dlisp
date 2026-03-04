use dlisp_core::ast::Value;
use dlisp_core::interpreter::{Interpreter, default_env};
use dlisp_core::parser::parse;

async fn run_code(code: &str) -> Result<Value, String> {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();
    let parsed = parse(code).map_err(|e| format!("{:?}", e))?;
    let mut result = Value::Nil;
    for stmt in parsed {
        result = interpreter.eval(stmt, &mut env).await?;
    }
    Ok(result)
}

// --- Basic backquote ---

#[tokio::test]
async fn test_backquote_literals() {
    // Integers are self-evaluating, should pass through
    assert_eq!(
        run_code("`(1 2 3)").await.unwrap(),
        Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
}

#[tokio::test]
async fn test_backquote_with_symbol() {
    // Symbols inside backquote should NOT be evaluated
    let res = run_code("`(a b c)").await.unwrap();
    assert_eq!(
        res,
        Value::List(vec![
            Value::Symbol("a".to_string()),
            Value::Symbol("b".to_string()),
            Value::Symbol("c".to_string()),
        ])
    );
}

#[tokio::test]
async fn test_backquote_atom() {
    // Backquote on a single atom should quote it
    assert_eq!(
        run_code("`a").await.unwrap(),
        Value::Symbol("a".to_string())
    );
    assert_eq!(run_code("`42").await.unwrap(), Value::Integer(42));
    assert_eq!(
        run_code("`\"hello\"").await.unwrap(),
        Value::String("hello".to_string())
    );
}

#[tokio::test]
async fn test_backquote_empty_list() {
    assert_eq!(run_code("`()").await.unwrap(), Value::Nil);
}

// --- Unquote ---

#[tokio::test]
async fn test_backquote_unquote() {
    let res = run_code("(let ((x 42)) `(1 2 ,x))").await.unwrap();
    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(42),
        ])
    );
}

#[tokio::test]
async fn test_backquote_unquote_expression() {
    // Unquote should evaluate the expression
    let res = run_code("`(a ,(+ 1 2))").await.unwrap();
    assert_eq!(
        res,
        Value::List(vec![Value::Symbol("a".to_string()), Value::Integer(3),])
    );
}

#[tokio::test]
async fn test_backquote_unquote_first_position() {
    let res = run_code("(let ((x 'foo)) `(,x 1 2))").await.unwrap();
    assert_eq!(
        res,
        Value::List(vec![
            Value::Symbol("foo".to_string()),
            Value::Integer(1),
            Value::Integer(2),
        ])
    );
}

// --- Unquote-splicing ---

#[tokio::test]
async fn test_backquote_unquote_splicing() {
    let res = run_code("(let ((x '(3 4))) `(1 2 ,@x 5))").await.unwrap();
    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
        ])
    );
}

#[tokio::test]
async fn test_backquote_splicing_at_beginning() {
    let res = run_code("(let ((x '(1 2))) `(,@x 3 4))").await.unwrap();
    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ])
    );
}

#[tokio::test]
async fn test_backquote_splicing_at_end() {
    let res = run_code("(let ((x '(3 4))) `(1 2 ,@x))").await.unwrap();
    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ])
    );
}

#[tokio::test]
async fn test_backquote_splicing_only() {
    let res = run_code("(let ((x '(1 2 3))) `(,@x))").await.unwrap();
    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
}

#[tokio::test]
async fn test_backquote_multiple_splicing() {
    let res = run_code("(let ((x '(1 2)) (y '(3 4))) `(,@x ,@y))")
        .await
        .unwrap();
    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ])
    );
}

// --- Nested backquote ---

#[tokio::test]
async fn test_backquote_nested() {
    let res = run_code("(let ((x 2)) `(1 `(a ,,x)))").await.unwrap();
    // Nested backquote should preserve inner backquote form but resolve outer unquote
    if let Value::List(l) = res {
        assert_eq!(l.len(), 2);
        assert_eq!(l[0], Value::Integer(1));
        // The inner form should still have a backquote wrapper
        if let Value::List(inner) = &l[1] {
            assert_eq!(inner[0], Value::Symbol("backquote".to_string()));
        } else {
            panic!("Expected list for nested backquote");
        }
    } else {
        panic!("Expected list");
    }
}

// --- Vector ---

#[tokio::test]
async fn test_backquote_vector() {
    let res = run_code("(let ((x 3)) `[1 2 ,x])").await.unwrap();
    assert_eq!(
        res,
        Value::Vector(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
}

// --- Backquote in macro definition ---

#[tokio::test]
async fn test_backquote_in_macro() {
    let code = r#"
        (defmacro when (cond body)
          `(if ,cond ,body nil))
        (when true 42)
    "#;
    assert_eq!(run_code(code).await.unwrap(), Value::Integer(42));
}

#[tokio::test]
async fn test_backquote_in_macro_false_branch() {
    let code = r#"
        (defmacro when (cond body)
          `(if ,cond ,body nil))
        (when false 42)
    "#;
    assert_eq!(run_code(code).await.unwrap(), Value::Nil);
}

#[tokio::test]
async fn test_backquote_macro_with_splicing() {
    let code = r#"
        (defmacro my-list (items)
          `(list ,@items))
        (my-list (1 2 3))
    "#;
    assert_eq!(
        run_code(code).await.unwrap(),
        Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
}
