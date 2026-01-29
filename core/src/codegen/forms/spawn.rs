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

    // We need to wrap the body in a lambda: (lambda () body)
    // list[1] is the body.
    let body = list[1].clone();

    // Construct (lambda () body)
    let lambda_sym = Value::Symbol("lambda".to_string());
    let args_list = Value::List(vec![]); // Empty args
    let synthetic_lambda = vec![lambda_sym, args_list, body];

    // Compile the lambda to get a closure_ptr
    let closure_ptr = crate::codegen::forms::lambda::compile_lambda(ctx, &synthetic_lambda)?;

    // Call dlisp_spawn(closure_ptr)
    let local_spawn = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.dlisp_spawn, ctx.builder.func);

    ctx.builder.ins().call(local_spawn, &[closure_ptr]);

    // spawn returns nil (0)
    Ok(ctx.builder.ins().iconst(ctx.ptr_type, 0))
}
