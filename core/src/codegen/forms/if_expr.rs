use crate::ast::Value;
use crate::codegen::context::FunctionTranslationContext;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_module::Module;

pub fn compile_if<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() < 3 {
        return Err("if requires condition and then-branch".to_string());
    }

    let cond_val = ctx.compile_expr(&list[1])?;

    // Check truthiness
    let local_truthy = ctx
        .module
        .declare_func_in_func(ctx.builtins.dlisp_is_truthy, ctx.builder.func);
    let truthy_call = ctx.builder.ins().call(local_truthy, &[cond_val]);
    let truthy_res = ctx.builder.inst_results(truthy_call)[0];

    let then_block = ctx.builder.create_block();
    let else_block = ctx.builder.create_block();
    let merge_block = ctx.builder.create_block();

    // Use stack slot to pass result effectively acting as Phi
    let slot =
        ctx.builder
            .create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 3)); // 8 bytes for ptr_type, 2^3 align

    // Branch
    ctx.builder
        .ins()
        .brif(truthy_res, then_block, &[], else_block, &[]);

    // Then Block
    ctx.builder.switch_to_block(then_block);
    ctx.builder.seal_block(then_block);
    let then_val = ctx.compile_expr(&list[2])?;
    ctx.builder.ins().stack_store(then_val, slot, 0);
    ctx.builder.ins().jump(merge_block, &[]);

    // Else Block
    ctx.builder.switch_to_block(else_block);
    ctx.builder.seal_block(else_block);
    let else_val = if list.len() > 3 {
        ctx.compile_expr(&list[3])?
    } else {
        ctx.builder.ins().iconst(ctx.ptr_type, 0)
    };
    ctx.builder.ins().stack_store(else_val, slot, 0);
    ctx.builder.ins().jump(merge_block, &[]);

    // Merge Block
    ctx.builder.switch_to_block(merge_block);
    ctx.builder.seal_block(merge_block);

    Ok(ctx.builder.ins().stack_load(ctx.ptr_type, slot, 0))
}
