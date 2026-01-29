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
            // Since our values are pointers/integers (i64), we can treat it as u64 ms?
            // Assuming compile_expr returns I64.

            let local_sleep = ctx
                .module
                .declare_func_in_func(ctx.builtins.dlisp_sleep, ctx.builder.func);

            ctx.builder.ins().call(local_sleep, &[ms_val]);
            Ok(ctx.builder.ins().iconst(ctx.ptr_type, 0))
        }
        "print" => {
            if list.len() != 2 {
                return Err("print takes 1 arg".to_string());
            }
            let arg_val = ctx.compile_expr(&list[1])?;

            let local_printf = ctx
                .module
                .declare_func_in_func(ctx.builtins.printf, ctx.builder.func);

            ctx.builder
                .ins()
                .call(local_printf, &[ctx.builtins.printf_fmt, arg_val]);
            Ok(arg_val)
        }
        "+" | "-" | "*" | ">" => {
            if list.len() == 3 {
                let lhs = ctx.compile_expr(&list[1])?;
                let rhs = ctx.compile_expr(&list[2])?;
                match op {
                    "+" => Ok(ctx.builder.ins().iadd(lhs, rhs)),
                    "-" => Ok(ctx.builder.ins().isub(lhs, rhs)),
                    "*" => Ok(ctx.builder.ins().imul(lhs, rhs)),
                    ">" => {
                        let cmp = ctx.builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs);
                        let one = ctx.builder.ins().iconst(ctx.ptr_type, 1);
                        let zero = ctx.builder.ins().iconst(ctx.ptr_type, 0);
                        Ok(ctx.builder.ins().select(cmp, one, zero))
                    }
                    _ => unreachable!(),
                }
            } else {
                Err(format!("Binary ops require 2 args: {}", op))
            }
        }
        _ => unreachable!(),
    }
}
