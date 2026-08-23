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
        "first" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_car, false),
        "cdr" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_cdr, false),
        "rest" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_cdr, false),
        "cons" => compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_cons),
        "list" => compile_list_builtin(ctx, list),
        "not" => compile_not(ctx, list),
        "+" => compile_variadic_arithmetic(ctx, op, list, ctx.builtins.funcs.dlisp_add),
        "-" => compile_variadic_arithmetic(ctx, op, list, ctx.builtins.funcs.dlisp_sub),
        "*" => compile_variadic_arithmetic(ctx, op, list, ctx.builtins.funcs.dlisp_mul),
        ">" => compile_binary_comparison(ctx, op, list, ctx.builtins.funcs.dlisp_gt),
        "<" => compile_binary_comparison(ctx, op, list, ctx.builtins.funcs.dlisp_lt),
        "=" => compile_binary_comparison(ctx, op, list, ctx.builtins.funcs.dlisp_eq),
        // HOF Phase
        "map" => compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_map),
        "filter" => compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_filter),
        "reduce" => compile_ternary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_reduce),
        "some" => compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_some),
        "every" => compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_every),
        "find" => compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_find),
        "for-each" => compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_for_each),
        // Vector operations
        "vector" => compile_vector_literal(ctx, list),
        "count" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_vector_count, false),
        "nth" => compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_vector_get),
        "conj" => compile_conj_builtin(ctx, list),
        "hash-map" => compile_hash_map(ctx, list),
        "assoc" => compile_assoc(ctx, list),
        "get" => compile_get(ctx, list),
        // Phase 2
        "/" => compile_variadic_arithmetic(ctx, op, list, ctx.builtins.funcs.dlisp_div),
        "%" => compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_mod),
        "mod" => compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_mod),
        ">=" => compile_binary_comparison(ctx, op, list, ctx.builtins.funcs.dlisp_gte),
        "<=" => compile_binary_comparison(ctx, op, list, ctx.builtins.funcs.dlisp_lte),
        "/=" => compile_binary_comparison(ctx, op, list, ctx.builtins.funcs.dlisp_neq),
        "str" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_str, false),
        "string-length" => {
            compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_string_length, false)
        }
        "substring" => compile_substring(ctx, list), // Special case for 3 args
        "string-append" => {
            compile_variadic_arithmetic(ctx, op, list, ctx.builtins.funcs.dlisp_string_append)
        } // Can be variadic? No, runtime is binary. Use binary for now.
        // Actually string-append in runtime is binary. If I use variadic arithmetic logic it loops.
        // But variadic arithmetic logic assumes the function takes 2 args (acc, next).
        // dlisp_string_append takes 2 args. So compile_variadic_arithmetic SHOULD work for string-append too!
        // predicates
        "nil?" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_nil_p, false),
        "list?" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_list_p, false),
        "number?" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_number_p, false),
        "string?" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_string_p, false),
        "symbol?" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_symbol_p, false),
        "keyword?" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_keyword_p, false),
        "map?" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_map_p, false),
        "vector?" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_vector_p, false),
        "type-of" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_type_of, false),
        // Phase 3 additions
        "file-exists?" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_file_exists, false),
        "is-dir?" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_is_dir, false),
        "is-file?" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_is_file, false),
        "list-dir" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_list_dir, false),
        "delete-file" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_delete_file, false),
        "getenv" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_getenv, false),
        "setenv" => compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_setenv),
        "cwd" => compile_nullary(ctx, op, list, ctx.builtins.funcs.dlisp_cwd),
        "set-cwd" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_set_cwd, false),
        "args" => compile_nullary(ctx, op, list, ctx.builtins.funcs.dlisp_args),
        "exit" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_exit, false),
        "sh" => compile_variadic_list_call(ctx, op, list, ctx.builtins.funcs.dlisp_sh),
        // Compiled list helpers
        "reverse" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_reverse, false),
        "last" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_last, false),
        "butlast" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_butlast, false),
        "flatten" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_flatten, false),
        "empty?" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_is_empty, false),
        "take" => compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_take),
        "drop" => compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_drop),
        "append" => compile_append_variadic(ctx, list),
        // Compiled string helpers
        "string-split" => {
            compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_string_split)
        }
        "string-replace" => {
            compile_ternary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_string_replace)
        }
        "string-upper" => {
            compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_string_upper, false)
        }
        "string-lower" => {
            compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_string_lower, false)
        }
        "string-trim" => compile_unary(ctx, op, list, ctx.builtins.funcs.dlisp_string_trim, false),
        "string-trim-left" => compile_unary(
            ctx,
            op,
            list,
            ctx.builtins.funcs.dlisp_string_trim_left,
            false,
        ),
        "string-trim-right" => compile_unary(
            ctx,
            op,
            list,
            ctx.builtins.funcs.dlisp_string_trim_right,
            false,
        ),
        "string-starts-with?" => {
            compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_string_starts_with)
        }
        "string-ends-with?" => {
            compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_string_ends_with)
        }
        "string-contains?" => {
            compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_string_contains)
        }
        "string-index-of" => {
            compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_string_index_of)
        }
        "string->number" => compile_unary(
            ctx,
            op,
            list,
            ctx.builtins.funcs.dlisp_string_to_number,
            false,
        ),
        "number->string" => compile_unary(
            ctx,
            op,
            list,
            ctx.builtins.funcs.dlisp_number_to_string,
            false,
        ),
        "char-at" => compile_binary_builtin(ctx, op, list, ctx.builtins.funcs.dlisp_char_at),
        _ => unreachable!("Unknown builtin: {}", op),
    }
}

