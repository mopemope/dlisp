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

#[tokio::test]
async fn test_map_basic() {
    let (mut i, mut e) = setup();
    let res = eval_str("(map (lambda (x) (* x 2)) '(1 2 3))", &mut i, &mut e).await;
    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(2),
            Value::Integer(4),
            Value::Integer(6)
        ])
    );
}

#[tokio::test]
async fn test_map_empty() {
    let (mut i, mut e) = setup();
    let res = eval_str("(map (lambda (x) (* x 2)) '())", &mut i, &mut e).await;
    assert_eq!(res, Value::List(vec![]));

    let res_nil = eval_str("(map (lambda (x) (* x 2)) nil)", &mut i, &mut e).await;
    assert_eq!(res_nil, Value::Nil);
}

#[tokio::test]
async fn test_map_named_func() {
    let (mut i, mut e) = setup();
    eval_str("(defun double (x) (* x 2))", &mut i, &mut e).await;
    let res = eval_str("(map double '(10 20))", &mut i, &mut e).await;
    assert_eq!(
        res,
        Value::List(vec![Value::Integer(20), Value::Integer(40)])
    );
}

#[tokio::test]
async fn test_filter_basic() {
    let (mut i, mut e) = setup();
    let res = eval_str("(filter (lambda (x) (> x 2)) '(1 2 3 4 5))", &mut i, &mut e).await;
    assert_eq!(
        res,
        Value::List(vec![
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5)
        ])
    );
}

#[tokio::test]
async fn test_filter_all_out() {
    let (mut i, mut e) = setup();
    let res = eval_str("(filter (lambda (x) (> x 10)) '(1 2 3))", &mut i, &mut e).await;
    assert_eq!(res, Value::List(vec![]));
}

#[tokio::test]
async fn test_filter_empty() {
    let (mut i, mut e) = setup();
    let res = eval_str("(filter (lambda (x) (> x 0)) '())", &mut i, &mut e).await;
    assert_eq!(res, Value::List(vec![]));
}

#[tokio::test]
async fn test_reduce_basic() {
    let (mut i, mut e) = setup();
    let res = eval_str("(reduce + 0 '(1 2 3 4 5))", &mut i, &mut e).await;
    assert_eq!(res, Value::Integer(15));
}

#[tokio::test]
async fn test_reduce_mul() {
    let (mut i, mut e) = setup();
    let res = eval_str("(reduce * 1 '(1 2 3 4))", &mut i, &mut e).await;
    assert_eq!(res, Value::Integer(24));
}

#[tokio::test]
async fn test_reduce_lambda() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(reduce (lambda (acc x) (+ acc x)) 10 '(1 2 3))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Integer(16));
}

#[tokio::test]
async fn test_reduce_empty() {
    let (mut i, mut e) = setup();
    let res = eval_str("(reduce + 42 '())", &mut i, &mut e).await;
    assert_eq!(res, Value::Integer(42));
}

#[tokio::test]
async fn test_higher_order_composition() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(reduce + 0 (map (lambda (x) (* x x)) (filter (lambda (x) (> x 2)) '(1 2 3 4))))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Integer(25));
}

// --- Error Cases & Edge Cases ---

#[tokio::test]
async fn test_map_wrong_args() {
    let (mut i, mut e) = setup();
    let exprs = parse("(map (lambda (x) x))").unwrap(); // Missing list
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("exactly 2 arguments"));
}

#[tokio::test]
async fn test_map_not_a_list() {
    let (mut i, mut e) = setup();
    let exprs = parse("(map (lambda (x) x) 42)").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("expects a list"));
}

#[tokio::test]
async fn test_map_not_a_func() {
    let (mut i, mut e) = setup();
    let exprs = parse("(map 42 '(1 2 3))").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("not a function"));
}

