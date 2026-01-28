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
