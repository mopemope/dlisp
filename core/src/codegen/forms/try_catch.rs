use crate::ast::Value;
use crate::codegen::context::FunctionTranslationContext;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_module::Module;
use std::collections::HashMap;

use super::control::ensure_open_block;

/// Lowers `(try body... (catch var handler-body...))`.
///
/// The catch block is registered on `ctx.try_frames` while the body is
/// compiled. A `throw` (or a user call returning the throw sentinel) inside
/// the body jumps there with the thrown value pending in thread-local state;
/// the handler takes it, wraps it in an error value, and binds `var`.
pub fn compile_try<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    // `(try)` evaluates to nil, matching the interpreter.
    if list.len() < 2 {
        return ctx.make_nil();
    }

    let mut body: &[Value] = &list[1..];
    let mut catch_var: Option<String> = None;
    let mut catch_body: &[Value] = &[];

    if let Some(Value::List(last_list)) = body.last()
        && !last_list.is_empty()
        && let Value::Symbol(s) = &last_list[0]
        && s == "catch"
    {
        if last_list.len() < 2 {
            return Err("catch requires a variable name".to_string());
        }
        let Value::Symbol(var) = &last_list[1] else {
            return Err("catch variable must be a symbol".to_string());
        };
        catch_var = Some(var.clone());
        catch_body = &last_list[2..];
        body = &body[..body.len() - 1];
    }

    let slot =
        ctx.builder
            .create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 3));

    let catch_block = ctx.builder.create_block();
    let merge_block = ctx.builder.create_block();

    ctx.try_frames.push(catch_block);

    let mut body_result = ctx.make_nil()?;
    for expr in body {
        body_result = ctx.compile_expr(expr)?;
        ensure_open_block(ctx);
    }

    ctx.try_frames.pop();

    // Normal path: body completed without throwing.
    ctx.builder.ins().stack_store(body_result, slot, 0);
    ctx.builder.ins().jump(merge_block, &[]);

    // Catch path: seal now that all predecessors (throw sites / call-site
    // escapes emitted during body compilation) are registered.
    ctx.builder.switch_to_block(catch_block);
    ctx.builder.seal_block(catch_block);

    if let Some(var) = catch_var {
        let thrown = call_take_thrown(ctx)?;
        let err_val = call_make_error(ctx, thrown)?;
        ctx.scopes.push(HashMap::new());
        let var_reg = ctx.builder.declare_var(ctx.ptr_type);
        ctx.builder.def_var(var_reg, err_val);
        ctx.scopes
            .last_mut()
            .expect("catch scope exists")
            .insert(var, var_reg);

        let mut handler_result = ctx.make_nil()?;
        for expr in catch_body {
            handler_result = ctx.compile_expr(expr)?;
            ensure_open_block(ctx);
        }

        ctx.scopes.pop();

        ctx.builder.ins().stack_store(handler_result, slot, 0);
        ctx.builder.ins().jump(merge_block, &[]);
    } else {
        // No catch clause: the throw escapes this try. Escape to the
        // enclosing try's catch block when one exists (the interpreter
        // bubbles the failure outward), otherwise return the sentinel out
        // of the current function so callers propagate it. The sentinel is
        // fetched explicitly here: the normal-path body_result does not
        // dominate this block.
        match ctx.try_frames.last().copied() {
            Some(outer_catch_block) => {
                ctx.builder.ins().jump(outer_catch_block, &[]);
            }
            None => {
                let sentinel = call_throw_sentinel(ctx)?;
                ctx.builder.ins().return_(&[sentinel]);
            }
        }
    }

    ctx.builder.switch_to_block(merge_block);
    ctx.builder.seal_block(merge_block);
    Ok(ctx.builder.ins().stack_load(ctx.ptr_type, slot, 0))
}

/// Lowers `(throw expr)`: evaluates the argument, stores it in thread-local
/// throw state, and escapes with the sentinel. Inside a `try` body the value
/// jumps to the innermost catch block; otherwise it returns out of the
/// current function so callers propagate it.
pub fn compile_throw<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() != 2 {
        return Err("throw requires exactly 1 argument".to_string());
    }

    let val = ctx.compile_expr(&list[1])?;
    ensure_open_block(ctx);

    let func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_throw, ctx.builder.func);
    let call = ctx.builder.ins().call(func, &[val]);
    let sentinel = ctx.builder.inst_results(call)[0];

    match ctx.try_frames.last().copied() {
        Some(catch_block) => {
            ctx.builder.ins().jump(catch_block, &[]);
            // The current block is terminated; the caller reopens a dead
            // block for any remaining expressions (they never run).
            Ok(sentinel)
        }
        None => {
            ctx.builder.ins().return_(&[sentinel]);
            ensure_open_block(ctx);
            Ok(sentinel)
        }
    }
}

fn call_take_thrown<M: Module>(ctx: &mut FunctionTranslationContext<M>) -> Result<IrValue, String> {
    let func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_take_thrown, ctx.builder.func);
    let call = ctx.builder.ins().call(func, &[]);
    Ok(ctx.builder.inst_results(call)[0])
}

fn call_throw_sentinel<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
) -> Result<IrValue, String> {
    let func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_throw_sentinel, ctx.builder.func);
    let call = ctx.builder.ins().call(func, &[]);
    Ok(ctx.builder.inst_results(call)[0])
}

fn call_make_error<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    inner: IrValue,
) -> Result<IrValue, String> {
    let func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_make_error, ctx.builder.func);
    let call = ctx.builder.ins().call(func, &[inner]);
    Ok(ctx.builder.inst_results(call)[0])
}
