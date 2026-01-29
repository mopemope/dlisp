pub mod builtins;
pub mod context;
pub mod forms;

use crate::ast::Value;
use context::{Builtins, FunctionTranslationContext};
use cranelift::prelude::*;
use cranelift_module::{DataDescription, Linkage, Module};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;

pub static LAMBDA_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct CodeGen {
    /// The function builder context, which is reused across multiple
    /// function compilations.
    pub builder_context: FunctionBuilderContext,

    /// The main Cranelift context, which contains the state for compiling
    /// a single function.
    pub ctx: codegen::Context,

    /// The data context, which contains the state for defining data
    /// objects.
    pub data_ctx: DataDescription,

    pub builtins_initialized: bool,
    pub global_signatures: HashMap<String, Signature>,
}

impl Default for CodeGen {
    fn default() -> Self {
        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: codegen::Context::new(),
            data_ctx: DataDescription::new(),
            builtins_initialized: false,
            global_signatures: HashMap::new(),
        }
    }
}

impl CodeGen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compile<M: Module>(
        &mut self,
        module: &mut M,
        name: &str,
        args: &[String],
        body: &[Value],
    ) -> Result<cranelift_module::FuncId, String> {
        self.module_compile_func(module, name, args, body)
    }

    fn module_compile_func<M: Module>(
        &mut self,
        module: &mut M,
        name: &str,
        args: &[String],
        body: &[Value],
    ) -> Result<cranelift_module::FuncId, String> {
        self.module_clear_context(module);

        let int = module.target_config().pointer_type();

        self.data_ctx.define(b"%ld\n\0".to_vec().into_boxed_slice());
        let fmt_id = module
            .declare_data("printf_fmt", Linkage::Local, true, false)
            .map_err(|e| e.to_string())?;

        if !self.builtins_initialized {
            module
                .define_data(fmt_id, &self.data_ctx)
                .map_err(|e| e.to_string())?;
            self.builtins_initialized = true;
        }
        self.data_ctx.clear();

        let builtin_defs = builtins::declare_builtins(module)?;

        // Function Signature: (env, args...)
        self.ctx.func.signature.params.push(AbiParam::new(int)); // env
        for _ in args {
            self.ctx.func.signature.params.push(AbiParam::new(int));
        }
        self.ctx.func.signature.returns.push(AbiParam::new(int));

        // Store global signature early to avoid borrow check issues
        let signature = self.ctx.func.signature.clone();
        self.global_signatures.insert(name.to_string(), signature);

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let printf_fmt_val = module.declare_data_in_func(fmt_id, builder.func);
        let builtins = Builtins {
            funcs: builtin_defs,
            printf_fmt: builder.ins().global_value(int, printf_fmt_val),
        };

        // Param 0 is env, Param 1..N are args
        let env_param = builder.block_params(entry_block)[0];

        let mut initial_scope = HashMap::new();
        for (i, arg_name) in args.iter().enumerate() {
            let val = builder.block_params(entry_block)[i + 1]; // +1 for env
            let var = builder.declare_var(int);
            builder.def_var(var, val);
            initial_scope.insert(arg_name.clone(), var);
        }

        let mut result_val = builder.ins().iconst(int, 0);

        {
            let mut trans_ctx = FunctionTranslationContext {
                builder: &mut builder,
                module,
                builtins: &builtins,
                scopes: vec![initial_scope],
                captured_vars: HashMap::new(),
                env_param: Some(env_param),
                ptr_type: int,
                global_signatures: &self.global_signatures,
            };

            for expr in body {
                result_val = trans_ctx.compile_expr(expr)?;
            }
        }

        builder.ins().return_(&[result_val]);
        builder.seal_all_blocks();
        builder.finalize();

        println!(
            "Declaring function: {} with {} params",
            name,
            self.ctx.func.signature.params.len()
        );

        let id = module
            .declare_function(name, Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;

        module
            .define_function(id, &mut self.ctx)
            .map_err(|e| e.to_string())?;

        Ok(id)
    }

    pub fn compile_entry_point<M: Module>(
        &mut self,
        module: &mut M,
        user_main_name: &str,
    ) -> Result<(), String> {
        self.module_clear_context(module);

        let int = module.target_config().pointer_type();

        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(int));
        let dlisp_main_id = module
            .declare_function("dlisp_main", Linkage::Import, &sig)
            .map_err(|e| e.to_string())?;

        // User main: (env) -> int
        let mut user_sig = module.make_signature();
        user_sig.params.push(AbiParam::new(int)); // env
        user_sig.returns.push(AbiParam::new(int));
        let user_main_id = module
            .declare_function(user_main_name, Linkage::Export, &user_sig)
            .map_err(|e| e.to_string())?;

        self.ctx
            .func
            .signature
            .returns
            .push(AbiParam::new(types::I32));

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        let entry = builder.create_block();
        builder.switch_to_block(entry);

        let local_user_main = module.declare_func_in_func(user_main_id, builder.func);
        let user_main_addr = builder.ins().func_addr(int, local_user_main);

        let local_dlisp_main = module.declare_func_in_func(dlisp_main_id, builder.func);
        builder.ins().call(local_dlisp_main, &[user_main_addr]);

        let ret_val = builder.ins().iconst(types::I32, 0);
        builder.ins().return_(&[ret_val]);

        builder.seal_all_blocks();
        builder.finalize();

        let main_id = module
            .declare_function("main", Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;

        module
            .define_function(main_id, &mut self.ctx)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn module_clear_context<M: Module>(&mut self, module: &mut M) {
        module.clear_context(&mut self.ctx);
    }
}