fn compile_list_builtin<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    let mut current = ctx.make_nil()?;
    let cons_func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_make_cons, ctx.builder.func);

    for arg in list[1..].iter().rev() {
        let arg_val = ctx.compile_expr(arg)?;
        let call = ctx.builder.ins().call(cons_func, &[arg_val, current]);
        current = ctx.builder.inst_results(call)[0];
    }

    Ok(current)
}

fn compile_not<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() != 2 {
        return Err("not requires exactly 1 argument".to_string());
    }

    let arg_val = ctx.compile_expr(&list[1])?;
    let truthy_func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_is_truthy, ctx.builder.func);
    let truthy_call = ctx.builder.ins().call(truthy_func, &[arg_val]);
    let truthy = ctx.builder.inst_results(truthy_call)[0];
    let is_falsy = ctx.builder.ins().icmp_imm(IntCC::Equal, truthy, 0);
    let one = ctx.builder.ins().iconst(types::I8, 1);
    let zero = ctx.builder.ins().iconst(types::I8, 0);
    let bool_int = ctx.builder.ins().select(is_falsy, one, zero);

    let make_bool = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_make_bool, ctx.builder.func);
    let call = ctx.builder.ins().call(make_bool, &[bool_int]);
    Ok(ctx.builder.inst_results(call)[0])
}

fn compile_variadic_list_call<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    _op: &str,
    list: &[Value],
    func_id: FuncId,
) -> Result<IrValue, String> {
    if list.len() < 2 {
        return Err(format!("{} requires at least 1 argument", _op));
    }
    // evaluate all arguments and form a Lisp list
    // (cons arg1 (cons arg2 ... nil))
    let local_nil_func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_make_nil, ctx.builder.func);
    let nil_call = ctx.builder.ins().call(local_nil_func, &[]);
    let mut current_list = ctx.builder.inst_results(nil_call)[0];

    let local_cons_func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_make_cons, ctx.builder.func);

    // we must build the list backwards
    for arg in list[1..].iter().rev() {
        let arg_val = ctx.compile_expr(arg)?;
        let cons_call = ctx
            .builder
            .ins()
            .call(local_cons_func, &[arg_val, current_list]);
        current_list = ctx.builder.inst_results(cons_call)[0];
    }

    let local_func = ctx.module.declare_func_in_func(func_id, ctx.builder.func);
    let call = ctx.builder.ins().call(local_func, &[current_list]);
    Ok(ctx.builder.inst_results(call)[0])
}