#[tokio::test]
async fn test_filter_truthiness() {
    let (mut i, mut e) = setup();
    // In DLisp, everything except nil, false, 0 is truthy.
    // Let's test filter with 0 and nil.
    let res = eval_str(
        "(filter (lambda (x) (if (= x 2) 0 1)) '(1 2 3))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::List(vec![Value::Integer(1), Value::Integer(3)]));
}

#[tokio::test]
async fn test_reduce_error_in_func() {
    let (mut i, mut e) = setup();
    let exprs = parse("(reduce (lambda (acc x) (/ acc 0)) 10 '(1 2 3))").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("zero"));
}

#[tokio::test]
async fn test_filter_wrong_args() {
    let (mut i, mut e) = setup();
    let exprs = parse("(filter (lambda (x) x))").unwrap(); // Missing list
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("exactly 2 arguments"));
}

#[tokio::test]
async fn test_filter_not_a_list() {
    let (mut i, mut e) = setup();
    let exprs = parse("(filter (lambda (x) x) 42)").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("expects a list"));
}

#[tokio::test]
async fn test_filter_not_a_func() {
    let (mut i, mut e) = setup();
    let exprs = parse("(filter 42 '(1 2 3))").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("not a function"));
}

#[tokio::test]
async fn test_reduce_wrong_args() {
    let (mut i, mut e) = setup();
    let exprs = parse("(reduce + 0)").unwrap(); // Missing list
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("exactly 3 arguments"));
}

#[tokio::test]
async fn test_reduce_not_a_list() {
    let (mut i, mut e) = setup();
    let exprs = parse("(reduce + 0 42)").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("expects a list"));
}

#[tokio::test]
async fn test_reduce_not_a_func() {
    let (mut i, mut e) = setup();
    let exprs = parse("(reduce 42 0 '(1 2 3))").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("not a function"));
}

#[tokio::test]
async fn test_map_vector() {
    let (mut i, mut e) = setup();
    let res = eval_str("(map (lambda (x) (* x 2)) [1 2 3])", &mut i, &mut e).await;
    assert_eq!(
        res,
        Value::Vector(vec![
            Value::Integer(2),
            Value::Integer(4),
            Value::Integer(6)
        ])
    );
}

#[tokio::test]
async fn test_filter_vector() {
    let (mut i, mut e) = setup();
    let res = eval_str("(filter (lambda (x) (> x 2)) [1 2 3 4 5])", &mut i, &mut e).await;
    assert_eq!(
        res,
        Value::Vector(vec![
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5)
        ])
    );
}

#[tokio::test]
async fn test_reduce_vector() {
    let (mut i, mut e) = setup();
    let res = eval_str("(reduce + 0 [1 2 3 4 5])", &mut i, &mut e).await;
    assert_eq!(res, Value::Integer(15));
}

#[tokio::test]
async fn test_some_basic() {
    let (mut i, mut e) = setup();
    let res = eval_str("(some (lambda (x) (> x 2)) '(1 2 3))", &mut i, &mut e).await;
    assert_eq!(res, Value::Bool(true));

    let res2 = eval_str("(some (lambda (x) (> x 5)) '(1 2 3))", &mut i, &mut e).await;
    assert_eq!(res2, Value::Nil);
}

