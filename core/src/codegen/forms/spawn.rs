use crate::ast::Value;
use crate::codegen::context::FunctionTranslationContext;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_module::Module;

pub fn compile_spawn<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() != 2 {
        return Err("spawn requires exactly one argument (function call)".to_string());
    }

    // Compile the argument to get a closure_ptr (or function logic)
    // Interpreter expects a function and runs it.
    // So we evaluate the argument, which should yield a Closure* (via resolve_variable or lambda).
    let closure_ptr = ctx.compile_expr(&list[1])?;

    // Call dlisp_spawn(closure_ptr)
    let local_spawn = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_spawn, ctx.builder.func);

    ctx.builder.ins().call(local_spawn, &[closure_ptr]);

    // spawn returns nil (0)
    Ok(ctx.builder.ins().iconst(ctx.ptr_type, 0))
}
