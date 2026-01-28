use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

pub async fn if_form(
    interpreter: &mut Interpreter,
    args: &[Value],
    env: &mut Rc<RefCell<Environment>>,
) -> Result<Option<Value>, String> {
    if args.len() < 2 {
        return Err("if requires at least condition and then-branch".to_string());
    }

    let cond = interpreter.eval(args[0].clone(), env).await?;
    let is_true = match cond {
        Value::Nil => false,        // nil is false
        Value::Integer(0) => false, // 0 is false? common lisp nil is false, but let's stick to nil.
        // Actually earlier code treated 0 as false in JIT? No JIT treated 0 as false for logic.
        // Let's settle: Nil is false. Everything else is true.
        // Or if we used (>) returning 0/1. The JIT (>) returns 0 or 1.
        // So Integer(0) should probably be false.
        Value::Integer(n) => n != 0,
        _ => true,
    };

    if is_true {
        Ok(Some(interpreter.eval(args[1].clone(), env).await?))
    } else if args.len() > 2 {
        Ok(Some(interpreter.eval(args[2].clone(), env).await?))
    } else {
        Ok(Some(Value::Nil))
    }
}
