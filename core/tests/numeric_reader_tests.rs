use dlisp_core::ast::Value;
use dlisp_core::interpreter::{Interpreter, default_env};
use dlisp_core::parser::parse;

#[tokio::test]
async fn decimal_and_unicode_reader_fixes_reach_real_jit() {
    for (body, expected) in [
        ("0.001", Value::Float(0.001)),
        (
            "9223372036854775808.5",
            Value::Float(9_223_372_036_854_775_808.0),
        ),
        (
            "0.12345678901234567890123456789",
            Value::Float(0.123_456_789_012_345_68),
        ),
        (
            "(let ((nilあ 7) (trueλ 8) (false猫 9)) (+ nilあ trueλ false猫))",
            Value::Integer(24),
        ),
    ] {
        for compiled in [false, true] {
            let mut interpreter = Interpreter::new();
            let mut env = default_env();
            for form in parse(&format!("(defun probe () {body})")).unwrap() {
                interpreter.eval(form, &mut env).await.unwrap();
            }
            let mut function = env.borrow().get("probe").unwrap();
            match &mut function {
                Value::UserFunc { jit_code, .. } => {
                    assert!(jit_code.is_some(), "must compile: {body}");
                    if !compiled {
                        *jit_code = None;
                    }
                }
                _ => panic!("expected function"),
            }
            env.borrow_mut().set("probe".into(), function);
            let result = interpreter
                .eval(parse("(probe)").unwrap().remove(0), &mut env)
                .await
                .unwrap();
            assert_eq!(result, expected, "compiled={compiled}: {body}");
        }
    }
}

#[tokio::test]
async fn numeric_helpers_keep_explicit_interpreter_fallback() {
    for (body, expected) in [
        (
            "(try (abs -9223372036854775808) (catch e (error-value e)))",
            Value::String("Integer overflow".into()),
        ),
        ("(pow 2 -4294967296)", Value::Float(0.0)),
        ("(pow -1.0 9223372036854775807)", Value::Float(-1.0)),
    ] {
        let mut interpreter = Interpreter::new();
        let mut env = default_env();
        for form in parse(&format!("(defun probe () {body})")).unwrap() {
            interpreter.eval(form, &mut env).await.unwrap();
        }
        assert!(matches!(
            env.borrow().get("probe"),
            Some(Value::UserFunc { jit_code: None, .. })
        ));
        let result = interpreter
            .eval(parse("(probe)").unwrap().remove(0), &mut env)
            .await
            .unwrap();
        assert_eq!(result, expected);
    }
}

#[tokio::test]
async fn jit_gate_keeps_parameters_and_cond_clauses_compilable() {
    for (definition, call, expected) in [
        (
            "(defun probe (f x) (let ((y x)) (f y)))",
            "(probe (lambda (x) (+ x 1)) 7)",
            Value::Integer(8),
        ),
        (
            "(defun probe (flag) (let ((y 7)) (cond (flag y) (true 0))))",
            "(probe true)",
            Value::Integer(7),
        ),
        (
            "(defun probe (x &rest xs) (let ((f (car xs))) (f x)))",
            "(probe 7 (lambda (x) (+ x 1)))",
            Value::Integer(8),
        ),
    ] {
        let mut interpreter = Interpreter::new();
        let mut env = default_env();
        interpreter
            .eval(parse(definition).unwrap().remove(0), &mut env)
            .await
            .unwrap();
        assert!(
            matches!(
                env.borrow().get("probe"),
                Some(Value::UserFunc {
                    jit_code: Some(_),
                    ..
                })
            ),
            "{definition}"
        );
        // Build closure arguments inside compiled code: passing an interpreter
        // UserFunc across run_jit_function would silently fall back at runtime.
        interpreter
            .eval(
                parse(&format!("(defun entry () {call})"))
                    .unwrap()
                    .remove(0),
                &mut env,
            )
            .await
            .unwrap();
        assert!(matches!(
            env.borrow().get("entry"),
            Some(Value::UserFunc {
                jit_code: Some(_),
                ..
            })
        ));
        assert_eq!(
            interpreter
                .eval(parse("(entry)").unwrap().remove(0), &mut env)
                .await
                .unwrap(),
            expected,
            "{call}"
        );
    }
}

#[tokio::test]
async fn jit_retry_keeps_parameter_calls_after_forward_reference_resolves() {
    let mut interpreter = Interpreter::new();
    let mut env = default_env();
    for definition in [
        "(defun probe (f x) (let ((y (later x))) (f y)))",
        "(defun later (x) (+ x 1))",
    ] {
        interpreter
            .eval(parse(definition).unwrap().remove(0), &mut env)
            .await
            .unwrap();
    }
    assert!(matches!(
        env.borrow().get("probe"),
        Some(Value::UserFunc {
            jit_code: Some(_),
            ..
        })
    ));
    interpreter
        .eval(
            parse("(defun entry () (probe (lambda (x) (+ x 1)) 7))")
                .unwrap()
                .remove(0),
            &mut env,
        )
        .await
        .unwrap();
    assert!(matches!(
        env.borrow().get("entry"),
        Some(Value::UserFunc {
            jit_code: Some(_),
            ..
        })
    ));
    assert_eq!(
        interpreter
            .eval(parse("(entry)").unwrap().remove(0), &mut env)
            .await
            .unwrap(),
        Value::Integer(9)
    );
}
