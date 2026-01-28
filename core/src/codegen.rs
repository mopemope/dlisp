use crate::ast::Value;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static LAMBDA_COUNTER: AtomicUsize = AtomicUsize::new(0);

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

pub struct Builtins {
    pub printf: FuncId,
    pub printf_fmt: IrValue,
    pub dlisp_spawn: FuncId,
    pub dlisp_sleep: FuncId,
    pub gc_malloc: FuncId,
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
        sleep_sig.params.push(AbiParam::new(types::I64));
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

    fn analyze_free_variables(body: &[Value], args: &[String]) -> Vec<String> {
        let mut free_vars = Vec::new();
        let mut bound_vars = std::collections::HashSet::new();
        for arg in args {
            bound_vars.insert(arg.clone());
        }

        for expr in body {
            Self::find_free_vars(expr, &mut bound_vars, &mut free_vars);
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
                    // It might be a global function or a free variable.
                    // For now we assume everything potentially valid is captured?
                    // But global functions shouldn't be captured.
                    // We can't easily distinguish without global context.
                    // However, typically in Lisp/Scheme, if it's not bound locally, it's free.
                    // If it turns out to be global at runtime, capturing it is harmless (just copying a pointer/value presumably?)
                    // Or we check standard builtins?
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
                                    if let Value::List(pair) = b {
                                        if pair.len() == 2 {
                                            // RHS is evaluated in current scope
                                            Self::find_free_vars(&pair[1], bound, free);
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
                                Self::find_free_vars(sub, &mut new_bound, free);
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
                                Self::find_free_vars(sub, &mut new_bound, free);
                            }
                        }
                    }
                    _ => {
                        for sub in list {
                            Self::find_free_vars(sub, bound, free);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

struct FunctionTranslationContext<'a, 'func, M: Module> {
    builder: &'a mut FunctionBuilder<'func>,
    module: &'a mut M,
    builtins: &'a Builtins,
    scopes: Vec<HashMap<String, Variable>>,
    // Map captured var name to offset (bytes) in env struct
    captured_vars: HashMap<String, u32>,
    env_param: Option<IrValue>,
    ptr_type: Type,
    global_signatures: &'a HashMap<String, Signature>,
}

impl<'a, 'func, M: Module> FunctionTranslationContext<'a, 'func, M> {
    fn compile_expr(&mut self, val: &Value) -> Result<IrValue, String> {
        match val {
            Value::Integer(n) => Ok(self.builder.ins().iconst(self.ptr_type, *n)),
            Value::Symbol(s) => self.resolve_variable(s),
            Value::List(list) => self.compile_list(list),
            _ => Err(format!("Unsupported Value type for JIT: {:?}", val)),
        }
    }

    fn resolve_variable(&mut self, name: &str) -> Result<IrValue, String> {
        // 1. Local Scopes (Stack)
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Ok(self.builder.use_var(*var));
            }
        }

        // 2. Captured Variables (Env)
        if let Some(&offset) = self.captured_vars.get(name) {
            if let Some(env) = self.env_param {
                return Ok(self.builder.ins().load(
                    self.ptr_type,
                    MemFlags::new(),
                    env,
                    offset as i32,
                ));
            }
        }

        // 3. Global Function (Allocation of Closure)
        // We assume it's a global function if not found locally.
        // We create a Closure { func_ptr, NULL } on the heap.
        // This is necessary because "spawn" expects a Closure*.
        // NOTE: This leaks memory if used repeatedly.

        // Declare function (speculative signature)
        // Global functions now take (env, args...), but we only need address here.
        // We use (int) -> int signature for address.
        let sig = if let Some(known_sig) = self.global_signatures.get(name) {
            known_sig.clone()
        } else {
            let mut s = self.module.make_signature();
            s.params.push(AbiParam::new(self.ptr_type)); // env
            s.returns.push(AbiParam::new(self.ptr_type));
            s
        };
        let func_id = self
            .module
            .declare_function(name, Linkage::Export, &sig)
            .map_err(|e| e.to_string())?;
        let local_func = self.module.declare_func_in_func(func_id, self.builder.func);
        let func_addr = self.builder.ins().func_addr(self.ptr_type, local_func);

        // Allocate Closure
        let closure_size = 16;
        let size_val = self.builder.ins().iconst(self.ptr_type, closure_size);
        let local_malloc = self
            .module
            .declare_func_in_func(self.builtins.gc_malloc, self.builder.func);
        let call = self.builder.ins().call(local_malloc, &[size_val]);
        let closure_ptr = self.builder.inst_results(call)[0];

        // Store func ptr at offset 0
        self.builder
            .ins()
            .store(MemFlags::new(), func_addr, closure_ptr, 0);
        // Store NULL env at offset 8
        let null_val = self.builder.ins().iconst(self.ptr_type, 0);
        self.builder
            .ins()
            .store(MemFlags::new(), null_val, closure_ptr, 8);

        return Ok(closure_ptr);
    }

    fn is_variable_bound(&self, name: &str) -> bool {
        // Check local scopes
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(name) {
                return true;
            }
        }
        // Check captured vars (only if environment is present)
        if self.env_param.is_some() && self.captured_vars.contains_key(name) {
            return true;
        }
        false
    }

    fn compile_list(&mut self, list: &[Value]) -> Result<IrValue, String> {
        if list.is_empty() {
            return Ok(self.builder.ins().iconst(self.ptr_type, 0)); // nil?
        }

        if let Value::Symbol(ref op) = list[0] {
            match op.as_str() {
                "let" => self.compile_let(list),
                "if" => self.compile_if(list),
                "lambda" => self.compile_lambda(list),
                "spawn" => self.compile_spawn(list),
                "print" | "+" | "-" | "*" | "sleep" | ">" => self.compile_builtin(op, list),
                _ => {
                    // Check if 'op' is a variable (parameter) -> Indirect call
                    if self.is_variable_bound(op) {
                        self.compile_indirect_call(op, list)
                    } else {
                        self.compile_function_call(op, list)
                    }
                }
            }
        } else {
            Err("JIT function calls not supported yet".to_string())
        }
    }

    fn compile_if(&mut self, list: &[Value]) -> Result<IrValue, String> {
        if list.len() < 3 {
            return Err("if requires condition and then-branch".to_string());
        }

        let cond_val = self.compile_expr(&list[1])?;

        let then_block = self.builder.create_block();
        let else_block = self.builder.create_block();
        let merge_block = self.builder.create_block();

        // Use stack slot to pass result effectively acting as Phi
        let slot = self.builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            8,
            3,
        )); // 8 bytes for ptr_type, 2^3 align

