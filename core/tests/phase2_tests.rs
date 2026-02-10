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

async fn eval_err(
    src: &str,
    interpreter: &mut dlisp_core::interpreter::Interpreter,
    env: &mut Rc<RefCell<dlisp_core::environment::Environment>>,
) -> String {
    let exprs = parse(src).unwrap();
    interpreter.eval(exprs[0].clone(), env).await.unwrap_err()
}

// --- Division ---

#[tokio::test]
async fn test_div_integers() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(/ 10 3)", &mut i, &mut e).await,
        Value::Integer(3)
    );
}

#[tokio::test]
async fn test_div_float() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(/ 10.0 4.0)", &mut i, &mut e).await,
        Value::Float(2.5)
    );
}

#[tokio::test]
async fn test_div_chain() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(/ 100 2 5)", &mut i, &mut e).await,
        Value::Integer(10)
    );
}

#[tokio::test]
async fn test_div_by_zero() {
    let (mut i, mut e) = setup();
    let err = eval_err("(/ 10 0)", &mut i, &mut e).await;
    assert!(err.contains("zero"));
}

// --- Modulo ---

#[tokio::test]
async fn test_mod_basic() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(% 10 3)", &mut i, &mut e).await,
        Value::Integer(1)
    );
}

#[tokio::test]
async fn test_mod_alias() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(mod 17 5)", &mut i, &mut e).await,
        Value::Integer(2)
    );
}

// --- Comparison >=, <=, /= ---

#[tokio::test]
async fn test_gte() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(>= 3 3)", &mut i, &mut e).await,
        Value::Bool(true)
    );
    assert_eq!(
        eval_str("(>= 4 3)", &mut i, &mut e).await,
        Value::Bool(true)
    );
    assert_eq!(
        eval_str("(>= 2 3)", &mut i, &mut e).await,
        Value::Bool(false)
    );
}

#[tokio::test]
async fn test_lte() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(<= 3 3)", &mut i, &mut e).await,
        Value::Bool(true)
    );
    assert_eq!(
        eval_str("(<= 2 3)", &mut i, &mut e).await,
        Value::Bool(true)
    );
    assert_eq!(
        eval_str("(<= 4 3)", &mut i, &mut e).await,
        Value::Bool(false)
    );
}

#[tokio::test]
async fn test_neq() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(/= 1 2)", &mut i, &mut e).await,
        Value::Bool(true)
    );
    assert_eq!(
        eval_str("(/= 1 1)", &mut i, &mut e).await,
        Value::Bool(false)
    );
}

// --- String operations ---

#[tokio::test]
async fn test_str() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str(r#"(str "hello " 42)"#, &mut i, &mut e).await,
        Value::String("hello 42".to_string())
    );
}

#[tokio::test]
async fn test_string_length() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str(r#"(string-length "hello")"#, &mut i, &mut e).await,
        Value::Integer(5)
    );
}

#[tokio::test]
async fn test_substring() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str(r#"(substring "hello" 1 3)"#, &mut i, &mut e).await,
        Value::String("el".to_string())
    );
}

#[tokio::test]
async fn test_string_append() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str(r#"(string-append "foo" "bar")"#, &mut i, &mut e).await,
        Value::String("foobar".to_string())
    );
}

// --- Type predicates ---

#[tokio::test]
async fn test_nil_pred() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(nil? nil)", &mut i, &mut e).await,
        Value::Bool(true)
    );
    assert_eq!(
        eval_str("(nil? 1)", &mut i, &mut e).await,
        Value::Bool(false)
    );
}

#[tokio::test]
async fn test_list_pred() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(list? '(1 2))", &mut i, &mut e).await,
        Value::Bool(true)
    );
    assert_eq!(
        eval_str("(list? nil)", &mut i, &mut e).await,
        Value::Bool(true)
    );
    assert_eq!(
        eval_str("(list? 42)", &mut i, &mut e).await,
        Value::Bool(false)
    );
}

#[tokio::test]
async fn test_number_pred() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(number? 42)", &mut i, &mut e).await,
        Value::Bool(true)
    );
    assert_eq!(
        eval_str("(number? 3.14)", &mut i, &mut e).await,
        Value::Bool(true)
    );
    assert_eq!(
        eval_str(r#"(number? "hi")"#, &mut i, &mut e).await,
        Value::Bool(false)
    );
}

#[tokio::test]
async fn test_string_pred() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str(r#"(string? "hello")"#, &mut i, &mut e).await,
        Value::Bool(true)
    );
    assert_eq!(
        eval_str("(string? 42)", &mut i, &mut e).await,
        Value::Bool(false)
    );
}

#[tokio::test]
async fn test_symbol_pred() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(symbol? 'foo)", &mut i, &mut e).await,
        Value::Bool(true)
    );
    assert_eq!(
        eval_str("(symbol? 42)", &mut i, &mut e).await,
        Value::Bool(false)
    );
}

#[tokio::test]
async fn test_vector_pred() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(vector? [1 2 3])", &mut i, &mut e).await,
        Value::Bool(true)
    );
    assert_eq!(
        eval_str("(vector? 42)", &mut i, &mut e).await,
        Value::Bool(false)
    );
}

#[tokio::test]
async fn test_map_pred() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(map? {1 2})", &mut i, &mut e).await,
        Value::Bool(true)
    );
    assert_eq!(
        eval_str("(map? 42)", &mut i, &mut e).await,
        Value::Bool(false)
    );
}

#[tokio::test]
async fn test_type_of() {
    let (mut i, mut e) = setup();
    assert_eq!(
        eval_str("(type-of 42)", &mut i, &mut e).await,
        Value::String("integer".to_string())
    );
    assert_eq!(
        eval_str("(type-of 3.14)", &mut i, &mut e).await,
        Value::String("float".to_string())
    );
    assert_eq!(
        eval_str(r#"(type-of "hi")"#, &mut i, &mut e).await,
        Value::String("string".to_string())
    );
    assert_eq!(
        eval_str("(type-of nil)", &mut i, &mut e).await,
        Value::String("nil".to_string())
    );
    assert_eq!(
        eval_str("(type-of '(1 2))", &mut i, &mut e).await,
        Value::String("list".to_string())
    );
    assert_eq!(
        eval_str("(type-of [1 2])", &mut i, &mut e).await,
        Value::String("vector".to_string())
    );
    assert_eq!(
        eval_str("(type-of {1 2})", &mut i, &mut e).await,
        Value::String("map".to_string())
    );
}

// --- Combined usage ---

#[tokio::test]
async fn test_fizzbuzz_logic() {
    let (mut i, mut e) = setup();
    eval_str(
        r#"(defun fizzbuzz (n)
             (cond
               ((= (% n 15) 0) "FizzBuzz")
               ((= (% n 3) 0)  "Fizz")
               ((= (% n 5) 0)  "Buzz")
               (1               (str n))))"#,
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(
        eval_str("(fizzbuzz 15)", &mut i, &mut e).await,
        Value::String("FizzBuzz".to_string())
    );
    assert_eq!(
        eval_str("(fizzbuzz 9)", &mut i, &mut e).await,
        Value::String("Fizz".to_string())
    );
    assert_eq!(
        eval_str("(fizzbuzz 10)", &mut i, &mut e).await,
        Value::String("Buzz".to_string())
    );
    assert_eq!(
        eval_str("(fizzbuzz 7)", &mut i, &mut e).await,
        Value::String("7".to_string())
    );
}
