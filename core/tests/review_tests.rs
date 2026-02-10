use dlisp_core::ast::Value;
use dlisp_core::interpreter::default_interpreter;
use dlisp_core::parser::parse;
use std::cell::RefCell;
use std::rc::Rc;

fn setup() -> (
    dlisp_core::interpreter::Interpreter,
    Rc<RefCell<dlisp_core::environment::Environment>>,
) {
    let interpreter = default_interpreter();
    let env = dlisp_core::interpreter::default_env();
    (interpreter, env)
}

async fn eval_str(
    src: &str,
    interpreter: &mut dlisp_core::interpreter::Interpreter,
    env: &mut Rc<RefCell<dlisp_core::environment::Environment>>,
) -> Value {
    let exprs = parse(src).unwrap();
    let mut result = Value::Nil;
    for expr in exprs {
        result = interpreter.eval(expr, env).await.unwrap();
    }
    result
}

// --- Parser Boundary Tests ---

#[test]
fn test_parser_boundaries() {
    // Keywords vs Symbols
    assert_eq!(parse("nil").unwrap()[0], Value::Nil);
    assert_eq!(parse("nil?").unwrap()[0], Value::Symbol("nil?".to_string()));
    assert_eq!(
        parse("nil-value").unwrap()[0],
        Value::Symbol("nil-value".to_string())
    );

    assert_eq!(parse("true").unwrap()[0], Value::Bool(true));
    assert_eq!(
        parse("true?").unwrap()[0],
        Value::Symbol("true?".to_string())
    );
    assert_eq!(
        parse("true-value").unwrap()[0],
        Value::Symbol("true-value".to_string())
    );

    assert_eq!(parse("false").unwrap()[0], Value::Bool(false));
    assert_eq!(
        parse("false?").unwrap()[0],
        Value::Symbol("false?".to_string())
    );
    assert_eq!(
        parse("false-val").unwrap()[0],
        Value::Symbol("false-val".to_string())
    );
}

// --- Unicode String Tests ---

#[tokio::test]
async fn test_unicode_strings() {
    let (mut i, mut e) = setup();

    // Length (chars, not bytes)
    // "あ" is 3 bytes, 1 char
    assert_eq!(
        eval_str(r#"(string-length "あいう")"#, &mut i, &mut e).await,
        Value::Integer(3)
    );

    // Substring (char indices)
    assert_eq!(
        eval_str(r#"(substring "aあbい" 1 3)"#, &mut i, &mut e).await,
        Value::String("あb".to_string())
    );

    // Concat
    assert_eq!(
        eval_str(r#"(str "Hola " "世界")"#, &mut i, &mut e).await,
        Value::String("Hola 世界".to_string())
    );
}

// --- Arithmetic Edge Cases ---

#[tokio::test]
async fn test_arithmetic_edges() {
    let (mut i, mut e) = setup();

    // Integer division truncation
    assert_eq!(eval_str("(/ 5 2)", &mut i, &mut e).await, Value::Integer(2));

    // Float promotion
    assert_eq!(
        eval_str("(/ 5 2.0)", &mut i, &mut e).await,
        Value::Float(2.5)
    );

    // Negative modulo
    // Rust % operator behavior: -5 % 2 = -1
    // Some Lisps use different modulo logic, but consistency with host (Rust) is usually fine.
    assert_eq!(
        eval_str("(% -5 2)", &mut i, &mut e).await,
        Value::Integer(-1)
    );

    // Negative float
    assert_eq!(
        eval_str("(/ -5.0 2.0)", &mut i, &mut e).await,
        Value::Float(-2.5)
    );
}

// --- Logic Short-circuiting ---

#[tokio::test]
async fn test_logic_short_circuit_with_side_effects() {
    let (mut i, mut e) = setup();

    // (and nil (setq x 1)) -> should not set x
    eval_str("(defvar x 0)", &mut i, &mut e).await;
    eval_str("(and nil (setq x 1))", &mut i, &mut e).await;
    assert_eq!(eval_str("x", &mut i, &mut e).await, Value::Integer(0));

    // (or true (setq y 1)) -> should not set y
    eval_str("(defvar y 0)", &mut i, &mut e).await;
    eval_str("(or 1 (setq y 1))", &mut i, &mut e).await;
    assert_eq!(eval_str("y", &mut i, &mut e).await, Value::Integer(0));
}
