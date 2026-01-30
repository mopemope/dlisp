use crate::ast::Value;
use crate::environment::Environment;
use crate::forms::registry::SpecialForm;
use crate::interpreter::Interpreter;
use futures::future::LocalBoxFuture;
use std::cell::RefCell;
use std::rc::Rc;

pub struct QuoteForm;

impl SpecialForm for QuoteForm {
    fn call<'a>(
        &self,
        _interpreter: &'a mut Interpreter,
        args: &'a [Value],
        _env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        let args = args.to_vec();
        Box::pin(async move {
            if args.len() != 1 {
                return Err("quote requires exactly 1 argument".to_string());
            }
            Ok(Some(args[0].clone()))
        })
    }
}
