use dlisp_core::ast::Value;
use dlisp_core::interpreter::{Interpreter, default_env};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "dlisp_load_{}_{}_{}",
        name,
        std::process::id(),
        unique
    ))
}

#[tokio::test]
async fn test_load_defines_vars_and_funcs() {
    let dir = std::env::temp_dir().join("dlisp_test_load");
    let _ = fs::create_dir_all(&dir);
    let test_file = dir.join("test_load.lisp");
    fs::write(
        &test_file,
        "(defvar *test-var* 42)\n(defun test-func (x) (+ x 1))",
    )
    .unwrap();

    let mut env = default_env();
    let mut interpreter = Interpreter::new();

    // Load the file
    let code = format!("(load \"{}\")", test_file.display());
    let parsed = dlisp_core::parser::parse(&code).unwrap();
    for stmt in parsed {
        interpreter.eval(stmt, &mut env).await.unwrap();
    }

    // Verify defvar
    let parsed = dlisp_core::parser::parse("*test-var*").unwrap();
    let var_val = interpreter.eval(parsed[0].clone(), &mut env).await.unwrap();
    assert_eq!(var_val, Value::Integer(42));

    // Verify defun
    let parsed = dlisp_core::parser::parse("(test-func 10)").unwrap();
    let func_val = interpreter.eval(parsed[0].clone(), &mut env).await.unwrap();
    assert_eq!(func_val, Value::Integer(11));

    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_dir(&dir);
}

#[tokio::test]
async fn test_load_nonexistent_file() {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();
    let code = "(load \"/tmp/nonexistent_dlisp_file_12345.lisp\")";
    let parsed = dlisp_core::parser::parse(code).unwrap();
    let result = interpreter.eval(parsed[0].clone(), &mut env).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to read file")
    );
}

#[tokio::test]
async fn test_load_invalid_lisp() {
    let dir = std::env::temp_dir().join("dlisp_test_load_bad");
    let _ = fs::create_dir_all(&dir);
    let test_file = dir.join("bad.lisp");
    fs::write(&test_file, "(defvar *x* ").unwrap(); // unclosed paren

    let mut env = default_env();
    let mut interpreter = Interpreter::new();
    let code = format!("(load \"{}\")", test_file.display());
    let parsed = dlisp_core::parser::parse(&code).unwrap();
    let result = interpreter.eval(parsed[0].clone(), &mut env).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Parse error"));

    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_dir(&dir);
}

#[tokio::test]
async fn test_load_requires_string_arg() {
    let mut env = default_env();
    let mut interpreter = Interpreter::new();
    // Pass a number instead of a string
    let code = "(load 42)";
    let parsed = dlisp_core::parser::parse(code).unwrap();
    let result = interpreter.eval(parsed[0].clone(), &mut env).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("string file path"));
}

#[tokio::test]
async fn test_load_nested_relative_path_uses_loaded_file_directory() {
    let dir = temp_dir("relative");
    let nested = dir.join("nested");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("value.lisp"), "(defvar loaded-value 77)").unwrap();
    let main_file = dir.join("main.lisp");
    fs::write(&main_file, r#"(load "./nested/value.lisp")"#).unwrap();

    let mut env = default_env();
    let mut interpreter = Interpreter::new();
    let code = format!(r#"(load "{}")"#, main_file.display());
    let parsed = dlisp_core::parser::parse(&code).unwrap();
    for stmt in parsed {
        interpreter.eval(stmt, &mut env).await.unwrap();
    }

    let parsed = dlisp_core::parser::parse("loaded-value").unwrap();
    let var_val = interpreter.eval(parsed[0].clone(), &mut env).await.unwrap();
    assert_eq!(var_val, Value::Integer(77));

    let _ = fs::remove_dir_all(&dir);
}
