use dlisp_core::ast::Value;
use dlisp_core::interpreter::{Interpreter, default_env};
use dlisp_core::parser::parse;

/// The fixture has one named function and no user-defined callees. Removing
/// its compiled pointer therefore selects a pure interpreter execution.
async fn probe(body: &str, compiled: bool) -> Value {
    let mut interpreter = Interpreter::new();
    let mut env = default_env();
    for form in parse(&format!("(defun probe () {body})")).unwrap() {
        interpreter.eval(form, &mut env).await.unwrap();
    }
    let mut function = env.borrow().get("probe").unwrap();
    match &mut function {
        Value::UserFunc { jit_code, .. } => {
            assert!(jit_code.is_some(), "fixture must actually compile: {body}");
            if !compiled {
                *jit_code = None;
            }
        }
        other => panic!("expected function, got {other:?}"),
    }
    env.borrow_mut().set("probe".into(), function);
    interpreter
        .eval(parse("(probe)").unwrap().remove(0), &mut env)
        .await
        .unwrap()
}

#[tokio::test]
async fn io_builtins_really_jit_and_match_pure_interpreter() {
    let path = format!("{}/../Cargo.toml", env!("CARGO_MANIFEST_DIR"));
    let cases = [
        (
            format!("(string-contains? (read-file \"{path}\") \"workspace\")"),
            Value::Bool(true),
        ),
        (format!("(file-exists? \"{path}\")"), Value::Bool(true)),
        (format!("(is-file? \"{path}\")"), Value::Bool(true)),
        (format!("(is-dir? \"{path}\")"), Value::Bool(false)),
        (
            format!("(try (list-dir \"{path}\") (catch e (error? e)))"),
            Value::Bool(true),
        ),
        (
            "(try (delete-file 1) (catch e (error? e)))".into(),
            Value::Bool(true),
        ),
        (
            "(get (sh \"printf ok\") :stdout)".into(),
            Value::String("ok".into()),
        ),
        (
            "(try (read-file 1) (catch e (error? e)))".into(),
            Value::Bool(true),
        ),
        (
            "(try (sh 1) (catch e (error? e)))".into(),
            Value::Bool(true),
        ),
    ];
    for (body, expected) in cases {
        for compiled in [false, true] {
            assert_eq!(
                probe(&body, compiled).await,
                expected,
                "compiled={compiled}: {body}"
            );
        }
    }
}

#[tokio::test]
async fn arithmetic_errors_really_jit_and_match_pure_interpreter() {
    let cases = [
        ("(+ 1 \"x\")", "Arguments must be numbers"),
        ("(+ \"x\")", "Arguments must be numbers"),
        ("(* \"x\")", "Arguments must be numbers"),
        ("(- \"x\")", "Arguments must be numbers"),
        ("(/ 1 0.0)", "Division by zero"),
        ("(% 1.0 -0.0)", "Division by zero"),
        ("(+ 9223372036854775807 1)", "Integer overflow"),
        ("(- -9223372036854775808 1)", "Integer overflow"),
        ("(* 9223372036854775807 2)", "Integer overflow"),
        ("(- -9223372036854775808)", "Integer overflow"),
        ("(/ -9223372036854775808 -1)", "Integer overflow"),
        ("(% -9223372036854775808 -1)", "Integer overflow"),
    ];
    for (expression, message) in cases {
        let body = format!("(try {expression} (catch e (error-value e)))");
        for compiled in [false, true] {
            assert_eq!(
                probe(&body, compiled).await,
                Value::String(message.into()),
                "compiled={compiled}: {body}"
            );
        }
    }
}

#[tokio::test]
async fn runtime_errors_escape_nested_callbacks_and_clear_pending_state() {
    for body in [
        "(try (try (/ 1 0)) (catch e (error-value e)))",
        "(try (let ((f (lambda () (/ 1 0)))) (f)) (catch e (error-value e)))",
        "(try (map (lambda (x) (/ 1 x)) (list 0 1)) (catch e (error-value e)))",
        "(try (filter (lambda (x) (/ 1 x)) (list 0 1)) (catch e (error-value e)))",
        "(try (reduce (lambda (acc x) (/ 1 x)) 0 (list 0 1)) (catch e (error-value e)))",
        "(try (some (lambda (x) (/ 1 x)) (list 0 1)) (catch e (error-value e)))",
        "(try (every (lambda (x) (/ 1 x)) (list 0 1)) (catch e (error-value e)))",
        "(try (find (lambda (x) (/ 1 x)) (list 0 1)) (catch e (error-value e)))",
        "(try (for-each (lambda (x) (/ 1 x)) (list 0 1)) (catch e (error-value e)))",
    ] {
        for compiled in [false, true] {
            assert_eq!(
                probe(body, compiled).await,
                Value::String("Division by zero".into())
            );
        }
    }
    let body = r#"(try (/ 1 0) (catch e nil))
                  (+ 2 3)
                  (try (throw "fresh") (catch e (error-value e)))"#;
    for compiled in [false, true] {
        assert_eq!(probe(body, compiled).await, Value::String("fresh".into()));
    }
}

#[tokio::test]
async fn numeric_results_and_boundaries_match_pure_interpreter() {
    for (body, expected) in [
        ("(+)", Value::Integer(0)),
        ("(*)", Value::Integer(1)),
        ("(+ 7)", Value::Integer(7)),
        ("(* 8)", Value::Integer(8)),
        ("(- 9)", Value::Integer(-9)),
        ("(/ 100 2 5)", Value::Integer(10)),
        ("(+ 1 2.5)", Value::Float(3.5)),
        ("(% 5.5 2)", Value::Float(1.5)),
        ("(+ 9223372036854775806 1)", Value::Integer(i64::MAX)),
        ("(- -9223372036854775807 1)", Value::Integer(i64::MIN)),
    ] {
        for compiled in [false, true] {
            assert_eq!(
                probe(body, compiled).await,
                expected,
                "compiled={compiled}: {body}"
            );
        }
    }
}

#[tokio::test]
async fn addition_uses_the_same_zero_identity_on_each_path() {
    for expression in ["(+ -0.0)", "(+ -0.0 -0.0)"] {
        for compiled in [false, true] {
            let Value::Float(value) = probe(expression, compiled).await else {
                panic!("expected float: {expression}");
            };
            assert_eq!(
                value.to_bits(),
                0.0_f64.to_bits(),
                "compiled={compiled}: {expression}"
            );
        }
    }
}
