use dlisp_core::ast::Value;
use dlisp_core::interpreter::{Interpreter, default_env};
use dlisp_core::parser::parse;
use dlisp_runtime as _;

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

    match env.borrow().get("max_val") {
        Some(Value::UserFunc { jit_code, .. }) => assert!(jit_code.is_some()),
        other => panic!("expected compiled user func, got {:?}", other),
    }

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

#[tokio::test]
async fn test_aot_multiple_functions() {
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
    compiler.compile(vals).await.expect("Compilation failed");
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

#[tokio::test]
async fn test_jit_recursion_fibonacci() {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();

    let code = "
    (defun fib (n)
        (if (< n 2)
            n
            (+ (fib (- n 1)) (fib (- n 2)))))
    
    (fib 10)
    ";

    let vals = parse(code).unwrap();
    let mut result = Value::Nil;
    for val in vals {
        result = interpreter.eval(val, &mut env).await.unwrap();
    }

    match env.borrow().get("fib") {
        Some(Value::UserFunc { jit_code, .. }) => assert!(jit_code.is_some()),
        other => panic!("expected compiled user func, got {:?}", other),
    }

    assert_eq!(result, Value::Integer(55));
}

#[tokio::test]
async fn test_jit_rest_param_compilation_and_execution() {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();

    let code = "
    (defun pack (head &rest tail)
        (conj tail head))

    (pack 1 2 3)
    ";

    let vals = parse(code).unwrap();
    let mut result = Value::Nil;
    for val in vals {
        result = interpreter.eval(val, &mut env).await.unwrap();
    }

    match env.borrow().get("pack") {
        Some(Value::UserFunc { jit_code, .. }) => assert!(jit_code.is_some()),
        other => panic!("expected compiled user func, got {:?}", other),
    }

    assert_eq!(
        result,
        Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
}

#[tokio::test]
async fn test_jit_mutual_recursion() {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();

    let code = "
    (defun is_even (n)
        (if (= n 0)
            1  
            (is_odd (- n 1))))
            
    (defun is_odd (n)
        (if (= n 0)
            0  
            (is_even (- n 1))))
            
    (is_even 10)
    ";

    let vals = parse(code).unwrap();
    let mut result = Value::Nil;
    for val in vals {
        result = interpreter.eval(val, &mut env).await.unwrap();
    }

    match env.borrow().get("is_even") {
        Some(Value::UserFunc { jit_code, .. }) => assert!(jit_code.is_some()),
        other => panic!("expected compiled user func, got {:?}", other),
    }
    match env.borrow().get("is_odd") {
        Some(Value::UserFunc { jit_code, .. }) => assert!(jit_code.is_some()),
        other => panic!("expected compiled user func, got {:?}", other),
    }

    // 1 is truthy in our dummy logical context for now
    assert_eq!(result, Value::Integer(1));

    let code_odd = "(is_odd 10)";
    let vals_odd = parse(code_odd).unwrap();
    let mut result_odd = Value::Nil;
    for val in vals_odd {
        result_odd = interpreter.eval(val, &mut env).await.unwrap();
    }
    assert_eq!(result_odd, Value::Integer(0));
}

#[tokio::test]
async fn test_jit_spawn_function() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            let mut env = default_env();
            let mut interpreter = Interpreter::new();

            let code = "
            (defun task ()
                (print 42)
                (sleep 1))

            (defun launch_task ()
                (spawn task))

            (launch_task)
            ";

            let mut result = Value::Nil;
            let vals = parse(code).unwrap();
            for val in vals {
                result = interpreter.eval(val, &mut env).await.unwrap();
            }

            match env.borrow().get("task") {
                Some(Value::UserFunc { jit_code, .. }) => assert!(jit_code.is_some()),
                other => panic!("expected compiled user func, got {:?}", other),
            }
            match env.borrow().get("launch_task") {
                Some(Value::UserFunc { jit_code, .. }) => assert!(jit_code.is_some()),
                other => panic!("expected compiled user func, got {:?}", other),
            }

            assert_eq!(result, Value::Nil);
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        })
        .await;
}

#[tokio::test]
async fn test_jit_spawn_function_with_arguments() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            let mut env = default_env();
            let mut interpreter = Interpreter::new();

            let code = "
            (defun task (x y)
                (print (+ x y))
                (sleep 1))

            (defun launch_task ()
                (spawn task 10 20))

            (launch_task)
            ";

            let mut result = Value::Nil;
            let vals = parse(code).unwrap();
            for val in vals {
                result = interpreter.eval(val, &mut env).await.unwrap();
            }

            match env.borrow().get("task") {
                Some(Value::UserFunc { jit_code, .. }) => assert!(jit_code.is_some()),
                other => panic!("expected compiled user func, got {:?}", other),
            }
            match env.borrow().get("launch_task") {
                Some(Value::UserFunc { jit_code, .. }) => assert!(jit_code.is_some()),
                other => panic!("expected compiled user func, got {:?}", other),
            }

            assert_eq!(result, Value::Nil);
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        })
        .await;
}
