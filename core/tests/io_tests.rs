use dlisp_core::ast::Value;
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
async fn test_fs_operations() {
    let temp_dir = env::temp_dir();
    let test_file_path = temp_dir.join("dlisp_test_io.txt");
    let test_file_str = test_file_path.to_str().unwrap();

    // 1. write-file
    let write_code = format!(r#"(write-file "{}" "hello dlisp")"#, test_file_str);
    let res = eval_lisp(&write_code).await.unwrap();
    assert_eq!(res, Value::Bool(true));

    // 2. file-exists? on true
    let file_exists_code = format!(r#"(file-exists? "{}")"#, test_file_str);
    let res = eval_lisp(&file_exists_code).await.unwrap();
    assert_eq!(res, Value::Bool(true));

    // 3. is-file? on true
    let is_file_code = format!(r#"(is-file? "{}")"#, test_file_str);
    let res = eval_lisp(&is_file_code).await.unwrap();
    assert_eq!(res, Value::Bool(true));

    // 4. is-dir? on file
    let is_dir_code = format!(r#"(is-dir? "{}")"#, test_file_str);
    let res = eval_lisp(&is_dir_code).await.unwrap();
    assert_eq!(res, Value::Bool(false));

    // 5. read-file
    let read_code = format!(r#"(read-file "{}")"#, test_file_str);
    let res = eval_lisp(&read_code).await.unwrap();
    assert_eq!(res, Value::String("hello dlisp".to_string()));

    // 6. delete-file
    let delete_code = format!(r#"(delete-file "{}")"#, test_file_str);
    let res = eval_lisp(&delete_code).await.unwrap();
    assert_eq!(res, Value::Bool(true));

    // 7. file-exists? after deletion
    let file_exists_code_after = format!(r#"(file-exists? "{}")"#, test_file_str);
    let res = eval_lisp(&file_exists_code_after).await.unwrap();
    assert_eq!(res, Value::Bool(false));

    // 8. list-dir on temp dir
    let temp_dir_str = temp_dir.to_str().unwrap();
    let list_dir_code = format!(r#"(list-dir "{}")"#, temp_dir_str);
    let res = eval_lisp(&list_dir_code).await.unwrap();
    match res {
        Value::List(l) => {
            assert!(!l.is_empty());
        }
        _ => panic!("list-dir did not return a list"),
    }
}
