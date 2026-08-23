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

async fn eval_str(src: &str) -> Value {
    let (mut interpreter, mut env) = setup();
    let exprs = parse(src).unwrap();
    interpreter.eval(exprs[0].clone(), &mut env).await.unwrap()
}

// --- Keyword parsing ---

#[tokio::test]
async fn test_keyword_parse_standalone() {
    let vals = parse(":foo").unwrap();
    assert_eq!(vals[0], Value::Keyword("foo".to_string()));
}

#[tokio::test]
async fn test_keyword_parse_in_list() {
    let val = eval_str("(list :a :b :c)").await;
    assert_eq!(
        val,
        Value::List(vec![
            Value::Keyword("a".to_string()),
            Value::Keyword("b".to_string()),
            Value::Keyword("c".to_string()),
        ])
    );
}

// --- Keyword self-evaluation ---

#[tokio::test]
async fn test_keyword_self_eval() {
    assert_eq!(
        eval_str(":hello").await,
        Value::Keyword("hello".to_string())
    );
}

// --- keyword? predicate ---

#[tokio::test]
async fn test_keyword_pred_true() {
    assert_eq!(eval_str("(keyword? :foo)").await, Value::Bool(true));
}

#[tokio::test]
async fn test_keyword_pred_false_symbol() {
    assert_eq!(eval_str("(keyword? 'foo)").await, Value::Bool(false));
}

#[tokio::test]
async fn test_keyword_pred_false_string() {
    assert_eq!(eval_str("(keyword? \"hello\")").await, Value::Bool(false));
}

// --- symbol? should NOT match keywords ---

#[tokio::test]
async fn test_symbol_pred_false_for_keyword() {
    assert_eq!(eval_str("(symbol? :foo)").await, Value::Bool(false));
}

// --- type-of ---

#[tokio::test]
async fn test_type_of_keyword() {
    assert_eq!(
        eval_str("(type-of :foo)").await,
        Value::String("keyword".to_string())
    );
}

// --- Keyword equality ---

#[tokio::test]
async fn test_keyword_equality() {
    assert_eq!(eval_str("(= :a :a)").await, Value::Bool(true));
    assert_eq!(eval_str("(= :a :b)").await, Value::Bool(false));
}

// --- Keyword as map key ---

#[tokio::test]
async fn test_keyword_as_map_key() {
    assert_eq!(
        eval_str("(get {:name \"Alice\"} :name)").await,
        Value::String("Alice".to_string())
    );
}

// --- str with keyword ---

#[tokio::test]
async fn test_str_with_keyword() {
    assert_eq!(
        eval_str("(str :hello)").await,
        Value::String(":hello".to_string())
    );
}
