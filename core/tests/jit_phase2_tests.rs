use dlisp_core::ast::Value;
use dlisp_core::interpreter::{Interpreter, default_env};
use dlisp_core::parser::parse;

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
async fn test_jit_arithmetic_div_mod() {
    // Division
    let result = eval_code("(/ 10 2)").await;
    assert_eq!(result, Value::Integer(5));

    let result = eval_code("(/ 10 3)").await; // Integer division
    assert_eq!(result, Value::Integer(3));

    let result = eval_code("(/ 10.0 2.0)").await;
    assert_eq!(result, Value::Float(5.0));

    // Modulo
    let result = eval_code("(% 10 3)").await;
    assert_eq!(result, Value::Integer(1));

    let result = eval_code("(mod 10 3)").await;
    assert_eq!(result, Value::Integer(1));
}

#[tokio::test]
async fn test_jit_comparison() {
    // >=
    let result = eval_code("(>= 10 5)").await;
    assert_eq!(result, Value::Bool(true));
    let result = eval_code("(>= 5 5)").await;
    assert_eq!(result, Value::Bool(true));
    let result = eval_code("(>= 4 5)").await;
    assert_eq!(result, Value::Bool(false));

    // <=
    let result = eval_code("(<= 5 10)").await;
    assert_eq!(result, Value::Bool(true));
    let result = eval_code("(<= 5 5)").await;
    assert_eq!(result, Value::Bool(true));
    let result = eval_code("(<= 6 5)").await;
    assert_eq!(result, Value::Bool(false));

    // /=
    let result = eval_code("(/= 10 5)").await;
    assert_eq!(result, Value::Bool(true));
    let result = eval_code("(/= 5 5)").await;
    assert_eq!(result, Value::Bool(false));
}

#[tokio::test]
async fn test_jit_string_ops() {
    // str
    let result = eval_code("(str 123)").await;
    match result {
        Value::String(s) => assert_eq!(s, "123"),
        _ => panic!("Expected string"),
    }

    // string-length
    let result = eval_code("(string-length \"hello\")").await;
    assert_eq!(result, Value::Integer(5));

    // substring
    let result = eval_code("(substring \"hello\" 1 3)").await;
    match result {
        Value::String(s) => assert_eq!(s, "el"),
        _ => panic!("Expected string"),
    }

    // string-append
    let result = eval_code("(string-append \"foo\" \"bar\")").await;
    match result {
        Value::String(s) => assert_eq!(s, "foobar"),
        _ => panic!("Expected string"),
    }
}

#[tokio::test]
async fn test_jit_type_predicates() {
    assert_eq!(eval_code("(nil? nil)").await, Value::Bool(true));
    assert_eq!(eval_code("(nil? 1)").await, Value::Bool(false));

    assert_eq!(eval_code("(number? 1)").await, Value::Bool(true));
    assert_eq!(eval_code("(number? 1.5)").await, Value::Bool(true));
    assert_eq!(eval_code("(number? \"s\")").await, Value::Bool(false));

    assert_eq!(eval_code("(string? \"s\")").await, Value::Bool(true));
    assert_eq!(eval_code("(string? 1)").await, Value::Bool(false));

    assert_eq!(eval_code("(list? '(1 2))").await, Value::Bool(true));
    assert_eq!(eval_code("(list? nil)").await, Value::Bool(true)); // nil is list

    // type-of
    let result = eval_code("(type-of 1)").await;
    match result {
        Value::String(s) => assert_eq!(s, "integer"),
        _ => panic!("Expected string"),
    }
}

// Ensure these functions compile via AOT/JIT by wrapping in defun
#[tokio::test]
async fn test_jit_compiled_function_execution() {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();

    // Define a function that uses new features to force JIT compilation of them
    let code = "
    (defun calc_stuff (x y)
        (if (>= x y)
            (string-append \"res:\" (str (/ x y)))
            (string-append \"res:\" (str (% y x)))))
    
    (calc_stuff 10 2)
    ";

    let vals = parse(code).unwrap();
    let mut result = Value::Nil;
    for val in vals {
        result = interpreter.eval(val, &mut env).await.unwrap();
    }

    match result {
        Value::String(s) => assert_eq!(s, "res:5"),
        _ => panic!("Expected string result, got {:?}", result),
    }

    // Call again with other branch
    let code2 = "(calc_stuff 3 10)";
    let vals2 = parse(code2).unwrap();
    let result2 = interpreter.eval(vals2[0].clone(), &mut env).await.unwrap();

    match result2 {
        Value::String(s) => assert_eq!(s, "res:1"), // 10 % 3 = 1
        _ => panic!("Expected string result, got {:?}", result2),
    }
}