        // Branch
        self.builder
            .ins()
            .brif(cond_val, then_block, &[], else_block, &[]);

        // Then Block
        self.builder.switch_to_block(then_block);
        self.builder.seal_block(then_block);
        let then_val = self.compile_expr(&list[2])?;
        self.builder.ins().stack_store(then_val, slot, 0);
        self.builder.ins().jump(merge_block, &[]);

        // Else Block
        self.builder.switch_to_block(else_block);
        self.builder.seal_block(else_block);
        let else_val = if list.len() > 3 {
            self.compile_expr(&list[3])?
        } else {
            self.builder.ins().iconst(self.ptr_type, 0)
        };
        self.builder.ins().stack_store(else_val, slot, 0);
        self.builder.ins().jump(merge_block, &[]);

        // Merge Block
        self.builder.switch_to_block(merge_block);
        self.builder.seal_block(merge_block);

        Ok(self.builder.ins().stack_load(self.ptr_type, slot, 0))
    }

    fn compile_let(&mut self, list: &[Value]) -> Result<IrValue, String> {
        if list.len() < 3 {
            return Err("let requires bindings and body".to_string());
        }

        let bindings_clause = &list[1];
        let bindings = match bindings_clause {
            Value::List(l) => l,
            Value::Nil => &Vec::new()[..],
            _ => return Err("let bindings must be a list".to_string()),
        };

        // Evaluate bindings in CURRENT context
        let mut evaluated_bindings = Vec::new();
        for binding in bindings {
            if let Value::List(pair) = binding {
                if pair.len() != 2 {
                    return Err("let binding invalid".to_string());
                }
                let name = match &pair[0] {
                    Value::Symbol(s) => s.clone(),
                    _ => return Err("let binding name must be symbol".to_string()),
                };
                let val = self.compile_expr(&pair[1])?;
                evaluated_bindings.push((name, val));
            } else {
                return Err("let binding must be a list".to_string());
            }
        }

        // Declare variables and push scope
        self.scopes.push(HashMap::new());
        {
            let current_scope = self.scopes.last_mut().unwrap();
            for (name, val) in evaluated_bindings {
                let var = self.builder.declare_var(self.ptr_type);
                self.builder.def_var(var, val);
                current_scope.insert(name, var);
            }
        }

        // Compile body
        let mut res = self.builder.ins().iconst(self.ptr_type, 0);
        for expr in &list[2..] {
            res = self.compile_expr(expr)?;
        }

        // Pop scope
        self.scopes.pop();

        Ok(res)
    }

    fn compile_lambda(&mut self, list: &[Value]) -> Result<IrValue, String> {
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
        let free_vars = CodeGen::analyze_free_variables(body, &arg_names);

        // 2. Allocate Environment
        let ptr_size = 8;
        let env_size = (free_vars.len() * ptr_size) as i64;

        // Malloc env if size > 0
        let env_ptr_val = if env_size > 0 {
            let size_val = self.builder.ins().iconst(self.ptr_type, env_size);
            let local_malloc = self
                .module
                .declare_func_in_func(self.builtins.gc_malloc, self.builder.func);
            let call = self.builder.ins().call(local_malloc, &[size_val]);
            self.builder.inst_results(call)[0]
        } else {
            self.builder.ins().iconst(self.ptr_type, 0) // NULL
        };

        // 3. Populate Environment
        let mut captured_offsets = HashMap::new();
        for (i, var_name) in free_vars.iter().enumerate() {
            let val = self.resolve_variable(var_name)?;
            let offset = (i * ptr_size) as i32;
            self.builder
                .ins()
                .store(MemFlags::new(), val, env_ptr_val, offset);
            captured_offsets.insert(var_name.clone(), offset as u32);
        }

        // 4. Compile Lambda Function
        let lambda_name = format!("lambda_{}", LAMBDA_COUNTER.fetch_add(1, Ordering::Relaxed));
        let mut ctx = codegen::Context::new();
        let mut builder_context = FunctionBuilderContext::new();
        let int = self.ptr_type;

        // Signature: (env, args...)
        ctx.func.signature.params.push(AbiParam::new(int));
        for _ in &arg_names {
            ctx.func.signature.params.push(AbiParam::new(int));
        }
        ctx.func.signature.returns.push(AbiParam::new(int));

        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_context);
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);

            // Import builtins
            let fmt_id = self
                .module
                .declare_data("printf_fmt", Linkage::Local, true, false)
                .map_err(|e| e.to_string())?;
            let printf_fmt_val = self.module.declare_data_in_func(fmt_id, builder.func);
            let inner_builtins = Builtins {
                printf: self.builtins.printf,
                printf_fmt: builder.ins().global_value(int, printf_fmt_val),
                dlisp_spawn: self.builtins.dlisp_spawn,
                dlisp_sleep: self.builtins.dlisp_sleep,
                gc_malloc: self.builtins.gc_malloc,
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
                    module: self.module,
                    builtins: &inner_builtins,
                    scopes: vec![initial_scope],
                    captured_vars: captured_offsets,
                    env_param: Some(env_param),
                    ptr_type: int,
                    global_signatures: self.global_signatures,
                };

                for expr in body {
                    result_val = trans_ctx.compile_expr(expr)?;
                }
            }

            builder.ins().return_(&[result_val]);
            builder.seal_all_blocks();
            builder.finalize();
        }

        let id = self
            .module
            .declare_function(&lambda_name, Linkage::Export, &ctx.func.signature)
            .map_err(|e| e.to_string())?;
        self.module
            .define_function(id, &mut ctx)
            .map_err(|e| e.to_string())?;

        let local_func = self.module.declare_func_in_func(id, self.builder.func);
        let func_addr = self.builder.ins().func_addr(self.ptr_type, local_func);

        // 5. Create Closure Struct { func_ptr, env_ptr }
        let closure_size = 16;
        let size_val = self.builder.ins().iconst(self.ptr_type, closure_size);
        let local_malloc = self
            .module
            .declare_func_in_func(self.builtins.gc_malloc, self.builder.func);
        let call = self.builder.ins().call(local_malloc, &[size_val]);
        let closure_ptr = self.builder.inst_results(call)[0];

        self.builder
            .ins()
            .store(MemFlags::new(), func_addr, closure_ptr, 0);
        self.builder
            .ins()
            .store(MemFlags::new(), env_ptr_val, closure_ptr, 8);

        Ok(closure_ptr)
    }

    fn compile_indirect_call(
        &mut self,
        func_var_name: &str,
        list: &[Value],
    ) -> Result<IrValue, String> {
        let closure_ptr = self.resolve_variable(func_var_name)?;

        let mut args = Vec::new();
        // Indirect Call: args are (env, args...)
        // We get env from closure_ptr->env (offset 8)
        let env_ptr = self
            .builder
            .ins()
            .load(self.ptr_type, MemFlags::new(), closure_ptr, 8);
        args.push(env_ptr);

        for arg in &list[1..] {
            args.push(self.compile_expr(arg)?);
        }

        let mut sig = self.module.make_signature();
        for _ in &args {
            sig.params.push(AbiParam::new(self.ptr_type));
        }
        sig.returns.push(AbiParam::new(self.ptr_type));

        let sig_ref = self.builder.import_signature(sig);

        // Load func ptr from closure_ptr->func (offset 0)
        let func_ptr = self
            .builder
            .ins()
            .load(self.ptr_type, MemFlags::new(), closure_ptr, 0);

        let call = self.builder.ins().call_indirect(sig_ref, func_ptr, &args);
        let result = self.builder.inst_results(call)[0];
        Ok(result)
    }

    fn compile_spawn(&mut self, list: &[Value]) -> Result<IrValue, String> {
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
        let closure_ptr = self.compile_lambda(&synthetic_lambda)?;

        // Call dlisp_spawn(closure_ptr)
        let local_spawn = self
            .module
            .declare_func_in_func(self.builtins.dlisp_spawn, self.builder.func);

        self.builder.ins().call(local_spawn, &[closure_ptr]);

        // spawn returns nil (0)
        Ok(self.builder.ins().iconst(self.ptr_type, 0))
    }

    fn compile_builtin(&mut self, op: &str, list: &[Value]) -> Result<IrValue, String> {
        match op {
            "sleep" => {
                if list.len() != 2 {
                    return Err("sleep requires 1 arg (ms)".to_string());
                }
                let ms_val = self.compile_expr(&list[1])?;
                // Since our values are pointers/integers (i64), we can treat it as u64 ms?
                // Assuming compile_expr returns I64.

                let local_sleep = self
                    .module
                    .declare_func_in_func(self.builtins.dlisp_sleep, self.builder.func);

                self.builder.ins().call(local_sleep, &[ms_val]);
                Ok(self.builder.ins().iconst(self.ptr_type, 0))
            }
            "print" => {
                if list.len() != 2 {
                    return Err("print takes 1 arg".to_string());
                }
                let arg_val = self.compile_expr(&list[1])?;

                let local_printf = self
                    .module
                    .declare_func_in_func(self.builtins.printf, self.builder.func);

                self.builder
                    .ins()
                    .call(local_printf, &[self.builtins.printf_fmt, arg_val]);
                Ok(arg_val)
            }
            "+" | "-" | "*" | ">" => {
                if list.len() == 3 {
                    let lhs = self.compile_expr(&list[1])?;
                    let rhs = self.compile_expr(&list[2])?;
                    match op {
                        "+" => Ok(self.builder.ins().iadd(lhs, rhs)),
                        "-" => Ok(self.builder.ins().isub(lhs, rhs)),
                        "*" => Ok(self.builder.ins().imul(lhs, rhs)),
                        ">" => {
                            let cmp = self.builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs);
                            let one = self.builder.ins().iconst(self.ptr_type, 1);
                            let zero = self.builder.ins().iconst(self.ptr_type, 0);
                            Ok(self.builder.ins().select(cmp, one, zero))
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

    fn compile_function_call(&mut self, op: &str, list: &[Value]) -> Result<IrValue, String> {
        let mut args = Vec::new();
        // Direct call: pass NULL env
        args.push(self.builder.ins().iconst(self.ptr_type, 0));

        for arg in &list[1..] {
            args.push(self.compile_expr(arg)?);
        }

        let mut sig = self.module.make_signature();
        for _ in &args {
            sig.params.push(AbiParam::new(self.ptr_type));
        }
        sig.returns.push(AbiParam::new(self.ptr_type));

        let func_id = self
            .module
            .declare_function(op, Linkage::Export, &sig)
            .map_err(|e| e.to_string())?;

        let local_func = self.module.declare_func_in_func(func_id, self.builder.func);
        let call = self.builder.ins().call(local_func, &args);
        let result = self.builder.inst_results(call)[0];
        Ok(result)
    }
}
