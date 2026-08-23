use crate::ast::Value;
use crate::codegen::context::{FunctionTranslationContext, LoopFrame};
use cranelift::codegen::ir::StackSlot;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_module::Module;
use std::collections::HashMap;

fn make_bool<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    value: bool,
) -> Result<IrValue, String> {
    let raw = ctx
        .builder
        .ins()
        .iconst(types::I8, if value { 1 } else { 0 });
    let func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_make_bool, ctx.builder.func);
    let call = ctx.builder.ins().call(func, &[raw]);
    Ok(ctx.builder.inst_results(call)[0])
}

/// Calls a `(DlispValue*) -> DlispValue*` runtime helper.
fn call_unary<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    func_id: cranelift_module::FuncId,
    arg: IrValue,
) -> Result<IrValue, String> {
    let func = ctx.module.declare_func_in_func(func_id, ctx.builder.func);
    let call = ctx.builder.ins().call(func, &[arg]);
    Ok(ctx.builder.inst_results(call)[0])
}

/// Calls a `(DlispValue*, DlispValue*) -> DlispValue*` runtime helper.
fn call_binary<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    func_id: cranelift_module::FuncId,
    arg0: IrValue,
    arg1: IrValue,
) -> Result<IrValue, String> {
    let func = ctx.module.declare_func_in_func(func_id, ctx.builder.func);
    let call = ctx.builder.ins().call(func, &[arg0, arg1]);
    Ok(ctx.builder.inst_results(call)[0])
}

fn make_int<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    value: i64,
) -> Result<IrValue, String> {
    let raw = ctx.builder.ins().iconst(types::I64, value);
    call_unary(ctx, ctx.builtins.funcs.dlisp_make_int, raw)
}

fn compile_truthy<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    val: IrValue,
) -> Result<IrValue, String> {
    let func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_is_truthy, ctx.builder.func);
    let call = ctx.builder.ins().call(func, &[val]);
    Ok(ctx.builder.inst_results(call)[0])
}

fn result_slot<M: Module>(ctx: &mut FunctionTranslationContext<M>) -> StackSlot {
    ctx.builder
        .create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 3))
}

/// True when the current block already ends in a terminator instruction.
pub(crate) fn block_terminated<M: Module>(ctx: &FunctionTranslationContext<M>) -> bool {
    let Some(block) = ctx.builder.current_block() else {
        return false;
    };
    let Some(last) = ctx.builder.func.layout.last_inst(block) else {
        return false;
    };
    ctx.builder.func.dfg.insts[last].opcode().is_terminator()
}

/// After compiling an expression that may have terminated the current block
/// (e.g. an unconditional `recur`), reopen a fresh block so the caller can
/// keep emitting instructions. Anything emitted there is dead code, matching
/// the interpreter's abort-on-recur semantics.
pub(crate) fn ensure_open_block<M: Module>(ctx: &mut FunctionTranslationContext<M>) {
    if block_terminated(ctx) {
        let dead = ctx.builder.create_block();
        ctx.builder.switch_to_block(dead);
        // No predecessor will ever be added to this block.
        ctx.builder.seal_block(dead);
    }
}

fn compile_body<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    body: &[Value],
) -> Result<IrValue, String> {
    let mut result = ctx.make_nil()?;
    for expr in body {
        result = ctx.compile_expr(expr)?;
        ensure_open_block(ctx);
    }
    Ok(result)
}

pub fn compile_progn<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    compile_body(ctx, &list[1..])
}

