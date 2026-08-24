//! Behavior tests for the Clojure-flavoured collection helpers in the
//! bundled stdlib (`stdlib/src/core.lisp`), loaded via `(require "core")`.
//!
//! Compiled-path parity for these functions is pinned by
//! `example-lisp/stdlib_collections.lisp` in `cli/tests/parity_golden_tests.rs`.

use dlisp_core::ast::Value;
use dlisp_core::interpreter::{default_env, default_interpreter};
use dlisp_core::parser::parse;
use std::cell::RefCell;
use std::rc::Rc;

fn setup() -> (
    dlisp_core::interpreter::Interpreter,
    Rc<RefCell<dlisp_core::environment::Environment>>,
) {
    (default_interpreter(), default_env())
}

async fn eval_str(
    src: &str,
    interpreter: &mut dlisp_core::interpreter::Interpreter,
    env: &mut Rc<RefCell<dlisp_core::environment::Environment>>,
) -> Value {
    // Load the bundled stdlib first; `require` caches, so repeats are free.
    let pre = parse("(require \"core\")").unwrap();
    for expr in pre {
        interpreter.eval(expr, env).await.unwrap();
    }
    let exprs = parse(src).unwrap();
    let mut result = Value::Nil;
    for expr in exprs {
        result = interpreter.eval(expr, env).await.unwrap();
    }
    result
}

fn list_of(nums: &[i64]) -> Value {
    Value::List(nums.iter().map(|n| Value::Integer(*n)).collect())
}

#[tokio::test]
async fn test_member_and_distinct() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(member? 2 '(1 2 3))", &mut i, &mut e).await,
        Value::Bool(true)
    );
    // `some` yields nil when no element matches.
    assert_eq!(
        eval_str("(member? 9 '(1 2 3))", &mut i, &mut e).await,
        Value::Nil
    );
    assert_eq!(
        eval_str("(distinct '(1 2 1 3 2))", &mut i, &mut e).await,
        list_of(&[1, 2, 3])
    );
}

#[tokio::test]
async fn test_frequencies() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(get (frequencies '(a b a c a)) 'a)", &mut i, &mut e).await,
        Value::Integer(3)
    );
    assert_eq!(
        eval_str("(get (frequencies '(x y)) 'z)", &mut i, &mut e).await,
        Value::Nil
    );
}

#[tokio::test]
async fn test_group_by() {
    let (mut i, mut e) = setup();
    // Even/odd partition sizes avoid printing map key order.
    assert_eq!(
        eval_str(
            "(map count (list (get (group-by (lambda (x) (% x 2)) '(1 2 3 4 5 6)) 0) \
                              (get (group-by (lambda (x) (% x 2)) '(1 2 3 4 5 6)) 1)))",
            &mut i,
            &mut e
        )
        .await,
        list_of(&[3, 3])
    );
}

#[tokio::test]
async fn test_merge_with() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str(
            "(get (merge-with (lambda (x y) (+ x y)) {:a 1 :b 2} {:b 10 :c 3}) :b)",
            &mut i,
            &mut e
        )
        .await,
        Value::Integer(12)
    );
    assert_eq!(
        eval_str(
            "(get (merge-with (lambda (x y) (+ x y)) {:a 1} {:c 3}) :c)",
            &mut i,
            &mut e
        )
        .await,
        Value::Integer(3)
    );
}

#[tokio::test]
async fn test_get_in_assoc_in_update_in() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(get-in {:a {:b 7}} [:a :b])", &mut i, &mut e).await,
        Value::Integer(7)
    );
    assert_eq!(
        eval_str("(get-in {} [:a :b])", &mut i, &mut e).await,
        Value::Nil
    );
    assert_eq!(
        eval_str("(get-in (assoc-in {} [:a :b] 5) [:a :b])", &mut i, &mut e).await,
        Value::Integer(5)
    );
    // update-in walks through an explicitly created nested map.
    assert_eq!(
        eval_str(
            "(get-in (update-in {:counter 41} [:counter] inc) [:counter])",
            &mut i,
            &mut e
        )
        .await,
        Value::Integer(42)
    );
}

#[tokio::test]
async fn test_partition() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(partition 2 '(1 2 3 4 5))", &mut i, &mut e).await,
        Value::List(vec![list_of(&[1, 2]), list_of(&[3, 4]), list_of(&[5])])
    );
    assert_eq!(
        eval_str("(partition 3 '())", &mut i, &mut e).await,
        Value::Nil
    );
}

