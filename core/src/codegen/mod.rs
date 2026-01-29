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

        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(int));
        sig.params.push(AbiParam::new(int));
        sig.returns.push(AbiParam::new(types::I32));
        let printf_id = module
            .declare_function("printf", Linkage::Import, &sig)
            .map_err(|e| e.to_string())?;

        // dlisp_spawn(Closure*)
        let mut spawn_sig = module.make_signature();
        spawn_sig.params.push(AbiParam::new(int));
        let spawn_id = module
            .declare_function("dlisp_spawn", Linkage::Import, &spawn_sig)
            .map_err(|e| e.to_string())?;

        let mut sleep_sig = module.make_signature();
        sleep_sig.params.push(AbiParam::new(int));
        sleep_sig.returns.push(AbiParam::new(int));
        let sleep_id = module
            .declare_function("dlisp_sleep", Linkage::Import, &sleep_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_gc_malloc(size_t) -> void*
        let mut malloc_sig = module.make_signature();
        malloc_sig.params.push(AbiParam::new(int));
        malloc_sig.returns.push(AbiParam::new(int));
        let malloc_id = module
            .declare_function("dlisp_gc_malloc", Linkage::Import, &malloc_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_make_int(i64) -> DlispValue*
        let mut make_int_sig = module.make_signature();
        make_int_sig.params.push(AbiParam::new(types::I64));
        make_int_sig.returns.push(AbiParam::new(int));
        let make_int_id = module
            .declare_function("dlisp_make_int", Linkage::Import, &make_int_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_make_string(char*) -> DlispValue*
        let mut make_string_sig = module.make_signature();
        make_string_sig.params.push(AbiParam::new(int));
        make_string_sig.returns.push(AbiParam::new(int));
        let make_string_id = module
            .declare_function("dlisp_make_string", Linkage::Import, &make_string_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_make_symbol(char*) -> DlispValue*
        let mut make_symbol_sig = module.make_signature();
        make_symbol_sig.params.push(AbiParam::new(int));
        make_symbol_sig.returns.push(AbiParam::new(int));
        let make_symbol_id = module
            .declare_function("dlisp_make_symbol", Linkage::Import, &make_symbol_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_make_cons(DlispValue*, DlispValue*) -> DlispValue*
        let mut make_cons_sig = module.make_signature();
        make_cons_sig.params.push(AbiParam::new(int));
        make_cons_sig.params.push(AbiParam::new(int));
        make_cons_sig.returns.push(AbiParam::new(int));
        let make_cons_id = module
            .declare_function("dlisp_make_cons", Linkage::Import, &make_cons_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_make_float(f64) -> DlispValue*
        let mut make_float_sig = module.make_signature();
        make_float_sig.params.push(AbiParam::new(types::F64));
        make_float_sig.returns.push(AbiParam::new(int));
        let make_float_id = module
            .declare_function("dlisp_make_float", Linkage::Import, &make_float_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_make_bool(bool) -> DlispValue*
        let mut make_bool_sig = module.make_signature();
        make_bool_sig.params.push(AbiParam::new(types::I8)); // bool as i8
        make_bool_sig.returns.push(AbiParam::new(int));
        let make_bool_id = module
            .declare_function("dlisp_make_bool", Linkage::Import, &make_bool_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_make_nil() -> DlispValue*
        let mut make_nil_sig = module.make_signature();
        make_nil_sig.returns.push(AbiParam::new(int));
        let make_nil_id = module
            .declare_function("dlisp_make_nil", Linkage::Import, &make_nil_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_print(DlispValue*)
        let mut print_sig = module.make_signature();
        print_sig.params.push(AbiParam::new(int));
        let print_id = module
            .declare_function("dlisp_print", Linkage::Import, &print_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_car(DlispValue*) -> DlispValue*
        let mut car_sig = module.make_signature();
        car_sig.params.push(AbiParam::new(int));
        car_sig.returns.push(AbiParam::new(int));
        let car_id = module
            .declare_function("dlisp_car", Linkage::Import, &car_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_cdr(DlispValue*) -> DlispValue*
        let mut cdr_sig = module.make_signature();
        cdr_sig.params.push(AbiParam::new(int));
        cdr_sig.returns.push(AbiParam::new(int));
        let cdr_id = module
            .declare_function("dlisp_cdr", Linkage::Import, &cdr_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_add(DlispValue*, DlispValue*) -> DlispValue*
        let mut add_sig = module.make_signature();
        add_sig.params.push(AbiParam::new(int));
        add_sig.params.push(AbiParam::new(int));
        add_sig.returns.push(AbiParam::new(int));
        let add_id = module
            .declare_function("dlisp_add", Linkage::Import, &add_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_sub(DlispValue*, DlispValue*) -> DlispValue*
        let mut sub_sig = module.make_signature();
        sub_sig.params.push(AbiParam::new(int));
        sub_sig.params.push(AbiParam::new(int));
        sub_sig.returns.push(AbiParam::new(int));
        let sub_id = module
            .declare_function("dlisp_sub", Linkage::Import, &sub_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_mul(DlispValue*, DlispValue*) -> DlispValue*
        let mut mul_sig = module.make_signature();
        mul_sig.params.push(AbiParam::new(int));
        mul_sig.params.push(AbiParam::new(int));
        mul_sig.returns.push(AbiParam::new(int));
        let mul_id = module
            .declare_function("dlisp_mul", Linkage::Import, &mul_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_gt(DlispValue*, DlispValue*) -> DlispValue*
        let mut gt_sig = module.make_signature();
        gt_sig.params.push(AbiParam::new(int));
        gt_sig.params.push(AbiParam::new(int));
        gt_sig.returns.push(AbiParam::new(int));
        let gt_id = module
            .declare_function("dlisp_gt", Linkage::Import, &gt_sig)
            .map_err(|e| e.to_string())?;

        // dlisp_is_truthy(DlispValue*) -> i32 (c_int)
        let mut truthy_sig = module.make_signature();
        truthy_sig.params.push(AbiParam::new(int));
        truthy_sig.returns.push(AbiParam::new(types::I32));
        let truthy_id = module
            .declare_function("dlisp_is_truthy", Linkage::Import, &truthy_sig)
            .map_err(|e| e.to_string())?;

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
            printf: printf_id,
            printf_fmt: builder.ins().global_value(int, printf_fmt_val),
            dlisp_spawn: spawn_id,
            dlisp_sleep: sleep_id,
            gc_malloc: malloc_id,
            dlisp_make_int: make_int_id,
            dlisp_make_string: make_string_id,
            dlisp_make_symbol: make_symbol_id,
            dlisp_make_cons: make_cons_id,
            dlisp_make_float: make_float_id,
            dlisp_make_bool: make_bool_id,
            dlisp_make_nil: make_nil_id,
            dlisp_car: car_id,
            dlisp_cdr: cdr_id,
            dlisp_print: print_id,
            dlisp_add: add_id,
            dlisp_sub: sub_id,
            dlisp_mul: mul_id,
            dlisp_gt: gt_id,
            dlisp_is_truthy: truthy_id,
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