pub fn compile_when<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
    unless: bool,
) -> Result<IrValue, String> {
    if list.len() < 2 {
        return Err(if unless {
            "unless requires at least a condition".to_string()
        } else {
            "when requires at least a condition".to_string()
        });
    }

    let cond_val = ctx.compile_expr(&list[1])?;
    ensure_open_block(ctx);
    let truthy = compile_truthy(ctx, cond_val)?;

    let body_block = ctx.builder.create_block();
    let nil_block = ctx.builder.create_block();
    let merge_block = ctx.builder.create_block();
    let slot = result_slot(ctx);

    if unless {
        ctx.builder
            .ins()
            .brif(truthy, nil_block, &[], body_block, &[]);
    } else {
        ctx.builder
            .ins()
            .brif(truthy, body_block, &[], nil_block, &[]);
    }

    ctx.builder.switch_to_block(body_block);
    ctx.builder.seal_block(body_block);
    let body_val = compile_body(ctx, &list[2..])?;
    ctx.builder.ins().stack_store(body_val, slot, 0);
    ctx.builder.ins().jump(merge_block, &[]);

    ctx.builder.switch_to_block(nil_block);
    ctx.builder.seal_block(nil_block);
    let nil_val = ctx.make_nil()?;
    ctx.builder.ins().stack_store(nil_val, slot, 0);
    ctx.builder.ins().jump(merge_block, &[]);

    ctx.builder.switch_to_block(merge_block);
    ctx.builder.seal_block(merge_block);
    Ok(ctx.builder.ins().stack_load(ctx.ptr_type, slot, 0))
}

pub fn compile_and<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    let args = &list[1..];
    if args.is_empty() {
        return make_bool(ctx, true);
    }

    let slot = result_slot(ctx);
    let end_block = ctx.builder.create_block();

    for (idx, expr) in args.iter().enumerate() {
        let val = ctx.compile_expr(expr)?;
        ensure_open_block(ctx);
        ctx.builder.ins().stack_store(val, slot, 0);

        if idx + 1 == args.len() {
            ctx.builder.ins().jump(end_block, &[]);
            break;
        }

        let truthy = compile_truthy(ctx, val)?;
        let next_block = ctx.builder.create_block();
        ctx.builder
            .ins()
            .brif(truthy, next_block, &[], end_block, &[]);
        ctx.builder.switch_to_block(next_block);
        ctx.builder.seal_block(next_block);
    }

    ctx.builder.switch_to_block(end_block);
    ctx.builder.seal_block(end_block);
    Ok(ctx.builder.ins().stack_load(ctx.ptr_type, slot, 0))
}

pub fn compile_or<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    let args = &list[1..];
    if args.is_empty() {
        return ctx.make_nil();
    }

    let slot = result_slot(ctx);
    let end_block = ctx.builder.create_block();

    for (idx, expr) in args.iter().enumerate() {
        let val = ctx.compile_expr(expr)?;
        ensure_open_block(ctx);
        ctx.builder.ins().stack_store(val, slot, 0);

        if idx + 1 == args.len() {
            ctx.builder.ins().jump(end_block, &[]);
            break;
        }

        let truthy = compile_truthy(ctx, val)?;
        let next_block = ctx.builder.create_block();
        ctx.builder
            .ins()
            .brif(truthy, end_block, &[], next_block, &[]);
        ctx.builder.switch_to_block(next_block);
        ctx.builder.seal_block(next_block);
    }

    ctx.builder.switch_to_block(end_block);
    ctx.builder.seal_block(end_block);
    Ok(ctx.builder.ins().stack_load(ctx.ptr_type, slot, 0))
}

pub fn compile_cond<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    let clauses = &list[1..];
    if clauses.is_empty() {
        return ctx.make_nil();
    }

    let slot = result_slot(ctx);
    let end_block = ctx.builder.create_block();

    for clause in clauses {
        let Value::List(items) = clause else {
            return Err("cond clause must be a list".to_string());
        };
        if items.is_empty() {
            return Err("cond clause must be a non-empty list".to_string());
        }

        let test_val = ctx.compile_expr(&items[0])?;
        ensure_open_block(ctx);
        let truthy = compile_truthy(ctx, test_val)?;
        let body_block = ctx.builder.create_block();
        let next_block = ctx.builder.create_block();
        ctx.builder
            .ins()
            .brif(truthy, body_block, &[], next_block, &[]);

        ctx.builder.switch_to_block(body_block);
        ctx.builder.seal_block(body_block);
        let result = if items.len() == 1 {
            test_val
        } else {
            compile_body(ctx, &items[1..])?
        };
        ctx.builder.ins().stack_store(result, slot, 0);
        ctx.builder.ins().jump(end_block, &[]);

        ctx.builder.switch_to_block(next_block);
        ctx.builder.seal_block(next_block);
    }

    let nil_val = ctx.make_nil()?;
    ctx.builder.ins().stack_store(nil_val, slot, 0);
    ctx.builder.ins().jump(end_block, &[]);

    ctx.builder.switch_to_block(end_block);
    ctx.builder.seal_block(end_block);
    Ok(ctx.builder.ins().stack_load(ctx.ptr_type, slot, 0))
}

