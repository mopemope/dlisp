use crate::ast::Value;
use crate::codegen::LAMBDA_COUNTER;
use crate::codegen::context::{Builtins, FunctionTranslationContext};
use cranelift::prelude::{Value as IrValue, *};

use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{Linkage, Module};
use std::collections::HashMap;
use std::sync::atomic::Ordering;

pub fn compile_lambda<M: Module>(
    ctx: &mut FunctionTranslationContext<M>,
    list: &[Value],
) -> Result<IrValue, String> {
    if list.len() < 3 {
        return Err("lambda requires args and body".to_string());
    }

    let args_val = &list[1];
    let body = &list[2..];

    let params = match args_val {
        Value::List(l) => l,
        _ => return Err("lambda args must be a list".to_string()),
    };

    let mut arg_names = Vec::new();
    for param in params {
        match param {
            Value::Symbol(s) => arg_names.push(s.clone()),
            _ => return Err("lambda param must be a symbol".to_string()),
        }
    }

    // 1. Analyze Free Variables
    let free_vars = analyze_free_variables(body, &arg_names);

    // 2. Allocate Environment
    let ptr_size = 8;
    let env_size = (free_vars.len() * ptr_size) as i64;

    // Malloc env if size > 0
    let env_ptr_val = if env_size > 0 {
        let size_val = ctx.builder.ins().iconst(ctx.ptr_type, env_size);
        let local_malloc = ctx
            .module
            .declare_func_in_func(ctx.builtins.funcs.gc_malloc, ctx.builder.func);
        let call = ctx.builder.ins().call(local_malloc, &[size_val]);
        ctx.builder.inst_results(call)[0]
    } else {
        ctx.builder.ins().iconst(ctx.ptr_type, 0) // NULL
    };

    // 3. Populate Environment
    let mut captured_offsets = HashMap::new();
    for (i, var_name) in free_vars.iter().enumerate() {
        let val = ctx.resolve_variable(var_name)?;
        let offset = (i * ptr_size) as i32;
        ctx.builder
            .ins()
            .store(MemFlags::new(), val, env_ptr_val, offset);
        captured_offsets.insert(var_name.clone(), offset as u32);
    }

    // 4. Compile Lambda Function
    let lambda_name = format!("lambda_{}", LAMBDA_COUNTER.fetch_add(1, Ordering::Relaxed));
    let mut inner_ctx = cranelift::codegen::Context::new();
    let mut builder_context = FunctionBuilderContext::new();
    let int = ctx.ptr_type;

    // Signature: (env, args...)
    inner_ctx.func.signature.params.push(AbiParam::new(int));
    for _ in &arg_names {
        inner_ctx.func.signature.params.push(AbiParam::new(int));
    }
    inner_ctx.func.signature.returns.push(AbiParam::new(int));

    {
        let mut builder = FunctionBuilder::new(&mut inner_ctx.func, &mut builder_context);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);

        // Import builtins
        let fmt_id = ctx
            .module
            .declare_data("printf_fmt", Linkage::Local, true, false)
            .map_err(|e| e.to_string())?;
        let printf_fmt_val = ctx.module.declare_data_in_func(fmt_id, builder.func);
        let inner_builtins = Builtins {
            funcs: ctx.builtins.funcs,
            printf_fmt: builder.ins().global_value(int, printf_fmt_val),
        };

        let env_param = builder.block_params(entry_block)[0];
        let mut initial_scope = HashMap::new();
        for (i, arg_name) in arg_names.iter().enumerate() {
            let val = builder.block_params(entry_block)[i + 1];
            let var = builder.declare_var(int);
            builder.def_var(var, val);
            initial_scope.insert(arg_name.clone(), var);
        }

        let mut result_val = builder.ins().iconst(int, 0);

        {
            let mut trans_ctx = FunctionTranslationContext {
                builder: &mut builder,
                module: ctx.module,
                builtins: &inner_builtins,
                scopes: vec![initial_scope],
                captured_vars: captured_offsets,
                env_param: Some(env_param),
                ptr_type: int,
                global_signatures: ctx.global_signatures,
            };

            for expr in body {
                result_val = trans_ctx.compile_expr(expr)?;
            }
        }

        builder.ins().return_(&[result_val]);
        builder.seal_all_blocks();
        builder.finalize();
    }

    let id = ctx
        .module
        .declare_function(&lambda_name, Linkage::Export, &inner_ctx.func.signature)
        .map_err(|e| e.to_string())?;
    ctx.module
        .define_function(id, &mut inner_ctx)
        .map_err(|e| e.to_string())?;

    let local_func = ctx.module.declare_func_in_func(id, ctx.builder.func);
    let func_addr = ctx.builder.ins().func_addr(ctx.ptr_type, local_func);

    // 5. Create Closure Struct { func_ptr, env_ptr }
    let closure_size = 16;
    let size_val = ctx.builder.ins().iconst(ctx.ptr_type, closure_size);
    let local_malloc = ctx
        .module
        .declare_func_in_func(ctx.builtins.funcs.gc_malloc, ctx.builder.func);
    let call = ctx.builder.ins().call(local_malloc, &[size_val]);
    let closure_ptr = ctx.builder.inst_results(call)[0];

    ctx.builder
        .ins()
        .store(MemFlags::new(), func_addr, closure_ptr, 0);
    ctx.builder
        .ins()
        .store(MemFlags::new(), env_ptr_val, closure_ptr, 8);

    Ok(closure_ptr)
}

