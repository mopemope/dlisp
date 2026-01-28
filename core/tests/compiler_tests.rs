use dlisp_core::ast::Value;
use dlisp_core::interpreter::{Interpreter, default_env};
use dlisp_core::parser::parse;

#[tokio::test]
async fn test_jit_print() {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();

    // Test if print works in JIT
    let code = "
    (defun test_print (x)
        (print x))
    (test_print 123)
    ";

    let vals = parse(code).unwrap();
    for val in vals {
        interpreter.eval(val, &mut env).await.unwrap();
    }
}

#[tokio::test]
async fn test_jit_if_gt() {
    let env = default_env();
    let mut interpreter = Interpreter::new();

    // (defun max_val (a b) (if (> a b) a b))
    let defun_max = Value::List(vec![
        Value::Symbol("defun".to_string()),
        Value::Symbol("max_val".to_string()),
        Value::List(vec![
            Value::Symbol("a".to_string()),
            Value::Symbol("b".to_string()),
        ]),
        Value::List(vec![
            Value::Symbol("if".to_string()),
            Value::List(vec![
                Value::Symbol(">".to_string()),
                Value::Symbol("a".to_string()),
                Value::Symbol("b".to_string()),
            ]),
            Value::Symbol("a".to_string()),
            Value::Symbol("b".to_string()),
        ]),
    ]);

    interpreter.eval(defun_max, &mut env.clone()).await.unwrap();

    let call_1 = Value::List(vec![
        Value::Symbol("max_val".to_string()),
        Value::Integer(10),
        Value::Integer(20),
    ]);
    assert_eq!(
        interpreter.eval(call_1, &mut env.clone()).await.unwrap(),
        Value::Integer(20)
    );

    let call_2 = Value::List(vec![
        Value::Symbol("max_val".to_string()),
        Value::Integer(30),
        Value::Integer(15),
    ]);
    assert_eq!(
        interpreter.eval(call_2, &mut env.clone()).await.unwrap(),
        Value::Integer(30)
    );
}

#[tokio::test]
async fn test_multiple_functions() {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();

    // Test defining multiple functions to check for duplicate symbol definition errors
    let code = "
    (defun func_a (x) (+ x 1))
    (defun func_b (x) (- x 1))
    (print (func_a 10))
    (print (func_b 10))
    ";

    let vals = parse(code).unwrap();
    for val in vals {
        interpreter.eval(val, &mut env).await.unwrap();
    }
}

#[test]
fn test_aot_multiple_functions() {
    use dlisp_core::compiler::AOTCompiler;

    let code = "
    (defun func_a (x) (+ x 1))
    (defun func_b (x) (- x 1))
    (defun main () 
        (print (func_a 10)) 
        (print (func_b 10)))
    ";

    let vals = parse(code).unwrap();
    let compiler = AOTCompiler::new();
    compiler.compile(vals).expect("Compilation failed");
}

#[tokio::test]
async fn test_mixed_execution() {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();

    // Test mixing builtins (+) and user function calls in nested positions
    let code = "
    (defun inc (x) (+ x 1))
    (defun double (x) (+ x x))
    (defun complex_calc (x) 
        (double (+ (inc x) 2)))
    
    (print (complex_calc 5))
    ";

    // (inc 5) -> 6
    // (+ 6 2) -> 8
    // (double 8) -> 16

    let vals = parse(code).unwrap();
    for val in vals {
        interpreter.eval(val, &mut env).await.unwrap();
    }
}
