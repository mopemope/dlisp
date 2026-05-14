use dlisp_core::compiler::AOTCompiler;
use dlisp_core::parser::parse;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "dlisp_compiler_macros_{}_{}_{}",
        name,
        std::process::id(),
        unique
    ))
}

#[tokio::test]
async fn test_aot_macro_compilation() {
    // Defines a macro 'when' and uses it in 'main'.
    // The AOT compiler must execute the macro at compile time.
    let code = "
    (defmacro when (cond body)
        (list 'if cond body 'nil))

    (defun main ()
        (when (> 2 1) (print 42)))
    ";

    let vals = parse(code).unwrap();
    let compiler = AOTCompiler::new();
    let result = compiler.compile(vals).await;

    // We just check if compilation succeeds. Executing the binary is harder in a unit test
    // without spinning up a subprocess, but succesful compilation implies the macro was expanded
    // (otherwise 'when' would be treated as an undefined function call or fail generation).
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
}

#[tokio::test]
async fn test_aot_require_core_macro_compilation() {
    let code = r#"
    (require "core")

    (defun main ()
        (print (when-let (x 41) (inc x))))
    "#;

    let vals = parse(code).unwrap();
    let compiler = AOTCompiler::new();
    let result = compiler.compile(vals).await;

    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
}

#[tokio::test]
async fn test_aot_file_require_macro_can_call_required_helper() {
    let dir = temp_dir("file_require_helper");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("macros.lisp"),
        r#"
        (defun macro-add-one (x)
          (list '+ x 1))

        (defmacro plus-one (x)
          (macro-add-one x))
        "#,
    )
    .unwrap();

    let code = r#"
    (require "./macros.lisp")

    (defun main ()
        (print (plus-one 6)))
    "#;

    let vals = parse(code).unwrap();
    let compiler = AOTCompiler::new();
    let result = compiler.compile_with_base_dir(vals, &dir).await;

    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());

    let _ = fs::remove_dir_all(&dir);
}