fn analyze_free_variables(body: &[Value], args: &[String]) -> Vec<String> {
    let mut free_vars = Vec::new();
    let mut bound_vars = std::collections::HashSet::new();
    for arg in args {
        bound_vars.insert(arg.clone());
    }

    for expr in body {
        find_free_vars(expr, &mut bound_vars, &mut free_vars);
    }

    // Dedup
    free_vars.sort();
    free_vars.dedup();
    free_vars
}

fn find_free_vars(
    expr: &Value,
    bound: &mut std::collections::HashSet<String>,
    free: &mut Vec<String>,
) {
    match expr {
        Value::Symbol(s) => {
            if !bound.contains(s) {
                let builtins = ["+", "-", "*", "print", "sleep", "spawn", "let", "lambda"];
                if !builtins.contains(&s.as_str()) {
                    free.push(s.clone());
                }
            }
        }
        Value::List(list) => {
            if list.is_empty() {
                return;
            }
            match &list[0] {
                Value::Symbol(op) if op == "let" => {
                    // (let ((v e) ...) body)
                    if list.len() >= 3 {
                        // bindings
                        let mut new_bound = bound.clone();
                        if let Value::List(bindings) = &list[1] {
                            for b in bindings {
                                #[allow(clippy::collapsible_if)]
                                if let Value::List(pair) = b {
                                    if pair.len() == 2 {
                                        // RHS is evaluated in current scope
                                        find_free_vars(&pair[1], bound, free);
                                        // LHS binds in body
                                        if let Value::Symbol(name) = &pair[0] {
                                            new_bound.insert(name.clone());
                                        }
                                    }
                                }
                            }
                        }
                        // body
                        for sub in &list[2..] {
                            find_free_vars(sub, &mut new_bound, free);
                        }
                    }
                }
                Value::Symbol(op) if op == "lambda" => {
                    // (lambda (args) body)
                    if list.len() >= 3 {
                        let mut new_bound = bound.clone();
                        if let Value::List(args) = &list[1] {
                            for arg in args {
                                if let Value::Symbol(name) = arg {
                                    new_bound.insert(name.clone());
                                }
                            }
                        }
                        for sub in &list[2..] {
                            find_free_vars(sub, &mut new_bound, free);
                        }
                    }
                }
                _ => {
                    for sub in list {
                        find_free_vars(sub, bound, free);
                    }
                }
            }
        }
        _ => {}
    }
}
