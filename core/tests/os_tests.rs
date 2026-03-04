use dlisp_core::ast::Value;
use dlisp_core::interpreter::default_env;

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
async fn test_os_sh_single_arg() {
    let code = r#"(sh "echo hello")"#;
    let res = eval_lisp(code).await.unwrap();

    match res {
        Value::Map(m) => {
            let status = m.get(&Value::Keyword("status".to_string())).unwrap();
            let stdout = m.get(&Value::Keyword("stdout".to_string())).unwrap();

            assert_eq!(status, &Value::Integer(0));
            if let Value::String(s) = stdout {
                assert_eq!(s.trim(), "hello");
            } else {
                panic!("stdout is not a string");
            }
        }
        _ => panic!("Expected a map"),
    }
}

#[tokio::test]
async fn test_os_sh_multi_arg() {
    let code = r#"(sh "echo" "hello" "world")"#;
    let res = eval_lisp(code).await.unwrap();

    match res {
        Value::Map(m) => {
            let status = m.get(&Value::Keyword("status".to_string())).unwrap();
            let stdout = m.get(&Value::Keyword("stdout".to_string())).unwrap();

            assert_eq!(status, &Value::Integer(0));
            if let Value::String(s) = stdout {
                // "echo hello world"
                assert_eq!(s.trim(), "hello world");
            } else {
                panic!("stdout is not a string");
            }
        }
        _ => panic!("Expected a map"),
    }
}

#[tokio::test]
async fn test_os_sh_error() {
    let code = r#"(sh "non_existent_command_dlisp")"#;
    let res = eval_lisp(code).await.unwrap();

    match res {
        Value::Map(m) => {
            let status = m.get(&Value::Keyword("status".to_string())).unwrap();
            // Should be non-zero status code
            if let Value::Integer(i) = status {
                assert!(*i != 0);
            } else {
                panic!("status is not an integer");
            }
        }
        _ => panic!("Expected a map"),
    }
}
