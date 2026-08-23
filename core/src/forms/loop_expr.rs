use crate::ast::Value;
use crate::environment::Environment;
use crate::eval_failure::EvalFailure;
use crate::forms::registry::{FormFuture, SpecialForm};
use crate::interpreter::Interpreter;
use std::cell::RefCell;
use std::rc::Rc;

/// `(loop [name init ...] body...)`
///
/// Evaluates `init` expressions in order, binds them in a fresh scope, and
/// evaluates the body. A `(recur expr ...)` anywhere in the body aborts the
/// current iteration, rebinds every name to the corresponding expression
/// value, and restarts the body. The loop yields the last body value of the
/// iteration that did not recur.
pub struct LoopForm;

impl SpecialForm for LoopForm {
    fn call<'a>(
        &self,
        interp: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> FormFuture<'a> {
        let args = args.to_vec();
        Box::pin(async move {
            if args.len() < 2 {
                return Err(EvalFailure::message(
                    "loop requires bindings and a body: (loop [name init ...] body...)",
                ));
            }
            let items: &[Value] = match &args[0] {
                Value::List(list) => list,
                Value::Vector(vec) => vec,
                _ => {
                    return Err(EvalFailure::message(
                        "loop bindings must be a list or vector of name/init pairs",
                    ));
                }
            };
            if !items.len().is_multiple_of(2) {
                return Err(EvalFailure::message(
                    "loop bindings must contain an even number of name/init forms",
                ));
            }

            let mut names = Vec::with_capacity(items.len() / 2);
            let mut inits = Vec::with_capacity(items.len() / 2);
            for pair in items.chunks(2) {
                let Value::Symbol(name) = &pair[0] else {
                    return Err(EvalFailure::message("loop binding name must be a symbol"));
                };
                names.push(name.clone());
                inits.push(pair[1].clone());
            }

            let mut loop_env = Environment::new(Some(env.clone()));
            for (name, init) in names.iter().zip(inits.iter()) {
                let val = interp.eval(init.clone(), env).await?;
                loop_env.set(name.clone(), val);
            }
            let mut loop_env = Rc::new(RefCell::new(loop_env));
            let body = &args[1..];

            loop {
                let mut last = Value::Nil;
                let mut recurred = false;
                for expr in body {
                    match interp.eval(expr.clone(), &mut loop_env).await {
                        Ok(val) => last = val,
                        Err(EvalFailure::Recur(new_vals)) => {
                            if new_vals.len() != names.len() {
                                return Err(EvalFailure::message(format!(
                                    "recur expects {} argument(s), got {}",
                                    names.len(),
                                    new_vals.len()
                                )));
                            }
                            {
                                let mut borrowed = loop_env.borrow_mut();
                                for (name, val) in names.iter().zip(new_vals) {
                                    borrowed.set(name.clone(), val);
                                }
                            }
                            recurred = true;
                            break;
                        }
                        Err(other) => return Err(other),
                    }
                }
                // A pass over the whole body without recur exits the loop.
                if !recurred {
                    return Ok(Some(last));
                }
            }
        })
    }
}

/// `(recur expr ...)`: abort the current iteration and rebind the nearest
/// enclosing `loop`.
pub struct RecurForm;

impl SpecialForm for RecurForm {
    fn call<'a>(
        &self,
        interp: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> FormFuture<'a> {
        let args = args.to_vec();
        Box::pin(async move {
            let mut vals = Vec::with_capacity(args.len());
            for arg in &args {
                vals.push(interp.eval(arg.clone(), env).await?);
            }
            Err(EvalFailure::Recur(vals))
        })
    }
}
