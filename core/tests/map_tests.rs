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

#[tokio::test]
async fn test_map_literal() {
    let val = eval_str("{:a 1 :b 2}").await;
    match val {
        Value::Map(m) => {
            assert_eq!(m.len(), 2);
            assert_eq!(
                m.get(&Value::Keyword("a".to_string())),
                Some(&Value::Integer(1))
            );
        }
        _ => panic!("Expected Map"),
    }
}

#[tokio::test]
async fn test_hash_map() {
    let val = eval_str("(hash-map :a 1 :b 2)").await;
    match val {
        Value::Map(m) => {
            assert_eq!(m.len(), 2);
            assert_eq!(
                m.get(&Value::Keyword("a".to_string())),
                Some(&Value::Integer(1))
            );
        }
        _ => panic!("Expected Map"),
    }
}

#[tokio::test]
async fn test_get() {
    assert_eq!(eval_str("(get {:a 1} :a)").await, Value::Integer(1));
    assert_eq!(eval_str("(get {:a 1} :b)").await, Value::Nil);
    assert_eq!(eval_str("(get {:a 1} :b 10)").await, Value::Integer(10));
    assert_eq!(eval_str("(get nil :a)").await, Value::Nil);
}

#[tokio::test]
async fn test_assoc() {
    let val = eval_str("(assoc {:a 1} :b 2)").await;
    match val {
        Value::Map(m) => {
            assert_eq!(m.len(), 2);
            assert_eq!(
                m.get(&Value::Keyword("a".to_string())),
                Some(&Value::Integer(1))
            );
            assert_eq!(
                m.get(&Value::Keyword("b".to_string())),
                Some(&Value::Integer(2))
            );
        }
        _ => panic!("Expected Map"),
    }

    let val2 = eval_str("(assoc nil :a 1)").await;
    match val2 {
        Value::Map(m) => {
            assert_eq!(m.len(), 1);
            assert_eq!(
                m.get(&Value::Keyword("a".to_string())),
                Some(&Value::Integer(1))
            );
        }
        _ => panic!("Expected Map"),
    }
}

#[tokio::test]
async fn test_dissoc() {
    let val = eval_str("(dissoc {:a 1 :b 2} :a)").await;
    match val {
        Value::Map(m) => {
            assert_eq!(m.len(), 1);
            assert_eq!(
                m.get(&Value::Keyword("b".to_string())),
                Some(&Value::Integer(2))
            );
            assert_eq!(m.get(&Value::Keyword("a".to_string())), None);
        }
        _ => panic!("Expected Map"),
    }
    assert_eq!(eval_str("(dissoc nil :a)").await, Value::Nil);
}

#[tokio::test]
async fn test_keys_vals() {
    let keys_val = eval_str("(keys {:a 1})").await;
    if let Value::List(l) = keys_val {
        assert_eq!(l.len(), 1);
        assert_eq!(l[0], Value::Keyword("a".to_string()));
    } else {
        panic!("Expected List");
    }

    let vals_val = eval_str("(vals {:a 1})").await;
    if let Value::List(l) = vals_val {
        assert_eq!(l.len(), 1);
        assert_eq!(l[0], Value::Integer(1));
    } else {
        panic!("Expected List");
    }
}

#[tokio::test]
async fn test_contains() {
    assert_eq!(eval_str("(contains? {:a 1} :a)").await, Value::Bool(true));
    assert_eq!(eval_str("(contains? {:a 1} :b)").await, Value::Bool(false));
    assert_eq!(eval_str("(contains? nil :a)").await, Value::Bool(false));
}

#[tokio::test]
async fn test_map_equality() {
    // Order shouldn't matter
    assert_eq!(
        eval_str("(= {:a 1 :b 2} {:b 2 :a 1})").await,
        Value::Integer(1)
    );
    assert_eq!(eval_str("(= {:a 1} {:a 2})").await, Value::Integer(0));

    // Recursion with floats
    assert_eq!(eval_str("(= {:a 1.0} {:a 1.0})").await, Value::Integer(1));
    assert_eq!(eval_str("(= {:a 1.0} {:a 1.1})").await, Value::Integer(0));
    // Nested collections
    assert_eq!(
        eval_str("(= {:a [1 2]} {:a [1 2]})").await,
        Value::Integer(1)
    );
}