fn compile_nullary<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    op: &str,
    list: &[Value],
    func_id: FuncId,
) -> Result<IrValue, String> {
    if list.len() != 1 {
        return Err(format!("{} requires exactly 0 arguments", op));
    }
    let local_func = ctx.module.declare_func_in_func(func_id, ctx.builder.func);
    let call = ctx.builder.ins().call(local_func, &[]);
    Ok(ctx.builder.inst_results(call)[0])
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

/// `(append arg...)` concatenates every argument (list/vector/nil) into one
/// list by folding the binary `dlisp_append` runtime helper.
fn compile_append_variadic<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() < 2 {
        return Err("append requires at least 1 argument".to_string());
    }

    let mut acc = ctx.compile_expr(&list[1])?;
    for arg in &list[2..] {
        let next_val = ctx.compile_expr(arg)?;
        let local_func = ctx
            .module
            .declare_func_in_func(ctx.builtins.funcs.dlisp_append, ctx.builder.func);
        let call = ctx.builder.ins().call(local_func, &[acc, next_val]);
        acc = ctx.builder.inst_results(call)[0];
    }
    Ok(acc)
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

fn compile_ternary_builtin<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    op: &str,
    list: &[Value],
    func_id: FuncId,
) -> Result<IrValue, String> {
    if list.len() != 4 {
        return Err(format!("{} requires exactly 3 arguments", op));
    }
    let arg1 = ctx.compile_expr(&list[1])?;
    let arg2 = ctx.compile_expr(&list[2])?;
    let arg3 = ctx.compile_expr(&list[3])?;
    let local_func = ctx.module.declare_func_in_func(func_id, ctx.builder.func);
    let call = ctx.builder.ins().call(local_func, &[arg1, arg2, arg3]);
    Ok(ctx.builder.inst_results(call)[0])
}

fn compile_vector_literal<M: Module>(
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

fn compile_conj_builtin<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() < 3 {
        return Err("conj requires at least collection and one item".to_string());
    }
    let mut col_val = ctx.compile_expr(&list[1])?;
    let conj_func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_conj, ctx.builder.func);

    for item in &list[2..] {
        let item_val = ctx.compile_expr(item)?;
        let call = ctx.builder.ins().call(conj_func, &[col_val, item_val]);
        col_val = ctx.builder.inst_results(call)[0];
    }

    Ok(col_val)
}

fn compile_substring<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() != 4 {
        return Err("substring requires exactly 3 arguments".to_string());
    }
    let s_val = ctx.compile_expr(&list[1])?;
    let start_val = ctx.compile_expr(&list[2])?;
    let end_val = ctx.compile_expr(&list[3])?;

    let local_func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_substring, ctx.builder.func);
    let call = ctx
        .builder
        .ins()
        .call(local_func, &[s_val, start_val, end_val]);
    Ok(ctx.builder.inst_results(call)[0])
}

fn compile_hash_map<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if !(list.len() - 1).is_multiple_of(2) {
        return Err("hash-map requires an even number of arguments".to_string());
    }

    let local_make_map = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_make_map, ctx.builder.func);
    let make_map_call = ctx.builder.ins().call(local_make_map, &[]);
    let mut map_ptr = ctx.builder.inst_results(make_map_call)[0];

    let mut i = 1;
    while i < list.len() {
        let key_val = ctx.compile_expr(&list[i])?;
        let val_val = ctx.compile_expr(&list[i + 1])?;

        let local_assoc = ctx
            .module
            .declare_func_in_func(ctx.builtins.funcs.dlisp_map_assoc, ctx.builder.func);
        let assoc_call = ctx
            .builder
            .ins()
            .call(local_assoc, &[map_ptr, key_val, val_val]);
        map_ptr = ctx.builder.inst_results(assoc_call)[0];
        i += 2;
    }

    Ok(map_ptr)
}

fn compile_assoc<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() != 4 {
        return Err("assoc requires 3 arguments (map key value)".to_string());
    }
    let map_val = ctx.compile_expr(&list[1])?;
    let key_val = ctx.compile_expr(&list[2])?;
    let val_val = ctx.compile_expr(&list[3])?;

    let func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_map_assoc, ctx.builder.func);
    let call = ctx.builder.ins().call(func, &[map_val, key_val, val_val]);
    Ok(ctx.builder.inst_results(call)[0])
}

fn compile_get<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() < 3 || list.len() > 4 {
        return Err("get requires 2 or 3 arguments (map key [default])".to_string());
    }
    let col_val = ctx.compile_expr(&list[1])?;
    let key_val = ctx.compile_expr(&list[2])?;
    let default_val = if list.len() == 4 {
        ctx.compile_expr(&list[3])?
    } else {
        ctx.make_nil()?
    };

    let func = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_get, ctx.builder.func);
    let call = ctx
        .builder
        .ins()
        .call(func, &[col_val, key_val, default_val]);
    Ok(ctx.builder.inst_results(call)[0])
}