#[tokio::test]
async fn test_interleave() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(interleave '(1 2 3) '(a b c))", &mut i, &mut e).await,
        Value::List(vec![
            Value::Integer(1),
            Value::Symbol("a".to_string()),
            Value::Integer(2),
            Value::Symbol("b".to_string()),
            Value::Integer(3),
            Value::Symbol("c".to_string()),
        ])
    );
    // Stops at the shorter collection.
    assert_eq!(
        eval_str("(interleave '(1 2) '(x))", &mut i, &mut e).await,
        Value::List(vec![Value::Integer(1), Value::Symbol("x".to_string())])
    );
}

#[tokio::test]
async fn test_match_literals_and_bindings() {
    let (mut i, mut e) = setup();
    // Literal patterns use structural `=` equality.
    assert_eq!(
        eval_str("(match 42 (42 :answer) (_ :other))", &mut i, &mut e).await,
        Value::Keyword("answer".to_string())
    );
    assert_eq!(
        eval_str(
            r#"(match "hi" ("hi" :greeting) (_ :other))"#,
            &mut i,
            &mut e
        )
        .await,
        Value::Keyword("greeting".to_string())
    );
    assert_eq!(
        eval_str("(match :ok (:ok :fine) (_ :other))", &mut i, &mut e).await,
        Value::Keyword("fine".to_string())
    );
    // A plain symbol pattern binds the scrutinee; `_` matches without binding.
    assert_eq!(
        eval_str("(match 7 (n (* n n)) (_ nil))", &mut i, &mut e).await,
        Value::Integer(49)
    );
    // No matching clause yields nil.
    assert_eq!(
        eval_str("(match :nope (1 :one) ([a b] :pair))", &mut i, &mut e).await,
        Value::Nil
    );
}

#[tokio::test]
async fn test_match_guards_fall_through() {
    let (mut i, mut e) = setup();
    // A failed guard must fall through to later clauses.
    assert_eq!(
        eval_str(
            "(match 8 ((x :when (> x 10)) :big) ((x :when (> x 5)) :medium) (x :small))",
            &mut i,
            &mut e
        )
        .await,
        Value::Keyword("medium".to_string())
    );
    assert_eq!(
        eval_str(
            "(match 2 ((x :when (> x 10)) :big) ((x :when (> x 5)) :medium) (x :small))",
            &mut i,
            &mut e
        )
        .await,
        Value::Keyword("small".to_string())
    );
}

#[tokio::test]
async fn test_match_sequence_patterns() {
    let (mut i, mut e) = setup();
    // Fixed-length sequence over lists and vectors.
    assert_eq!(
        eval_str("(match [1 2] ([a b] (+ a b)))", &mut i, &mut e).await,
        Value::Integer(3)
    );
    assert_eq!(
        eval_str("(match '(1 2) ([a b] (+ a b)))", &mut i, &mut e).await,
        Value::Integer(3)
    );
    // Wrong length must not match.
    assert_eq!(
        eval_str(
            "(match '(1 2 3) ([a b] :two) ([a b c] :three))",
            &mut i,
            &mut e
        )
        .await,
        Value::Keyword("three".to_string())
    );
    // `&rest` captures the tail.
    assert_eq!(
        eval_str("(match [1 2 3 4] ([a b &rest r] r))", &mut i, &mut e).await,
        Value::Vector(vec![Value::Integer(3), Value::Integer(4)])
    );
}

#[tokio::test]
async fn test_match_map_and_nested_patterns() {
    let (mut i, mut e) = setup();
    // Every key in the map pattern must be present.
    assert_eq!(
        eval_str("(match {:x 2 :y 3} ({:x x :y y} (+ x y)))", &mut i, &mut e).await,
        Value::Integer(5)
    );
    // A missing key means no match.
    assert_eq!(
        eval_str(
            "(match {:x 1} ({:x x :y y} :both) (_ :fallback))",
            &mut i,
            &mut e
        )
        .await,
        Value::Keyword("fallback".to_string())
    );
    // Nested composition: sequence of map and guard patterns.
    assert_eq!(
        eval_str(
            "(match [{:kind :add} 5] ([{:kind k} v] (list k v)))",
            &mut i,
            &mut e
        )
        .await,
        Value::List(vec![Value::Keyword("add".to_string()), Value::Integer(5),])
    );
    // The scrutinee is evaluated exactly once (side effects happen once).
    assert_eq!(
        eval_str(
            "(defvar calls 0) (defvar once (lambda () (setq calls (+ calls 1)) 9)) \
             (match (once) (9 :nine)) calls",
            &mut i,
            &mut e
        )
        .await,
        Value::Integer(1)
    );
}
