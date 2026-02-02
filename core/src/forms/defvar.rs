use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use futures::future::LocalBoxFuture;
use std::cell::RefCell;
use std::rc::Rc;

pub fn defvar<'a>(
    interpreter: &'a mut Interpreter,
    args: &'a [Value],
    env: &'a mut Rc<RefCell<Environment>>,
) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
    Box::pin(async move {
        if args.is_empty() || args.len() > 3 {
            return Err(
                "defvar requires 1 to 3 arguments (symbol, [init-value, [doc-string]])".to_string(),
            );
        }

        let symbol_name = match &args[0] {
            Value::Symbol(s) => s.clone(),
            _ => return Err("defvar first argument must be a symbol".to_string()),
        };

        // Defvar normally defines globally (or at top level).
        // We need to find the root environment to define the variable if strictly following "defvar" semantics
        // for "global definition".
        // However, we must first check if it is bound *anywhere* visible (lexical check).
        let is_bound = env.borrow().get(&symbol_name).is_some();

        if !is_bound && args.len() >= 2 {
            let init_val = interpreter.eval(args[1].clone(), env).await?;

            // Find root
            let root;
            let mut current = env.clone();
            loop {
                let parent_opt = current.borrow().parent.clone();
                match parent_opt {
                    Some(p) => {
                        current = p;
                    }
                    None => {
                        root = current;
                        break;
                    }
                }
            }
            root.borrow_mut().set(symbol_name.clone(), init_val);
        }

        Ok(Some(Value::Symbol(symbol_name)))
    })
}
