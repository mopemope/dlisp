use dlisp_core::ast::Value;
use dlisp_core::interpreter::{Interpreter, default_env};
use dlisp_core::parser::parse;

/// Evaluates `code` through the default interpreter, which JIT-compiles
/// eligible `defun`s. These tests pin interpreter/JIT parity for compiled
/// builtins whose runtime FFI previously diverged from the evaluator.
async fn eval_code(code: &str) -> Value {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();
    let vals = parse(code).unwrap();
    let mut last_val = Value::Nil;
    for val in vals {
        last_val = interpreter.eval(val, &mut env).await.unwrap();
    }
    last_val
}

#[tokio::test]
async fn test_jit_eq_deep_collection_equality() {
    // Deep equality on lists and vectors inside a JIT-compiled function
    assert_eq!(
        eval_code(
            "(defun f () (= (list 1 2) (list 1 2)))
             (f)"
        )
        .await,
        Value::Bool(true)
    );

    assert_eq!(
        eval_code(
            "(defun f () (= (vector 1 2) (vector 1 2)))
             (f)"
        )
        .await,
        Value::Bool(true)
    );

    assert_eq!(
        eval_code(
            "(defun f () (= (list 1 (list 2 3)) (list 1 (list 2 3))))
             (f)"
        )
        .await,
        Value::Bool(true)
    );

    assert_eq!(
        eval_code(
            "(defun f () (= (list 1 2) (list 1 3)))
             (f)"
        )
        .await,
        Value::Bool(false)
    );
}

#[tokio::test]
async fn test_jit_nth_supports_lists() {
    // nth over a list inside a JIT-compiled function must not abort
    assert_eq!(
        eval_code(
            "(defun f (xs) (nth xs 1))
             (f (list 7 8 9))"
        )
        .await,
        Value::Integer(8)
    );

    // Out-of-range yields nil in compiled code
    assert_eq!(
        eval_code(
            "(defun f (xs) (nth xs 10))
             (f (vector 1 2 3))"
        )
        .await,
        Value::Nil
    );
}

#[tokio::test]
async fn test_jit_higher_order_supports_vectors() {
    // map preserves vector shape
    assert_eq!(
        eval_code(
            "(defun f (xs) (map (lambda (x) (+ x 1)) xs))
             (f (vector 1 2 3))"
        )
        .await,
        Value::Vector(vec![
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4)
        ])
    );

    // filter treats Int 0 as falsy, matching interpreter truthiness
    assert_eq!(
        eval_code(
            "(defun f (xs) (filter (lambda (x) x) xs))
             (f (vector 0 1 2))"
        )
        .await,
        Value::Vector(vec![Value::Integer(1), Value::Integer(2)])
    );

    // reduce folds over vectors
    assert_eq!(
        eval_code(
            "(defun f (xs) (reduce (lambda (a b) (+ a b)) 0 xs))
             (f (vector 1 2 3))"
        )
        .await,
        Value::Integer(6)
    );
}

#[tokio::test]
async fn test_jit_vector_literal_construction() {
    // Vector literals lowered by codegen must build correctly at runtime
    assert_eq!(
        eval_code(
            "(defun f () (vector 1 2 3))
             (f)"
        )
        .await,
        Value::Vector(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3)
        ])
    );
}
