use crate::ast::Value;
use crate::codegen::context::FunctionTranslationContext;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_module::Module;

pub fn compile_builtin<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    op: &str,
    list: &[Value],
) -> Result<IrValue, String> {
    match op {
        "sleep" => {
            if list.len() != 2 {
                return Err("sleep requires 1 arg (ms)".to_string());
            }
            let ms_val = ctx.compile_expr(&list[1])?;

            let local_sleep = ctx
                .module
                .declare_func_in_func(ctx.builtins.funcs.dlisp_sleep, ctx.builder.func);

            let call = ctx.builder.ins().call(local_sleep, &[ms_val]);
            Ok(ctx.builder.inst_results(call)[0])
        }
        "print" => {
            if list.len() != 2 {
                return Err("print takes 1 arg".to_string());
            }
            let arg_val = ctx.compile_expr(&list[1])?;

            let local_print = ctx
                .module
                .declare_func_in_func(ctx.builtins.funcs.dlisp_print, ctx.builder.func);

            ctx.builder.ins().call(local_print, &[arg_val]);
            Ok(arg_val)
        }
        "car" => {
            if list.len() != 2 {
                return Err("car requires 1 arg".to_string());
            }
            let val = ctx.compile_expr(&list[1])?;
            let func = ctx
                .module
                .declare_func_in_func(ctx.builtins.funcs.dlisp_car, ctx.builder.func);
            let call = ctx.builder.ins().call(func, &[val]);
            Ok(ctx.builder.inst_results(call)[0])
        }
        "cdr" => {
            if list.len() != 2 {
                return Err("cdr requires 1 arg".to_string());
            }
            let val = ctx.compile_expr(&list[1])?;
            let func = ctx
                .module
                .declare_func_in_func(ctx.builtins.funcs.dlisp_cdr, ctx.builder.func);
            let call = ctx.builder.ins().call(func, &[val]);
            Ok(ctx.builder.inst_results(call)[0])
        }
        "+" | "-" | "*" => {
            if list.len() < 2 {
                // TODO: Handle 0 args (+ -> 0, * -> 1). For now require at least 1.
                return Err(format!("{} requires at least 1 argument", op));
            }

            // Unary case handling or identity
            // If len == 2 (one arg):
            //   (+) -> identity
            //   (*) -> identity
            //   (-) -> negate (0 - arg)
            if list.len() == 2 {
                let val = ctx.compile_expr(&list[1])?;
                if op == "-" {
                    // Negate: sub(0, val)
                    // We need to generate a 0 int constant? or does the runtime handle mixed types?
                    // Runtime dlisp_sub handles Int/Float. We generally want to preserve type?
                    // For simplicity, let's assume Int 0 for negation for now, assuming standard Lisp behavior?
                    // BUT: strict typing might be an issue.
                    // Safer: compile `0` as dlisp_make_int(0).
                    let zero = ctx.builder.ins().iconst(types::I64, 0);
                    let zero_val_func = ctx
                        .module
                        .declare_func_in_func(ctx.builtins.funcs.dlisp_make_int, ctx.builder.func);
                    let zero_obj = ctx.builder.ins().call(zero_val_func, &[zero]);
                    let zero_obj_val = ctx.builder.inst_results(zero_obj)[0];

                    let local_func = ctx
                        .module
                        .declare_func_in_func(ctx.builtins.funcs.dlisp_sub, ctx.builder.func);
                    let call = ctx.builder.ins().call(local_func, &[zero_obj_val, val]);
                    Ok(ctx.builder.inst_results(call)[0])
                } else {
                    Ok(val)
                }
            } else {
                // Variadic: (> 1 arg)
                // acc = first
                let mut acc = ctx.compile_expr(&list[1])?;

                for arg in &list[2..] {
                    let next_val = ctx.compile_expr(arg)?;
                    let func_id = match op {
                        "+" => ctx.builtins.funcs.dlisp_add,
                        "-" => ctx.builtins.funcs.dlisp_sub,
                        "*" => ctx.builtins.funcs.dlisp_mul,
                        _ => unreachable!(),
                    };
                    let local_func = ctx.module.declare_func_in_func(func_id, ctx.builder.func);
                    let call = ctx.builder.ins().call(local_func, &[acc, next_val]);
                    acc = ctx.builder.inst_results(call)[0];
                }
                Ok(acc)
            }
        }
        ">" | "<" | "=" => {
            if list.len() == 3 {
                let lhs = ctx.compile_expr(&list[1])?;
                let rhs = ctx.compile_expr(&list[2])?;

                let func_id = match op {
                    ">" => ctx.builtins.funcs.dlisp_gt,
                    "<" => ctx.builtins.funcs.dlisp_lt,
                    "=" => ctx.builtins.funcs.dlisp_eq,
                    _ => unreachable!(),
                };

                let local_func = ctx.module.declare_func_in_func(func_id, ctx.builder.func);
                let call = ctx.builder.ins().call(local_func, &[lhs, rhs]);
                Ok(ctx.builder.inst_results(call)[0])
            } else {
                Err(format!("Binary comparison ops require 2 args: {}", op))
            }
        }
        _ => unreachable!(),
    }
}