#[tokio::test]
async fn test_some_returning_truthy() {
    let (mut i, mut e) = setup();
    // Return the actual truthy value
    let res = eval_str(
        "(some (lambda (x) (if (> x 2) x nil)) '(1 3 5))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Integer(3));
}

#[tokio::test]
async fn test_every_basic() {
    let (mut i, mut e) = setup();
    let res = eval_str("(every (lambda (x) (> x 0)) '(1 2 3))", &mut i, &mut e).await;
    assert_eq!(res, Value::Bool(true));

    let res2 = eval_str("(every (lambda (x) (> x 2)) '(1 2 3))", &mut i, &mut e).await;
    assert_eq!(res2, Value::Bool(false));
}

#[tokio::test]
async fn test_find_basic() {
    let (mut i, mut e) = setup();
    let res = eval_str("(find (lambda (x) (> x 2)) '(1 2 3 4))", &mut i, &mut e).await;
    assert_eq!(res, Value::Integer(3));

    let res2 = eval_str("(find (lambda (x) (> x 5)) '(1 2 3))", &mut i, &mut e).await;
    assert_eq!(res2, Value::Nil);
}

#[tokio::test]
async fn test_for_each_basic() {
    let (mut i, mut e) = setup();
    // Use for-each to update a global variable side-effectfully
    eval_str("(defvar *sum* 0)", &mut i, &mut e).await;
    let res = eval_str(
        "(for-each (lambda (x) (setq *sum* (+ *sum* x))) '(1 2 3))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Nil);

    let sum = eval_str("*sum*", &mut i, &mut e).await;
    assert_eq!(sum, Value::Integer(6));
}

#[tokio::test]
async fn test_map_indexed_basic() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(map-indexed (lambda (idx val) (list idx val)) '(\"a\" \"b\" \"c\"))",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(
        res,
        Value::List(vec![
            Value::List(vec![Value::Integer(0), Value::String("a".to_string())]),
            Value::List(vec![Value::Integer(1), Value::String("b".to_string())]),
            Value::List(vec![Value::Integer(2), Value::String("c".to_string())]),
        ])
    );
}

#[tokio::test]
async fn test_map_indexed_vector() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(map-indexed (lambda (idx val) (list idx val)) [\"a\" \"b\"])",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(
        res,
        Value::Vector(vec![
            Value::List(vec![Value::Integer(0), Value::String("a".to_string())]),
            Value::List(vec![Value::Integer(1), Value::String("b".to_string())]),
        ])
    );
}

// --- Edge case tests for new higher-order functions ---

#[tokio::test]
async fn test_some_empty_list() {
    let (mut i, mut e) = setup();
    let res = eval_str("(some (lambda (x) (> x 0)) '())", &mut i, &mut e).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_some_nil() {
    let (mut i, mut e) = setup();
    let res = eval_str("(some (lambda (x) (> x 0)) nil)", &mut i, &mut e).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_some_vector() {
    let (mut i, mut e) = setup();
    let res = eval_str("(some (lambda (x) (> x 2)) [1 2 3])", &mut i, &mut e).await;
    assert_eq!(res, Value::Bool(true));
}

#[tokio::test]
async fn test_some_wrong_args() {
    let (mut i, mut e) = setup();
    let exprs = parse("(some (lambda (x) x))").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("exactly 2 arguments"));
}

#[tokio::test]
async fn test_some_not_a_list() {
    let (mut i, mut e) = setup();
    let exprs = parse("(some (lambda (x) x) 42)").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("expects a list"));
}

#[tokio::test]
async fn test_every_empty_list() {
    let (mut i, mut e) = setup();
    let res = eval_str("(every (lambda (x) (> x 0)) '())", &mut i, &mut e).await;
    assert_eq!(res, Value::Bool(true));
}

#[tokio::test]
async fn test_every_nil() {
    let (mut i, mut e) = setup();
    let res = eval_str("(every (lambda (x) (> x 0)) nil)", &mut i, &mut e).await;
    assert_eq!(res, Value::Bool(true));
}

#[tokio::test]
async fn test_every_vector() {
    let (mut i, mut e) = setup();
    let res = eval_str("(every (lambda (x) (> x 0)) [1 2 3])", &mut i, &mut e).await;
    assert_eq!(res, Value::Bool(true));
}

#[tokio::test]
async fn test_every_wrong_args() {
    let (mut i, mut e) = setup();
    let exprs = parse("(every number?)").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_find_empty_list() {
    let (mut i, mut e) = setup();
    let res = eval_str("(find (lambda (x) (> x 0)) '())", &mut i, &mut e).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_find_nil() {
    let (mut i, mut e) = setup();
    let res = eval_str("(find (lambda (x) (> x 0)) nil)", &mut i, &mut e).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_find_vector() {
    let (mut i, mut e) = setup();
    let res = eval_str("(find (lambda (x) (> x 2)) [1 2 3 4])", &mut i, &mut e).await;
    assert_eq!(res, Value::Integer(3));
}

#[tokio::test]
async fn test_find_wrong_args() {
    let (mut i, mut e) = setup();
    let exprs = parse("(find (lambda (x) x))").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_for_each_empty() {
    let (mut i, mut e) = setup();
    let res = eval_str("(for-each (lambda (x) x) '())", &mut i, &mut e).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_for_each_nil() {
    let (mut i, mut e) = setup();
    let res = eval_str("(for-each (lambda (x) x) nil)", &mut i, &mut e).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_for_each_wrong_args() {
    let (mut i, mut e) = setup();
    let exprs = parse("(for-each (lambda (x) x))").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_map_indexed_empty() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(map-indexed (lambda (i x) (list i x)) '())",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::List(vec![]));
}

#[tokio::test]
async fn test_map_indexed_nil() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(map-indexed (lambda (i x) (list i x)) nil)",
        &mut i,
        &mut e,
    )
    .await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_map_indexed_wrong_args() {
    let (mut i, mut e) = setup();
    let exprs = parse("(map-indexed (lambda (i x) x))").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_some_all_falsy() {
    let (mut i, mut e) = setup();
    let res = eval_str("(some (lambda (x) (> x 10)) '(1 2 3))", &mut i, &mut e).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_every_all_truthy() {
    let (mut i, mut e) = setup();
    let res = eval_str("(every (lambda (x) (< x 10)) '(1 2 3))", &mut i, &mut e).await;
    assert_eq!(res, Value::Bool(true));
}

#[tokio::test]
async fn test_find_with_named_func() {
    let (mut i, mut e) = setup();
    eval_str("(defun positive? (x) (> x 0))", &mut i, &mut e).await;
    let res = eval_str("(find positive? '(-1 -2 3 4))", &mut i, &mut e).await;
    assert_eq!(res, Value::Integer(3));
}

#[tokio::test]
async fn test_for_each_vector() {
    let (mut i, mut e) = setup();
    eval_str("(defvar *items* '())", &mut i, &mut e).await;
    eval_str(
        "(for-each (lambda (x) (setq *items* (append *items* (list x)))) [10 20 30])",
        &mut i,
        &mut e,
    )
    .await;
    let items = eval_str("*items*", &mut i, &mut e).await;
    assert_eq!(
        items,
        Value::List(vec![
            Value::Integer(10),
            Value::Integer(20),
            Value::Integer(30)
        ])
    );
}

// --- Map higher-order function tests ---

#[tokio::test]
async fn test_update_basic() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(update {:a 1 :b 2} :a (lambda (x) (+ x 1)))",
        &mut i,
        &mut e,
    )
    .await;

    if let Value::Map(m) = res {
        assert_eq!(
            m.get(&Value::Keyword("a".to_string())),
            Some(&Value::Integer(2))
        );
        assert_eq!(
            m.get(&Value::Keyword("b".to_string())),
            Some(&Value::Integer(2))
        );
    } else {
        panic!("Expected Map");
    }
}

#[tokio::test]
async fn test_update_missing_key() {
    let (mut i, mut e) = setup();
    // Default handles nil (missing key defaults to nil)
    let res = eval_str(
        "(update {:a 1} :b (lambda (x) (if (nil? x) 42 x)))",
        &mut i,
        &mut e,
    )
    .await;

    if let Value::Map(m) = res {
        assert_eq!(
            m.get(&Value::Keyword("a".to_string())),
            Some(&Value::Integer(1))
        );
        assert_eq!(
            m.get(&Value::Keyword("b".to_string())),
            Some(&Value::Integer(42))
        );
    } else {
        panic!("Expected Map");
    }
}

#[tokio::test]
async fn test_update_with_extra_args() {
    let (mut i, mut e) = setup();
    let res = eval_str("(update {:a 1} :a + 10 20)", &mut i, &mut e).await;

    if let Value::Map(m) = res {
        // (+ 1 10 20) = 31
        assert_eq!(
            m.get(&Value::Keyword("a".to_string())),
            Some(&Value::Integer(31))
        );
    } else {
        panic!("Expected Map");
    }
}

#[tokio::test]
async fn test_update_nil() {
    let (mut i, mut e) = setup();
    let res = eval_str("(update nil :a (lambda (x) 100))", &mut i, &mut e).await;

    if let Value::Map(m) = res {
        assert_eq!(
            m.get(&Value::Keyword("a".to_string())),
            Some(&Value::Integer(100))
        );
    } else {
        panic!("Expected Map");
    }
}

#[tokio::test]
async fn test_map_keys_basic() {
    let (mut i, mut e) = setup();
    // Transform string keys to uppercase using `str` (or simple function)
    eval_str("(defun prefix-key (k) (str \"prefix-\" k))", &mut i, &mut e).await;
    let res = eval_str("(map-keys prefix-key {\"a\" 1 \"b\" 2})", &mut i, &mut e).await;

    if let Value::Map(m) = res {
        assert_eq!(
            m.get(&Value::String("prefix-a".to_string())),
            Some(&Value::Integer(1))
        );
        assert_eq!(
            m.get(&Value::String("prefix-b".to_string())),
            Some(&Value::Integer(2))
        );
    } else {
        panic!("Expected Map");
    }
}

#[tokio::test]
async fn test_map_keys_nil() {
    let (mut i, mut e) = setup();
    let res = eval_str("(map-keys (lambda (x) x) nil)", &mut i, &mut e).await;
    assert_eq!(res, Value::Nil);
}

#[tokio::test]
async fn test_map_vals_basic() {
    let (mut i, mut e) = setup();
    let res = eval_str(
        "(map-vals (lambda (x) (* x 2)) {:a 1 :b 2})",
        &mut i,
        &mut e,
    )
    .await;

    if let Value::Map(m) = res {
        assert_eq!(
            m.get(&Value::Keyword("a".to_string())),
            Some(&Value::Integer(2))
        );
        assert_eq!(
            m.get(&Value::Keyword("b".to_string())),
            Some(&Value::Integer(4))
        );
    } else {
        panic!("Expected Map");
    }
}

#[tokio::test]
async fn test_map_vals_nil() {
    let (mut i, mut e) = setup();
    let res = eval_str("(map-vals (lambda (x) x) nil)", &mut i, &mut e).await;
    assert_eq!(res, Value::Nil);
}

// --- Edge case / error tests for map higher-order functions ---

#[tokio::test]
async fn test_update_wrong_args() {
    let (mut i, mut e) = setup();
    // Too few arguments: should error
    let exprs = parse("(update {:a 1} :a)").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_update_wrong_map_type() {
    let (mut i, mut e) = setup();
    // First arg is not a map
    let exprs = parse("(update 42 :a (lambda (x) x))").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_map_keys_wrong_args() {
    let (mut i, mut e) = setup();
    // Too few arguments
    let exprs = parse("(map-keys (lambda (x) x))").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_map_keys_wrong_type() {
    let (mut i, mut e) = setup();
    // Second arg is not a map
    let exprs = parse("(map-keys (lambda (x) x) 42)").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_map_vals_wrong_args() {
    let (mut i, mut e) = setup();
    // Too many arguments
    let exprs = parse("(map-vals (lambda (x) x) {:a 1} {:b 2})").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_map_vals_wrong_type() {
    let (mut i, mut e) = setup();
    let exprs = parse("(map-vals (lambda (x) x) \"not-a-map\")").unwrap();
    let res = i.eval(exprs[0].clone(), &mut e).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_map_keys_collision() {
    let (mut i, mut e) = setup();
    // Map all keys to the same value — later entries overwrite earlier ones
    let res = eval_str(
        "(map-keys (lambda (k) \"same\") {\"a\" 1 \"b\" 2})",
        &mut i,
        &mut e,
    )
    .await;
    if let Value::Map(m) = res {
        // Both keys mapped to "same", so only one entry survives
        assert_eq!(m.len(), 1);
        assert!(m.contains_key(&Value::String("same".to_string())));
    } else {
        panic!("Expected Map");
    }
}
