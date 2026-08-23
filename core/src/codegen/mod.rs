pub mod builtins;
pub mod context;
pub mod forms;

use crate::ast::Value;
use context::{Builtins, FunctionTranslationContext};
use cranelift::prelude::*;
use cranelift_module::{DataDescription, DataId, Linkage, Module};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;

pub static LAMBDA_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Builtin function names lowered directly by codegen (see
/// `forms::builtins::compile_builtin`).
///
/// This is the single source of truth for what codegen supports. Any builtin
/// outside this list is interpreter-only: JIT gating rejects functions that
/// call it (they keep running on the interpreter), and AOT compilation
/// reports an explicit error instead of emitting an unresolvable import.
pub const COMPILED_BUILTINS: &[&str] = &[
    // arithmetic / comparison
    "+",
    "-",
    "*",
    "/",
    "%",
    "mod",
    ">",
    "<",
    "=",
    ">=",
    "<=",
    "/=",
    // values / collections
    "print",
    "not",
    "list",
    "cons",
    "car",
    "first",
    "cdr",
    "rest",
    "vector",
    "nth",
    "count",
    "conj",
    "hash-map",
    "assoc",
    "get",
    // strings / types
    "str",
    "string-length",
    "substring",
    "string-append",
    "nil?",
    "list?",
    "number?",
    "string?",
    "symbol?",
    "keyword?",
    "vector?",
    "map?",
    "type-of",
    // higher-order
    "map",
    "filter",
    "reduce",
    "some",
    "every",
    "find",
    "for-each",
    // list helpers
    "append",
    "reverse",
    "last",
    "butlast",
    "flatten",
    "take",
    "drop",
    "empty?",
    // string helpers
    "string-split",
    "string-replace",
    "string-upper",
    "string-lower",
    "string-trim",
    "string-trim-left",
    "string-trim-right",
    "string-starts-with?",
    "string-ends-with?",
    "string-contains?",
    "string-index-of",
    "string->number",
    "number->string",
    "char-at",
    // io / sys / os
    "read-file",
    "file-exists?",
    "is-dir?",
    "is-file?",
    "list-dir",
    "delete-file",
    "getenv",
    "setenv",
    "cwd",
    "set-cwd",
    "args",
    "exit",
    "sh",
    "sleep",
];

#[derive(Clone)]
pub struct FunctionMetadata {
    pub fixed_args: Vec<String>,
    pub rest_param: Option<String>,
    pub signature: Signature,
}

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
    pub global_functions: HashMap<String, FunctionMetadata>,
    pub global_variables: HashMap<String, DataId>,
}

impl Default for CodeGen {
    fn default() -> Self {
        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: codegen::Context::new(),
            data_ctx: DataDescription::new(),
            builtins_initialized: false,
            global_functions: HashMap::new(),
            global_variables: HashMap::new(),
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
        self.module_compile_func(module, name, args, None, body)
    }

    pub fn compile_with_rest<M: Module>(
        &mut self,
        module: &mut M,
        name: &str,
        args: &[String],
        rest_param: Option<String>,
        body: &[Value],
    ) -> Result<cranelift_module::FuncId, String> {
        self.module_compile_func(module, name, args, rest_param, body)
    }

    pub fn declare_global_variable<M: Module>(
        &mut self,
        module: &mut M,
        name: &str,
    ) -> Result<(), String> {
        if self.global_variables.contains_key(name) {
            return Ok(());
        }

        let data_id = module
            .declare_data(
                &format!("dlisp_global_{}", name),
                Linkage::Local,
                true,
                false,
            )
            .map_err(|e| e.to_string())?;

        let ptr_size = module.target_config().pointer_bytes() as usize;
        let mut data_ctx = DataDescription::new();
        data_ctx.define(vec![0; ptr_size].into_boxed_slice());
        module
            .define_data(data_id, &data_ctx)
            .map_err(|e| e.to_string())?;

        self.global_variables.insert(name.to_string(), data_id);
        Ok(())
    }

    pub fn register_function_metadata<M: Module>(
        &mut self,
        module: &mut M,
        name: &str,
        args: &[String],
        rest_param: Option<String>,
    ) {
        let int = module.target_config().pointer_type();
        let mut signature = module.make_signature();
        signature.params.push(AbiParam::new(int));
        for _ in args {
            signature.params.push(AbiParam::new(int));
        }
        if rest_param.is_some() {
            signature.params.push(AbiParam::new(int));
        }
        signature.returns.push(AbiParam::new(int));

        self.global_functions.insert(
            name.to_string(),
            FunctionMetadata {
                fixed_args: args.to_vec(),
                rest_param,
                signature,
            },
        );
    }

