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

#[tokio::test]
async fn test_jit_while_loop() {
    // Accumulation loop with setq inside a JIT-compiled function
    assert_eq!(
        eval_code(
            "(defun f (n)
               (let ((acc 0) (i 0))
                 (while (< i n)
                   (setq acc (+ acc i))
                   (setq i (+ i 1)))
                 acc))
             (f 5)"
        )
        .await,
        Value::Integer(10)
    );

    // while evaluates to the last body value of the final iteration
    assert_eq!(
        eval_code(
            "(defun f ()
               (let ((i 0))
                 (while (< i 3)
                   (setq i (+ i 1))
                   (* i 10))))
             (f)"
        )
        .await,
        Value::Integer(30)
    );

    // A false condition on entry yields nil without running the body
    assert_eq!(
        eval_code(
            "(defun f ()
               (while nil
                 42))
             (f)"
        )
        .await,
        Value::Nil
    );
}

#[tokio::test]
async fn test_jit_dotimes_loop() {
    assert_eq!(
        eval_code(
            "(defun f (n)
               (let ((acc 0))
                 (dotimes (k n)
                   (setq acc (+ acc k)))
                 acc))
             (f 5)"
        )
        .await,
        Value::Integer(10)
    );

    // Zero iterations bind nothing and return nil
    assert_eq!(
        eval_code(
            "(defun f ()
               (dotimes (k 0)
                 42))
             (f)"
        )
        .await,
        Value::Nil
    );
}

#[tokio::test]
async fn test_jit_dolist_loop() {
    // Lists and vectors iterate identically
    assert_eq!(
        eval_code(
            "(defun f (xs)
               (let ((acc 0))
                 (dolist (x xs)
                   (setq acc (+ acc x)))
                 acc))
             (f '(1 2 3))"
        )
        .await,
        Value::Integer(6)
    );

    assert_eq!(
        eval_code(
            "(defun f (xs)
               (let ((acc 0))
                 (dolist (x xs)
                   (setq acc (+ acc x)))
                 acc))
             (f (vector 4 5))"
        )
        .await,
        Value::Integer(9)
    );
}

#[tokio::test]
async fn test_jit_higher_order_predicates() {
    // some returns the first truthy result (arithmetic keeps the result
    // type stable across interpreter and compiled paths)
    assert_eq!(
        eval_code(
            "(defun f (xs) (some (lambda (x) (- x 2)) xs))
             (f '(5 6))"
        )
        .await,
        Value::Integer(3)
    );

    assert_eq!(
        eval_code(
            "(defun f (xs) (some (lambda (x) (- x 2)) xs))
             (f '(2 2))"
        )
        .await,
        Value::Nil
    );

    // every is vacuously true and short-circuits on falsy results
    assert_eq!(
        eval_code(
            "(defun f (xs) (every (lambda (x) (> x 0)) xs))
             (f '(1 2 3))"
        )
        .await,
        Value::Bool(true)
    );

    assert_eq!(
        eval_code(
            "(defun f (xs) (every (lambda (x) (> x 0)) xs))
             (f '(1 -2 3))"
        )
        .await,
        Value::Bool(false)
    );

    // find returns the element, not the predicate result
    assert_eq!(
        eval_code(
            "(defun f (xs) (find (lambda (x) (> x 2)) xs))
             (f '(1 2 3 4))"
        )
        .await,
        Value::Integer(3)
    );

    // for-each returns nil
    assert_eq!(
        eval_code(
            "(defun f (xs) (for-each (lambda (x) x) xs))
             (f '(1 2 3))"
        )
        .await,
        Value::Nil
    );
}

#[tokio::test]
async fn test_jit_gate_accepts_let_loops_and_higher_order_forms() {
    // Regression: the JIT gate used to reject functions containing `let`
    // bindings, `lambda` parameters, or env-unbound compiled builtins
    // (map/filter/some/...), silently falling back to the interpreter.
    let mut env = default_env();
    let mut interpreter = Interpreter::new();
    let code = r#"
        (defun sum-with-let (xs)
          (let ((acc 0))
            (dolist (x xs)
              (setq acc (+ acc x)))
            acc))
        (defun map-inc (xs)
          (map (lambda (x) (+ x 1)) xs))
        (sum-with-let '(1 2 3))
        (map-inc '(1 2))
    "#;
    for val in parse(code).unwrap() {
        interpreter.eval(val, &mut env).await.unwrap();
    }

    for name in ["sum-with-let", "map-inc"] {
        match env.borrow().get(name) {
            Some(Value::UserFunc { jit_code, .. }) => {
                assert!(jit_code.is_some(), "{} should be JIT-compiled", name);
            }
            other => panic!("{} should be a user function, got {:?}", name, other),
        }
    }
}