pub fn compile_while<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() < 2 {
        return Err("while requires at least a condition".to_string());
    }

    let header_block = ctx.builder.create_block();
    let body_block = ctx.builder.create_block();
    let end_block = ctx.builder.create_block();

    // The loop evaluates to the last body value of the final iteration,
    // or nil when the condition is false immediately.
    let slot = result_slot(ctx);
    let nil_val = ctx.make_nil()?;
    ctx.builder.ins().stack_store(nil_val, slot, 0);
    ctx.builder.ins().jump(header_block, &[]);

    ctx.builder.switch_to_block(header_block);
    let cond_val = ctx.compile_expr(&list[1])?;
    ensure_open_block(ctx);
    let truthy = compile_truthy(ctx, cond_val)?;
    ctx.builder
        .ins()
        .brif(truthy, body_block, &[], end_block, &[]);

    ctx.builder.switch_to_block(body_block);
    ctx.builder.seal_block(body_block);
    for expr in &list[2..] {
        let val = ctx.compile_expr(expr)?;
        ensure_open_block(ctx);
        ctx.builder.ins().stack_store(val, slot, 0);
    }
    ctx.builder.ins().jump(header_block, &[]);

    ctx.builder.switch_to_block(end_block);
    ctx.builder.seal_block(end_block);
    ctx.builder.seal_block(header_block);
    Ok(ctx.builder.ins().stack_load(ctx.ptr_type, slot, 0))
}

pub fn compile_dotimes<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() < 2 {
        return Err("dotimes requires at least a binding form (var count)".to_string());
    }
    let Value::List(binding) = &list[1] else {
        return Err("dotimes first argument must be (var count)".to_string());
    };
    if binding.len() != 2 {
        return Err("dotimes binding must be (var count)".to_string());
    }
    let Value::Symbol(var_name) = &binding[0] else {
        return Err("dotimes variable must be a symbol".to_string());
    };

    // The count is evaluated once, before the loop starts.
    let count_val = ctx.compile_expr(&binding[1])?;
    let zero_val = make_int(ctx, 0)?;
    let one_val = make_int(ctx, 1)?;

    let counter = ctx.builder.declare_var(ctx.ptr_type);
    ctx.builder.def_var(counter, zero_val);

    ctx.scopes.push(HashMap::new());
    ctx.scopes
        .last_mut()
        .expect("dotimes scope exists")
        .insert(var_name.clone(), counter);

    let header_block = ctx.builder.create_block();
    let body_block = ctx.builder.create_block();
    let end_block = ctx.builder.create_block();
    ctx.builder.ins().jump(header_block, &[]);

    ctx.builder.switch_to_block(header_block);
    let i_val = ctx.builder.use_var(counter);
    let cmp_val = call_binary(ctx, ctx.builtins.funcs.dlisp_lt, i_val, count_val)?;
    let truthy = compile_truthy(ctx, cmp_val)?;
    ctx.builder
        .ins()
        .brif(truthy, body_block, &[], end_block, &[]);

    ctx.builder.switch_to_block(body_block);
    ctx.builder.seal_block(body_block);
    for expr in &list[2..] {
        ctx.compile_expr(expr)?;
        ensure_open_block(ctx);
    }
    let cur_val = ctx.builder.use_var(counter);
    let next_val = call_binary(ctx, ctx.builtins.funcs.dlisp_add, cur_val, one_val)?;
    ctx.builder.def_var(counter, next_val);
    ctx.builder.ins().jump(header_block, &[]);

    ctx.builder.switch_to_block(end_block);
    ctx.builder.seal_block(end_block);
    ctx.builder.seal_block(header_block);

    ctx.scopes.pop();
    ctx.make_nil()
}

