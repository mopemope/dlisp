use dlisp_core::ast::Value;
use dlisp_core::interpreter::default_interpreter;
use dlisp_core::parser::parse;
use std::cell::RefCell;
use std::rc::Rc;

fn setup() -> (
    dlisp_core::interpreter::Interpreter,
    Rc<RefCell<dlisp_core::environment::Environment>>,
) {
    let interpreter = default_interpreter();
    let env = dlisp_core::interpreter::default_env();
    (interpreter, env)
}

async fn eval_str(
    src: &str,
    interpreter: &mut dlisp_core::interpreter::Interpreter,
    env: &mut Rc<RefCell<dlisp_core::environment::Environment>>,
) -> Value {
    let exprs = parse(src).unwrap();
    interpreter.eval(exprs[0].clone(), env).await.unwrap()
}

#[tokio::test]
async fn test_defvar_inside_let() {
    let (mut interpreter, mut env) = setup();

    // (defvar *global-inside* 999) inside a let
    // If defvar is global, *global-inside* should be visible outside.
    // If defvar is local (current impl), it won't be.

    let code = r#"
    (let () 
        (defvar *global-inside* 999))
    "#;

    eval_str(code, &mut interpreter, &mut env).await;

    // Try to access *global-inside*
    let exprs = parse("*global-inside*").unwrap();
    let res = interpreter.eval(exprs[0].clone(), &mut env).await;

    // If this fails, it means defvar is LOCAL, which is likely technically correct for "current env"
    // but semantically wrong for "defvar" (which is typically global).
    // Failing this assertion means we need to fix it.
    assert!(res.is_ok(), "defvar inside let should be visible globally");
    assert_eq!(res.unwrap(), Value::Integer(999));
}
