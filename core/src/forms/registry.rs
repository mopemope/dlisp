use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use futures::future::LocalBoxFuture;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub trait SpecialForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>>;
}

#[derive(Clone)]
pub struct FormRegistry {
    map: HashMap<String, Rc<dyn SpecialForm>>,
}

impl FormRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, name: &str, form: F)
    where
        F: SpecialForm + 'static,
    {
        self.map.insert(name.to_string(), Rc::new(form));
    }

    pub fn get(&self, name: &str) -> Option<Rc<dyn SpecialForm>> {
        self.map.get(name).cloned()
    }
}

impl Default for FormRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// --- Standard Forms ---

pub struct DefunForm;
impl SpecialForm for DefunForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        // defun is synchronous, so we execute it immediately and return the result
        let res = crate::forms::defun::defun(&mut interpreter.jit, args, env);
        Box::pin(async move { res })
    }
}

pub struct IfForm;
impl SpecialForm for IfForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::if_expr::if_form(interpreter, args, env))
    }
}

pub struct LetForm;
impl SpecialForm for LetForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::let_expr::let_form(interpreter, args, env))
    }
}

pub struct SpawnForm;
impl SpecialForm for SpawnForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::spawn::spawn(interpreter, args, env))
    }
}

pub struct LambdaForm;
impl SpecialForm for LambdaForm {
    fn call<'a>(
        &self,
        _interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        let res = crate::forms::lambda::lambda(args, env);
        Box::pin(async move { res })
    }
}

pub struct DefVarForm;
impl SpecialForm for DefVarForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::defvar::defvar(interpreter, args, env))
    }
}

pub struct QuoteForm;
impl SpecialForm for QuoteForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::quote::QuoteForm.call(interpreter, args, env))
    }
}

pub fn standard_registry() -> FormRegistry {
    let mut reg = FormRegistry::new();
    reg.register("defun", DefunForm);
    reg.register("if", IfForm);
    reg.register("let", LetForm);
    reg.register("spawn", SpawnForm);
    reg.register("lambda", LambdaForm);
    reg.register("defvar", DefVarForm);
    reg.register("quote", QuoteForm);
    reg
}