pub fn compile_dolist<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() < 2 {
        return Err("dolist requires at least a binding form (var coll)".to_string());
    }
    let Value::List(binding) = &list[1] else {
        return Err("dolist first argument must be (var coll)".to_string());
    };
    if binding.len() != 2 {
        return Err("dolist binding must be (var coll)".to_string());
    }
    let Value::Symbol(var_name) = &binding[0] else {
        return Err("dolist variable must be a symbol".to_string());
    };

    // The collection is evaluated once; vectors are normalized to lists
    // so a single cons walk handles lists, vectors, and nil.
    let coll_val = ctx.compile_expr(&binding[1])?;
    let vec_check = call_unary(ctx, ctx.builtins.funcs.dlisp_vector_p, coll_val)?;
    let is_vector = compile_truthy(ctx, vec_check)?;

    let conv_block = ctx.builder.create_block();
    let keep_block = ctx.builder.create_block();
    let merge_block = ctx.builder.create_block();
    let norm_slot = result_slot(ctx);

    ctx.builder
        .ins()
        .brif(is_vector, conv_block, &[], keep_block, &[]);

    ctx.builder.switch_to_block(conv_block);
    ctx.builder.seal_block(conv_block);
    let as_list = call_unary(ctx, ctx.builtins.funcs.dlisp_vector_to_list, coll_val)?;
    ctx.builder.ins().stack_store(as_list, norm_slot, 0);
    ctx.builder.ins().jump(merge_block, &[]);

    ctx.builder.switch_to_block(keep_block);
    ctx.builder.seal_block(keep_block);
    ctx.builder.ins().stack_store(coll_val, norm_slot, 0);
    ctx.builder.ins().jump(merge_block, &[]);

    ctx.builder.switch_to_block(merge_block);
    ctx.builder.seal_block(merge_block);
    let iter_val = ctx.builder.ins().stack_load(ctx.ptr_type, norm_slot, 0);

    let current = ctx.builder.declare_var(ctx.ptr_type);
    ctx.builder.def_var(current, iter_val);

    let elem = ctx.builder.declare_var(ctx.ptr_type);

    ctx.scopes.push(HashMap::new());
    ctx.scopes
        .last_mut()
        .expect("dolist scope exists")
        .insert(var_name.clone(), elem);

    let header_block = ctx.builder.create_block();
    let body_block = ctx.builder.create_block();
    let end_block = ctx.builder.create_block();
    ctx.builder.ins().jump(header_block, &[]);

    ctx.builder.switch_to_block(header_block);
    let cur_val = ctx.builder.use_var(current);
    let truthy = compile_truthy(ctx, cur_val)?;
    ctx.builder
        .ins()
        .brif(truthy, body_block, &[], end_block, &[]);

    ctx.builder.switch_to_block(body_block);
    ctx.builder.seal_block(body_block);
    let car_val = call_unary(ctx, ctx.builtins.funcs.dlisp_car, cur_val)?;
    ctx.builder.def_var(elem, car_val);
    for expr in &list[2..] {
        ctx.compile_expr(expr)?;
        ensure_open_block(ctx);
    }
    let walked_val = call_unary(ctx, ctx.builtins.funcs.dlisp_cdr, cur_val)?;
    ctx.builder.def_var(current, walked_val);
    ctx.builder.ins().jump(header_block, &[]);

    ctx.builder.switch_to_block(end_block);
    ctx.builder.seal_block(end_block);
    ctx.builder.seal_block(header_block);

    ctx.scopes.pop();
    ctx.make_nil()
}

