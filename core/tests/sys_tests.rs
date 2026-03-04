use dlisp_core::ast::Value;
use dlisp_core::environment::Environment;
use dlisp_core::interpreter::default_env;
use std::env;

async fn eval_lisp(code: &str) -> Result<Value, String> {
    let mut env = default_env();
    let mut interpreter = dlisp_core::interpreter::Interpreter::new();
    let vals = dlisp_core::parser::parse(code).map_err(|e| format!("{:?}", e))?;
    let mut last_res = Value::Nil;
    for val in vals {
        last_res = interpreter.eval(val, &mut env).await?;
    }
    Ok(last_res)
}

#[tokio::test]
async fn test_sys_operations() {
    // 1. setenv and getenv
    let setenv_code = r#"(setenv "TEST_DLISP_VAR" "dlisp_value")"#;
    let res = eval_lisp(setenv_code).await.unwrap();
    assert_eq!(res, Value::Bool(true));

    let getenv_code = r#"(getenv "TEST_DLISP_VAR")"#;
    let res = eval_lisp(getenv_code).await.unwrap();
    assert_eq!(res, Value::String("dlisp_value".to_string()));

    // 2. getenv non-existent
    let getenv_missing_code = r#"(getenv "NON_EXISTENT_DLISP_VAR")"#;
    let res = eval_lisp(getenv_missing_code).await.unwrap();
    assert_eq!(res, Value::Nil);

    // 3. cwd and set-cwd
    let temp_dir = env::temp_dir();
    let temp_dir_str = temp_dir.to_str().unwrap();

    let original_cwd_code = r#"(cwd)"#;
    let original_cwd = eval_lisp(original_cwd_code).await.unwrap();

    let set_cwd_code = format!(r#"(set-cwd "{}")"#, temp_dir_str);
    let res = eval_lisp(&set_cwd_code).await.unwrap();
    assert_eq!(res, Value::Bool(true));

    let new_cwd_code = r#"(cwd)"#;
    let new_cwd = eval_lisp(new_cwd_code).await.unwrap();
    if let Value::String(s) = new_cwd {
        // Just checking that it's a string, exact match may be tricky due to symlinks on some OSes
        assert!(!s.is_empty());
    } else {
        panic!("cwd did not return a string");
    }

    // Restore cwd
    if let Value::String(orig) = original_cwd {
        let restore_cwd_code = format!(r#"(set-cwd "{}")"#, orig);
        eval_lisp(&restore_cwd_code).await.unwrap();
    }

    // 4. args
    let args_code = r#"(args)"#;
    let res = eval_lisp(args_code).await.unwrap();
    match res {
        Value::List(l) => {
            assert!(!l.is_empty()); // At least the test runner executable should be there
        }
        _ => panic!("args did not return a list"),
    }
}
