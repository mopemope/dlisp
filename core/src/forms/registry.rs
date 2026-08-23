use crate::ast::Value;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use futures::future::LocalBoxFuture;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Result of evaluating a special form: `Some(value)` or `None` (no value).
pub type FormResult = Result<Option<Value>, String>;

/// Boxed future returned by [`SpecialForm::call`].
pub type FormFuture<'a> = LocalBoxFuture<'a, FormResult>;

/// A special form: syntax evaluated with delayed/unevaluated arguments.
///
/// Implementations receive the raw `args` and decide which subexpressions to
/// evaluate, so they can implement binding, control flow, and quoting.
pub trait SpecialForm {
    fn call<'a>(
        &self,
        interpreter: &'a mut Interpreter,
        args: &'a [Value],
        env: &'a mut Rc<RefCell<Environment>>,
    ) -> FormFuture<'a>;
}

/// Name-to-form table consulted by the evaluator before function application.
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

    /// Registers `form` under `name` (the symbol used in source code).
    pub fn register<F>(&mut self, name: &str, form: F)
    where
        F: SpecialForm + 'static,
    {
        self.map.insert(name.to_string(), Rc::new(form));
    }

    /// Looks up the special form bound to `name`, if any.
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

/// Special form: `(defun name (params...) body...)` defines a named function.
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

/// Special form: `(if cond then else?)` evaluates only the taken branch.
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

/// Special form: `(let ((x v)...) body...)` with parallel bindings.
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

/// Special form: `(let* ((x v)...) body...)` with sequential bindings.
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

/// Special form: `(throw value)` raises `value` as an error.
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

/// Special form: `(spawn closure)` runs a closure on a background task.
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

/// Special form: `(lambda (params...) body...)` creates a closure.
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

/// Special form: `(defvar name value)` defines a global variable.
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

/// Special form: `(quote x)` returns `x` without evaluating it.
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

/// Special form: `(setq name value)` assigns to an existing binding.
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

/// Special form: `(progn body...)` / `(do body...)` evaluates sequentially and returns the last value.
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

/// Special form: `(cond (test expr)...)` evaluates the first true branch.
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

/// Special form: `(and x...)` short-circuiting conjunction.
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

/// Special form: `(or x...)` short-circuiting disjunction.
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

/// Special form: `(map f coll)` applies `f` to each element.
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

/// Special form: `(filter pred coll)` keeps truthy results.
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

/// Special form: `(reduce f init coll)` folds left with accumulator.
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

/// Special form: `(some f coll)` returns the first truthy result of `f`.
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

/// Special form: `(every f coll)` true if `f` is truthy for all elements.
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

/// Special form: `(find f coll)` first element for which `f` is truthy.
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

/// Special form: `(for-each f coll)` applies `f` for side effects; returns nil.
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

/// Special form: `(map-indexed f coll)` applies `f` to `(index elem)` pairs.
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

/// Special form: `(eval expr)` parses and evaluates at runtime.
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

/// Special form: `(apply f arg... arglist)` calls `f` with args plus the spread final list/vector.
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

/// Special form: `(while test body...)` loops until `test` is false.
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

/// Special form: `(when test body...)` evaluates `body` only if `test` is truthy.
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

/// Special form: `(unless test body...)` evaluates `body` only if `test` is nil/false.
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

/// Special form: `(dotimes (i n) body...)` iterates `i` from 0 to n-1.
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

/// Special form: `(dolist (x coll) body...)` iterates over collection elements.
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

/// Special form: `(macroexpand expr)` expands macros one level.
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

/// Special form: `(load path)` loads and evaluates a source file.
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

/// Special form: `(require module)` loads a module or bundled stdlib once.
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

/// Special form: `(update map key f extra...)` updates the value at `key` with `(f old-value extra...)`.
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

/// Special form: `(map-keys f map)` transforms every key.
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

/// Special form: `(map-vals f map)` transforms every value.
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

/// Builds the default special form registry used by `Interpreter::new`.
///
/// The set of names registered here is the authoritative language surface for
/// special forms; language docs must stay in sync with it.
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
