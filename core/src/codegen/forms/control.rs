use crate::ast::Value;
use crate::codegen::context::FunctionTranslationContext;
use cranelift::codegen::ir::StackSlot;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_module::Module;

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

fn compile_body<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    body: &[Value],
) -> Result<IrValue, String> {
    let mut result = ctx.make_nil()?;
    for expr in body {
        result = ctx.compile_expr(expr)?;
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
