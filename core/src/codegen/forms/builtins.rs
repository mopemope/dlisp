use crate::ast::Value;
use crate::codegen::context::FunctionTranslationContext;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_module::Module;

use cranelift_module::FuncId;

enum BuiltinCategory {
    Unary { returns_arg: bool },
    BinaryComparison,
    VariadicArithmetic,
    IO,
}

pub fn compile_builtin<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    op: &str,
    list: &[Value],
) -> Result<IrValue, String> {
    let (category, func_id) = match op {
        "sleep" => (
            BuiltinCategory::Unary { returns_arg: false },
            ctx.builtins.funcs.dlisp_sleep,
        ),
        "print" => (
            BuiltinCategory::Unary { returns_arg: true },
            ctx.builtins.funcs.dlisp_print,
        ),
        "car" => (
            BuiltinCategory::Unary { returns_arg: false },
            ctx.builtins.funcs.dlisp_car,
        ),
        "cdr" => (
            BuiltinCategory::Unary { returns_arg: false },
            ctx.builtins.funcs.dlisp_cdr,
        ),
        "+" => (
            BuiltinCategory::VariadicArithmetic,
            ctx.builtins.funcs.dlisp_add,
        ),
        "-" => (
            BuiltinCategory::VariadicArithmetic,
            ctx.builtins.funcs.dlisp_sub,
        ),
        "*" => (
            BuiltinCategory::VariadicArithmetic,
            ctx.builtins.funcs.dlisp_mul,
        ),
        ">" => (
            BuiltinCategory::BinaryComparison,
            ctx.builtins.funcs.dlisp_gt,
        ),
        "<" => (
            BuiltinCategory::BinaryComparison,
            ctx.builtins.funcs.dlisp_lt,
        ),
        "=" => (
            BuiltinCategory::BinaryComparison,
            ctx.builtins.funcs.dlisp_eq,
        ),
        "read-file" => (BuiltinCategory::IO, ctx.builtins.funcs.dlisp_read_file),
        _ => unreachable!("Unknown builtin: {}", op),
    };

    match category {
        BuiltinCategory::Unary { returns_arg } => {
            compile_unary(ctx, op, list, func_id, returns_arg)
        }
        BuiltinCategory::BinaryComparison => compile_binary_comparison(ctx, op, list, func_id),
        BuiltinCategory::VariadicArithmetic => compile_variadic_arithmetic(ctx, op, list, func_id),
        BuiltinCategory::IO => compile_io(ctx, op, list, func_id),
    }
}

fn compile_unary<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    op: &str,
    list: &[Value],
    func_id: FuncId,
    returns_arg: bool,
) -> Result<IrValue, String> {
    if list.len() != 2 {
        return Err(format!("{} requires exactly 1 argument", op));
    }
    let arg_val = ctx.compile_expr(&list[1])?;
    let local_func = ctx.module.declare_func_in_func(func_id, ctx.builder.func);
    let call = ctx.builder.ins().call(local_func, &[arg_val]);

    if returns_arg {
        Ok(arg_val)
    } else {
        Ok(ctx.builder.inst_results(call)[0])
    }
}

fn compile_binary_comparison<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    op: &str,
    list: &[Value],
    func_id: FuncId,
) -> Result<IrValue, String> {
    if list.len() != 3 {
        return Err(format!("{} requires exactly 2 arguments", op));
    }
    let lhs = ctx.compile_expr(&list[1])?;
    let rhs = ctx.compile_expr(&list[2])?;
    let local_func = ctx.module.declare_func_in_func(func_id, ctx.builder.func);
    let call = ctx.builder.ins().call(local_func, &[lhs, rhs]);
    Ok(ctx.builder.inst_results(call)[0])
}

fn compile_variadic_arithmetic<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    op: &str,
    list: &[Value],
    func_id: FuncId,
) -> Result<IrValue, String> {
    if list.len() < 2 {
        return Err(format!("{} requires at least 1 argument", op));
    }

    if list.len() == 2 {
        let val = ctx.compile_expr(&list[1])?;
        if op == "-" {
            // Negate: sub(0, val)
            let zero = ctx.builder.ins().iconst(types::I64, 0);
            let make_int = ctx
                .module
                .declare_func_in_func(ctx.builtins.funcs.dlisp_make_int, ctx.builder.func);
            let zero_obj = ctx.builder.ins().call(make_int, &[zero]);
            let zero_obj_val = ctx.builder.inst_results(zero_obj)[0];

            let local_sub = ctx
                .module
                .declare_func_in_func(ctx.builtins.funcs.dlisp_sub, ctx.builder.func);
            let call = ctx.builder.ins().call(local_sub, &[zero_obj_val, val]);
            Ok(ctx.builder.inst_results(call)[0])
        } else {
            Ok(val)
        }
    } else {
        let mut acc = ctx.compile_expr(&list[1])?;
        for arg in &list[2..] {
            let next_val = ctx.compile_expr(arg)?;
            let local_func = ctx.module.declare_func_in_func(func_id, ctx.builder.func);
            let call = ctx.builder.ins().call(local_func, &[acc, next_val]);
            acc = ctx.builder.inst_results(call)[0];
        }
        Ok(acc)
    }
}

fn compile_io<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    op: &str,
    list: &[Value],
    func_id: FuncId,
) -> Result<IrValue, String> {
    // Basic IO compilation is similar to Unary for now, assuming (op arg) signature
    // Future IO might take multiple args
    if list.len() != 2 {
        return Err(format!("{} requires exactly 1 argument", op));
    }
    let arg_val = ctx.compile_expr(&list[1])?;
    let local_func = ctx.module.declare_func_in_func(func_id, ctx.builder.func);
    let call = ctx.builder.ins().call(local_func, &[arg_val]);
    Ok(ctx.builder.inst_results(call)[0])
}
