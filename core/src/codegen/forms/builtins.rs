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
        "+" | "-" | "*" | ">" | "<" | "=" => {
            if list.len() == 3 {
                let lhs = ctx.compile_expr(&list[1])?;
                let rhs = ctx.compile_expr(&list[2])?;

                let func_id = match op {
                    "+" => ctx.builtins.funcs.dlisp_add,
                    "-" => ctx.builtins.funcs.dlisp_sub,
                    "*" => ctx.builtins.funcs.dlisp_mul,
                    ">" => ctx.builtins.funcs.dlisp_gt,
                    "<" => ctx.builtins.funcs.dlisp_lt,
                    "=" => ctx.builtins.funcs.dlisp_eq,
                    _ => unreachable!(),
                };

                let local_func = ctx.module.declare_func_in_func(func_id, ctx.builder.func);
                let call = ctx.builder.ins().call(local_func, &[lhs, rhs]);
                Ok(ctx.builder.inst_results(call)[0])
            } else {
                Err(format!("Binary ops require 2 args: {}", op))
            }
        }
        _ => unreachable!(),
    }
}
