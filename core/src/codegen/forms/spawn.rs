use crate::ast::Value;
use crate::codegen::context::FunctionTranslationContext;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_module::Module;

pub fn compile_spawn<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() < 2 {
        return Err("spawn requires a function or function call".to_string());
    }

    let closure_ptr = if list.len() == 2 {
        ctx.compile_expr(&list[1])?
    } else {
        let mut thunk_body = Vec::with_capacity(list.len() - 1);
        thunk_body.push(list[1].clone());
        thunk_body.extend_from_slice(&list[2..]);

        let thunk = Value::List(vec![
            Value::Symbol("lambda".to_string()),
            Value::List(vec![]),
            Value::List(thunk_body),
        ]);

        ctx.compile_expr(&thunk)?
    };

    // Call dlisp_spawn(closure_value_ptr)
    let local_spawn = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_spawn, ctx.builder.func);

    ctx.builder.ins().call(local_spawn, &[closure_ptr]);

    // spawn returns nil (0)
    Ok(ctx.builder.ins().iconst(ctx.ptr_type, 0))
}
