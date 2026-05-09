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

pub struct LetStarForm;
impl SpecialForm for LetStarForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::let_star::let_star_form(
            interpreter,
            args,
            env,
        ))
    }
}

pub struct ThrowForm;
impl SpecialForm for ThrowForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::throw::throw_form(interpreter, args, env))
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

pub struct SetQForm;
impl SpecialForm for SetQForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::setq::setq(interpreter, args, env))
    }
}

pub struct PrognForm;
impl SpecialForm for PrognForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::progn::progn(interpreter, args, env))
    }
}

pub struct CondForm;
impl SpecialForm for CondForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::cond::cond(interpreter, args, env))
    }
}

pub struct AndForm;
impl SpecialForm for AndForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::and_or::and_form(interpreter, args, env))
    }
}

pub struct OrForm;
impl SpecialForm for OrForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::and_or::or_form(interpreter, args, env))
    }
}

pub struct MapForm;
impl SpecialForm for MapForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::higher_order::map_form(interpreter, args, env))
    }
}

pub struct FilterForm;
impl SpecialForm for FilterForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::higher_order::filter_form(
            interpreter,
            args,
            env,
        ))
    }
}

pub struct ReduceForm;
impl SpecialForm for ReduceForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::higher_order::reduce_form(
            interpreter,
            args,
            env,
        ))
    }
}

pub struct SomeForm;
impl SpecialForm for SomeForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::higher_order::some_form(
            interpreter,
            args,
            env,
        ))
    }
}

pub struct EveryForm;
impl SpecialForm for EveryForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::higher_order::every_form(
            interpreter,
            args,
            env,
        ))
    }
}

pub struct FindForm;
impl SpecialForm for FindForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::higher_order::find_form(
            interpreter,
            args,
            env,
        ))
    }
}

pub struct ForEachForm;
impl SpecialForm for ForEachForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::higher_order::for_each_form(
            interpreter,
            args,
            env,
        ))
    }
}

pub struct MapIndexedForm;
impl SpecialForm for MapIndexedForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::higher_order::map_indexed_form(
            interpreter,
            args,
            env,
        ))
    }
}

pub struct EvalForm;
impl SpecialForm for EvalForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::eval::eval_form(interpreter, args, env))
    }
}

pub struct ApplyForm;
impl SpecialForm for ApplyForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::apply::apply_form(interpreter, args, env))
    }
}

pub struct WhileForm;
impl SpecialForm for WhileForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::while_loop::while_form(interpreter, args, env))
    }
}

pub struct WhenForm;
impl SpecialForm for WhenForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::when_unless::when_form(interpreter, args, env))
    }
}

pub struct UnlessForm;
impl SpecialForm for UnlessForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::when_unless::unless_form(
            interpreter,
            args,
            env,
        ))
    }
}

pub struct DotimesForm;
impl SpecialForm for DotimesForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::dotimes::dotimes_form(interpreter, args, env))
    }
}

pub struct DolistForm;
impl SpecialForm for DolistForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::dolist::dolist_form(interpreter, args, env))
    }
}

pub struct MacroExpandForm;
impl SpecialForm for MacroExpandForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::macroexpand::macroexpand_form(
            interpreter,
            args,
            env,
        ))
    }
}

pub struct LoadForm;
impl SpecialForm for LoadForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::load::load_form(interpreter, args, env))
    }
}

pub struct RequireForm;
impl SpecialForm for RequireForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::require::require_form(interpreter, args, env))
    }
}

pub struct UpdateForm;
impl SpecialForm for UpdateForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::higher_order::update_form(
            interpreter,
            args,
            env,
        ))
    }
}

pub struct MapKeysForm;
impl SpecialForm for MapKeysForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::higher_order::map_keys_form(
            interpreter,
            args,
            env,
        ))
    }
}

pub struct MapValsForm;
impl SpecialForm for MapValsForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> LocalBoxFuture<'a, Result<Option<Value>, String>> {
        Box::pin(crate::forms::higher_order::map_vals_form(
            interpreter,
            args,
            env,
        ))
    }
}

use crate::forms::try_catch::TryCatchForm;

pub fn standard_registry() -> FormRegistry {
    let mut reg = FormRegistry::new();
    reg.register("defun", DefunForm);
    reg.register("if", IfForm);
    reg.register("let", LetForm);
    reg.register("let*", LetStarForm);
    reg.register("spawn", SpawnForm);
    reg.register("lambda", LambdaForm);
    reg.register("defvar", DefVarForm);
    reg.register("quote", QuoteForm);
    reg.register("setq", SetQForm);
    reg.register("progn", PrognForm);
    reg.register("do", PrognForm);
    reg.register("cond", CondForm);
    reg.register("and", AndForm);
    reg.register("or", OrForm);
    reg.register("map", MapForm);
    reg.register("filter", FilterForm);
    reg.register("reduce", ReduceForm);
    reg.register("some", SomeForm);
    reg.register("every", EveryForm);
    reg.register("find", FindForm);
    reg.register("for-each", ForEachForm);
    reg.register("map-indexed", MapIndexedForm);
    reg.register("update", UpdateForm);
    reg.register("map-keys", MapKeysForm);
    reg.register("map-vals", MapValsForm);
    reg.register("eval", EvalForm);
    reg.register("apply", ApplyForm);
    reg.register("while", WhileForm);
    reg.register("when", WhenForm);
    reg.register("unless", UnlessForm);
    reg.register("dotimes", DotimesForm);
    reg.register("dolist", DolistForm);
    reg.register("macroexpand", MacroExpandForm);
    reg.register("load", LoadForm);
    reg.register("require", RequireForm);
    reg.register("try", TryCatchForm);
    reg.register("throw", ThrowForm);
    reg
}