#[tokio::test]
async fn test_get_vector() {
    assert_eq!(eval_str("(get [10 20 30] 1)").await, Value::Integer(20));
    assert_eq!(eval_str("(get [10 20] 5)").await, Value::Nil);
    assert_eq!(
        eval_str("(get [10 20] 5 :default)").await,
        Value::Keyword("default".to_string())
    );
}

#[tokio::test]
async fn test_nested_map() {
    let val = eval_str("(get {:a {:b 1}} :a)").await;
    match val {
        Value::Map(m) => {
            assert_eq!(m.len(), 1);
            assert_eq!(
                m.get(&Value::Keyword("b".to_string())),
                Some(&Value::Integer(1))
            );
        }
        _ => panic!("Expected Map"),
    }
}

// --- merge integration tests ---

#[tokio::test]
async fn test_merge_basic() {
    let val = eval_str("(merge {:a 1} {:b 2})").await;
    match val {
        Value::Map(m) => {
            assert_eq!(m.len(), 2);
            assert_eq!(
                m.get(&Value::Keyword("a".to_string())),
                Some(&Value::Integer(1))
            );
            assert_eq!(
                m.get(&Value::Keyword("b".to_string())),
                Some(&Value::Integer(2))
            );
        }
        _ => panic!("Expected Map"),
    }
}

#[tokio::test]
async fn test_merge_overwrite() {
    let val = eval_str("(merge {:a 1} {:a 99 :b 2})").await;
    match val {
        Value::Map(m) => {
            assert_eq!(
                m.get(&Value::Keyword("a".to_string())),
                Some(&Value::Integer(99))
            );
            assert_eq!(
                m.get(&Value::Keyword("b".to_string())),
                Some(&Value::Integer(2))
            );
        }
        _ => panic!("Expected Map"),
    }
}

#[tokio::test]
async fn test_merge_three_maps() {
    let val = eval_str("(merge {:a 1} {:b 2} {:c 3})").await;
    match val {
        Value::Map(m) => {
            assert_eq!(m.len(), 3);
        }
        _ => panic!("Expected Map"),
    }
}

#[tokio::test]
async fn test_merge_with_nil() {
    let val = eval_str("(merge {:a 1} nil)").await;
    match val {
        Value::Map(m) => {
            assert_eq!(m.len(), 1);
            assert_eq!(
                m.get(&Value::Keyword("a".to_string())),
                Some(&Value::Integer(1))
            );
        }
        _ => panic!("Expected Map"),
    }
}

#[tokio::test]
async fn test_merge_empty() {
    let val = eval_str("(merge)").await;
    match val {
        Value::Map(m) => assert_eq!(m.len(), 0),
        _ => panic!("Expected Map"),
    }
}

// --- select-keys integration tests ---

#[tokio::test]
async fn test_select_keys_basic() {
    let val = eval_str("(select-keys {:a 1 :b 2 :c 3} '(:a :c))").await;
    match val {
        Value::Map(m) => {
            assert_eq!(m.len(), 2);
            assert_eq!(
                m.get(&Value::Keyword("a".to_string())),
                Some(&Value::Integer(1))
            );
            assert_eq!(
                m.get(&Value::Keyword("c".to_string())),
                Some(&Value::Integer(3))
            );
        }
        _ => panic!("Expected Map"),
    }
}

#[tokio::test]
async fn test_select_keys_missing() {
    let val = eval_str("(select-keys {:a 1} '(:a :x))").await;
    match val {
        Value::Map(m) => {
            assert_eq!(m.len(), 1);
            assert_eq!(
                m.get(&Value::Keyword("a".to_string())),
                Some(&Value::Integer(1))
            );
        }
        _ => panic!("Expected Map"),
    }
}

#[tokio::test]
async fn test_select_keys_nil_map() {
    assert_eq!(eval_str("(select-keys nil '(:a))").await, Value::Nil);
}

#[tokio::test]
async fn test_select_keys_vector_keys() {
    let val = eval_str("(select-keys {:a 1 :b 2} [:a])").await;
    match val {
        Value::Map(m) => {
            assert_eq!(m.len(), 1);
            assert_eq!(
                m.get(&Value::Keyword("a".to_string())),
                Some(&Value::Integer(1))
            );
        }
        _ => panic!("Expected Map"),
    }
}