    fn module_compile_func<M: Module>(
        &mut self,
        module: &mut M,
        name: &str,
        args: &[String],
        rest_param: Option<String>,
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
        if rest_param.is_some() {
            self.ctx.func.signature.params.push(AbiParam::new(int));
        }
        self.ctx.func.signature.returns.push(AbiParam::new(int));

        // Store global signature early to avoid borrow check issues
        let signature = self.ctx.func.signature.clone();
        self.global_functions.insert(
            name.to_string(),
            FunctionMetadata {
                fixed_args: args.to_vec(),
                rest_param: rest_param.clone(),
                signature,
            },
        );

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
        if let Some(rest_name) = rest_param.clone() {
            let val = builder.block_params(entry_block)[args.len() + 1];
            let var = builder.declare_var(int);
            builder.def_var(var, val);
            initial_scope.insert(rest_name, var);
        }

        let mut result_val = builder.ins().iconst(int, 0);

        let compile_result = {
            let mut trans_ctx = FunctionTranslationContext {
                builder: &mut builder,
                module,
                builtins: &builtins,
                scopes: vec![initial_scope],
                captured_vars: HashMap::new(),
                env_param: Some(env_param),
                ptr_type: int,
                global_functions: &self.global_functions,
                global_variables: &self.global_variables,
            };

            for expr in body {
                result_val = trans_ctx.compile_expr(expr)?;
            }
            Ok::<(), String>(())
        };

        if let Err(err) = compile_result {
            builder.finalize();
            return Err(err);
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

    pub fn compile_top_level_init<M: Module>(
        &mut self,
        module: &mut M,
        name: &str,
        body: &[Value],
    ) -> Result<cranelift_module::FuncId, String> {
        self.module_clear_context(module);

        let int = module.target_config().pointer_type();

        if !self.builtins_initialized {
            self.data_ctx.define(b"%ld\n\0".to_vec().into_boxed_slice());
            let fmt_id = module
                .declare_data("printf_fmt", Linkage::Local, true, false)
                .map_err(|e| e.to_string())?;
            module
                .define_data(fmt_id, &self.data_ctx)
                .map_err(|e| e.to_string())?;
            self.builtins_initialized = true;
            self.data_ctx.clear();
        }

        let builtin_defs = builtins::declare_builtins(module)?;

        self.ctx.func.signature.params.push(AbiParam::new(int));
        self.ctx.func.signature.returns.push(AbiParam::new(int));

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let fmt_id = module
            .declare_data("printf_fmt", Linkage::Local, true, false)
            .map_err(|e| e.to_string())?;
        let printf_fmt_val = module.declare_data_in_func(fmt_id, builder.func);
        let builtins = Builtins {
            funcs: builtin_defs,
            printf_fmt: builder.ins().global_value(int, printf_fmt_val),
        };

        let env_param = builder.block_params(entry_block)[0];
        let mut trans_ctx = FunctionTranslationContext {
            builder: &mut builder,
            module,
            builtins: &builtins,
            scopes: vec![HashMap::new()],
            captured_vars: HashMap::new(),
            env_param: Some(env_param),
            ptr_type: int,
            global_functions: &self.global_functions,
            global_variables: &self.global_variables,
        };

        let mut result_val = trans_ctx.make_nil()?;
        for expr in body {
            result_val = trans_ctx.compile_top_level_expr(expr)?;
        }

        builder.ins().return_(&[result_val]);
        builder.seal_all_blocks();
        builder.finalize();

        let id = module
            .declare_function(name, Linkage::Local, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;
        module
            .define_function(id, &mut self.ctx)
            .map_err(|e| e.to_string())?;

        Ok(id)
    }

    pub fn compile_entry_point<M: Module>(
        &mut self,
        module: &mut M,
        _user_main_name: Option<&str>,
        _init_name: Option<&str>,
    ) -> Result<(), String> {
        self.module_clear_context(module);

        let int = module.target_config().pointer_type();

        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(int));
        let dlisp_main_id = module
            .declare_function("dlisp_main", Linkage::Import, &sig)
            .map_err(|e| e.to_string())?;

        let mut entry_sig = module.make_signature();
        entry_sig.params.push(AbiParam::new(int));
        entry_sig.returns.push(AbiParam::new(int));
        let entry_id = module
            .declare_function("dlisp_user_entry", Linkage::Local, &entry_sig)
            .map_err(|e| e.to_string())?;

        self.ctx
            .func
            .signature
            .returns
            .push(AbiParam::new(types::I32));

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        let entry = builder.create_block();
        builder.switch_to_block(entry);

        let local_entry = module.declare_func_in_func(entry_id, builder.func);
        let entry_addr = builder.ins().func_addr(int, local_entry);

        let local_dlisp_main = module.declare_func_in_func(dlisp_main_id, builder.func);
        builder.ins().call(local_dlisp_main, &[entry_addr]);

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

    pub fn compile_program_entry<M: Module>(
        &mut self,
        module: &mut M,
        name: &str,
        init_name: Option<&str>,
        user_main_name: Option<&str>,
    ) -> Result<cranelift_module::FuncId, String> {
        self.module_clear_context(module);

        let int = module.target_config().pointer_type();
        self.ctx.func.signature.params.push(AbiParam::new(int));
        self.ctx.func.signature.returns.push(AbiParam::new(int));
        let local_sig = self.ctx.func.signature.clone();

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        builder.seal_block(entry);

        let env_param = builder.block_params(entry)[0];
        let mut last_val = builder.ins().iconst(int, 0);

        if let Some(init_name) = init_name {
            let func_id = module
                .declare_function(init_name, Linkage::Local, &local_sig)
                .map_err(|e| e.to_string())?;
            let local_func = module.declare_func_in_func(func_id, builder.func);
            let call = builder.ins().call(local_func, &[env_param]);
            last_val = builder.inst_results(call)[0];
        }

        if let Some(user_main_name) = user_main_name {
            let func_id = if let Some(meta) = self.global_functions.get(user_main_name) {
                module
                    .declare_function(user_main_name, Linkage::Export, &meta.signature)
                    .map_err(|e| e.to_string())?
            } else {
                return Err(format!("Undefined entry function: {}", user_main_name));
            };
            let local_func = module.declare_func_in_func(func_id, builder.func);
            let call = builder.ins().call(local_func, &[env_param]);
            last_val = builder.inst_results(call)[0];
        }

        builder.ins().return_(&[last_val]);
        builder.seal_all_blocks();
        builder.finalize();

        let id = module
            .declare_function(name, Linkage::Local, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;
        module
            .define_function(id, &mut self.ctx)
            .map_err(|e| e.to_string())?;
        Ok(id)
    }

    fn module_clear_context<M: Module>(&mut self, module: &mut M) {
        module.clear_context(&mut self.ctx);
    }
}
