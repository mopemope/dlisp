use dlisp_core::ast::Value;
use dlisp_core::interpreter::{Interpreter, default_env};
use dlisp_core::parser::parse;

#[tokio::test]
async fn test_macro_simple() {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();

    // (defmacro unless (cond body) (list 'if cond 'nil body))
    let code = "
    (defmacro unless (cond body) 
        (list 'if cond 'nil body))
    
    (unless (> 1 2) 42)
    ";

    let vals = parse(code).unwrap();
    let mut result = Value::Nil;
    for val in vals {
        result = interpreter.eval(val, &mut env).await.unwrap();
    }

    // (> 1 2) is false (nil), so if nil nil 42 -> 42
    assert_eq!(result, Value::Integer(42));
}

#[tokio::test]
async fn test_macro_expansion() {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();

    // Define a macro that doubles its argument at compile time(expansion time)? NO.
    // Macros return AST.
    // (defmacro double_val (x) (list '+ x x))
    let code = "
    (defmacro double_val (x) 
        (list '+ x x))
    
    (double_val 10)
    ";

    let vals = parse(code).unwrap();
    let mut result = Value::Nil;
    for val in vals {
        result = interpreter.eval(val, &mut env).await.unwrap();
    }

    assert_eq!(result, Value::Integer(20));
}

#[tokio::test]
async fn test_macro_recursive_expansion() {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();

    // Macro expanding into another macro
    // (defmacro inc (x) (list '+ x 1))
    // (defmacro inc2 (x) (list 'inc (list 'inc x)))
    let code = "
    (defmacro inc (x) (list '+ x 1))
    (defmacro inc2 (x) (list 'inc (list 'inc x)))
    
    (inc2 10)
    ";

    let vals = parse(code).unwrap();
    let mut result = Value::Nil;
    for val in vals {
        result = interpreter.eval(val, &mut env).await.unwrap();
    }

    assert_eq!(result, Value::Integer(12));
}
