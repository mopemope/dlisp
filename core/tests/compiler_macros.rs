use dlisp_core::compiler::AOTCompiler;
use dlisp_core::parser::parse;

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
