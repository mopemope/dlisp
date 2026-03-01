use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

pub async fn spawn(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, String> {
    if args.is_empty() {
        return Err("spawn requires a function or function call".to_string());
    }

    let func_val = interpreter.eval(args[0].clone(), env).await?;
    let mut eval_args = Vec::with_capacity(args.len().saturating_sub(1));
    for arg in &args[1..] {
        eval_args.push(interpreter.eval(arg.clone(), env).await?);
    }

    let env_clone = env.clone();

    tokio::task::spawn_local(async move {
        let mut interpreter = Interpreter::new();
        // apply expects the arguments and env to be passed seamlessly.
        if let Err(e) = interpreter
            .apply(func_val, eval_args, &mut env_clone.clone())
            .await
        {
            eprintln!("Spawned task error: {}", e);
        }
    });

    Ok(Some(Value::Nil))
}
