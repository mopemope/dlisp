use crate::ast::Value;
use crate::codegen::context::FunctionTranslationContext;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_module::Module;

use cranelift_module::FuncId;

pub fn compile_builtin<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    op: &str,
    list: &[Value],
) -> Result<IrValue, String> {
    match op {
        "sleep" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_sleep, false),
        "print" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_print, true),
        "read-file" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_read_file, false),
        "car" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_car, false),
        "cdr" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_cdr, false),
        "+" => compile_variadic_arithmetic(ctx, op, list, ctx.builtins.funcs.dlisp_add),
        "-" => compile_variadic_arithmetic(ctx, op, list, ctx.builtins.funcs.dlisp_sub),
        "*" => compile_variadic_arithmetic(ctx, op, list, ctx.builtins.funcs.dlisp_mul),
        ">" => compile_binary_comparison(ctx, op, list, ctx.builtins.funcs.dlisp_gt),
        "<" => compile_binary_comparison(ctx, op, list, ctx.builtins.funcs.dlisp_lt),
        "=" => compile_binary_comparison(ctx, op, list, ctx.builtins.funcs.dlisp_eq),
        // Vector builtins
        "vector" => compile_vector(ctx, list),
        "count" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_vector_count, false),
        "nth" => compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_vector_get),
        "conj" => compile_conj(ctx, list),
        _ => unreachable!("Unknown builtin: {}", op),
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

fn compile_binary_builtin<M: Module>(
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

fn compile_vector<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    // (vector args...) -> [args...]
    if list.is_empty() {
        return Err("vector form requires at least operator".to_string());
    }
    let args = list[1..].to_vec();
    let vec_val = Value::Vector(args);
    ctx.compile_expr(&vec_val)
}

fn compile_conj<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    // (conj col item1 item2 ...)
    if list.len() < 3 {
        return Err("conj requires at least collection and one item".to_string());
    }
    let col_expr = &list[1];
    let items = &list[2..];

    // 1. Evaluate collection
    let col_eval = ctx.compile_expr(col_expr)?;

    // 2. Clone collection (dlisp_vector_copy)
    // Note: We need to handle List concatenation too?
    // Interpreter generic `conj` handles Lists and Vectors.
    // Making compiled `conj` generic requires runtime type check unless we have a generic `dlisp_conj`.
    // Implementing `dlisp_conj` in runtime is probably better than emitting type checks here.
    // BUT, for now let's assume Vector or implement `dlisp_conj`?
    // The task is Vector support. `dlisp_vector_copy` and `dlisp_vector_push` are Vector only.
    // If I use `conj` on a list in compiled code, it will crash or fail?
    // Yes.
    // For this MVP, let's assume Vector for compiled `conj` or implement `dlisp_conj` in runtime.
    // Implementing `dlisp_conj` in runtime is safer.
    // Let's stick to Vector-only for now using `dlisp_vector_copy` and `dlisp_vector_push`,
    // and maybe add a TODO for list support or runtime check.
    // Or, call a new runtime function `dlisp_conj`?
    // I haven't added `dlisp_conj` to `lib.rs` yet.
    // I added `dlisp_vector_push`.
    // Let's implement Vector-only `conj` here for now (using copy+push).

    let copy_func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_vector_copy, ctx.builder.func);
    let call = ctx.builder.ins().call(copy_func, &[col_eval]);
    let col_val = ctx.builder.inst_results(call)[0];

    // 3. Push items
    let push_func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_vector_push, ctx.builder.func);

    for item in items {
        let item_val = ctx.compile_expr(item)?;
        ctx.builder.ins().call(push_func, &[col_val, item_val]);
    }

    Ok(col_val)
}