/// `(loop [name init ...] body...)`: iteration with `recur` support.
///
/// Bindings become Cranelift variables in a fresh scope; a `recur` re-defines
/// them with freshly evaluated values and jumps back to the body header.
pub fn compile_loop<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() < 2 {
        return Err("loop requires bindings and a body".to_string());
    }
    let items: &[Value] = match &list[1] {
        Value::List(items) => items,
        Value::Vector(items) => items,
        _ => return Err("loop bindings must be a list or vector of name/init pairs".to_string()),
    };
    if !items.len().is_multiple_of(2) {
        return Err("loop bindings must contain an even number of name/init forms".to_string());
    }
    let mut names = Vec::with_capacity(items.len() / 2);
    let mut inits = Vec::with_capacity(items.len() / 2);
    for pair in items.chunks(2) {
        let Value::Symbol(name) = &pair[0] else {
            return Err("loop binding name must be a symbol".to_string());
        };
        names.push(name.clone());
        inits.push(&pair[1]);
    }

    ctx.scopes.push(HashMap::new());
    let mut vars = Vec::with_capacity(names.len());
    for (name, init) in names.iter().zip(inits.iter()) {
        let val = ctx.compile_expr(init)?;
        let var = ctx.builder.declare_var(ctx.ptr_type);
        ctx.builder.def_var(var, val);
        ctx.scopes
            .last_mut()
            .expect("loop scope exists")
            .insert(name.clone(), var);
        vars.push(var);
    }

    // The body block doubles as the loop header; `recur` jumps straight to it.
    let body_block = ctx.builder.create_block();
    let end_block = ctx.builder.create_block();
    ctx.builder.ins().jump(body_block, &[]);
    ctx.builder.switch_to_block(body_block);

    let slot = result_slot(ctx);
    let nil_val = ctx.make_nil()?;
    ctx.builder.ins().stack_store(nil_val, slot, 0);

    ctx.loop_frames.push(LoopFrame {
        vars,
        header_block: body_block,
    });
    for expr in &list[2..] {
        if block_terminated(ctx) {
            // A previous expression ended this iteration (unconditional
            // recur); the rest of the body is unreachable by construction,
            // matching the interpreter's abort-on-recur semantics.
            break;
        }
        let val = ctx.compile_expr(expr)?;
        if !block_terminated(ctx) {
            ctx.builder.ins().stack_store(val, slot, 0);
        }
    }
    ctx.loop_frames.pop();
    ctx.scopes.pop();

    if !block_terminated(ctx) {
        ctx.builder.ins().jump(end_block, &[]);
    }

    ctx.builder.switch_to_block(end_block);
    ctx.builder.seal_block(body_block);
    ctx.builder.seal_block(end_block);
    Ok(ctx.builder.ins().stack_load(ctx.ptr_type, slot, 0))
}

/// `(recur expr ...)`: rebind the innermost loop's variables and iterate.
///
/// All new values are evaluated into temporaries before any binding is
/// overwritten, so `(recur (- n 1) (* acc n))` sees the old bindings.
pub fn compile_recur<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    let Some(frame) = ctx.loop_frames.last().map(|f| LoopFrame {
        vars: f.vars.clone(),
        header_block: f.header_block,
    }) else {
        return Err("recur outside loop".to_string());
    };
    let args = &list[1..];
    if args.len() != frame.vars.len() {
        return Err(format!(
            "recur expects {} argument(s), got {}",
            frame.vars.len(),
            args.len()
        ));
    }

    let mut tmps = Vec::with_capacity(args.len());
    for arg in args {
        let val = ctx.compile_expr(arg)?;
        let tmp = ctx.builder.declare_var(ctx.ptr_type);
        ctx.builder.def_var(tmp, val);
        tmps.push(tmp);
    }
    let nil_val = ctx.make_nil()?;
    for (var, tmp) in frame.vars.iter().zip(tmps.iter()) {
        let val = ctx.builder.use_var(*tmp);
        ctx.builder.def_var(*var, val);
    }
    ctx.builder.ins().jump(frame.header_block, &[]);
    Ok(nil_val)
}
