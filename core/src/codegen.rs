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
}

pub struct Builtins {
    pub printf: FuncId,
    pub printf_fmt: IrValue,
    pub dlisp_spawn: FuncId,
    pub dlisp_sleep: FuncId,
}

impl Default for CodeGen {
    fn default() -> Self {
        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: codegen::Context::new(),
            data_ctx: DataDescription::new(),
            builtins_initialized: false,
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
        // Reset context for reusable
        self.module_clear_context(module);

        let int = module.target_config().pointer_type();

        // Setup printf format string
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

        // Setup printf signature
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(int)); // format string
        sig.params.push(AbiParam::new(int)); // value
        sig.returns.push(AbiParam::new(types::I32));
        let printf_id = module
            .declare_function("printf", Linkage::Import, &sig)
            .map_err(|e| e.to_string())?;

        // Setup dlisp_spawn signature: void dlisp_spawn(void (*)(void))
        let mut spawn_sig = module.make_signature();
        spawn_sig.params.push(AbiParam::new(int)); // func_ptr
        let spawn_id = module
            .declare_function("dlisp_spawn", Linkage::Import, &spawn_sig)
            .map_err(|e| e.to_string())?;

        // Setup dlisp_sleep signature: void dlisp_sleep(u64)
        let mut sleep_sig = module.make_signature();
        sleep_sig.params.push(AbiParam::new(types::I64)); // ms
        let sleep_id = module
            .declare_function("dlisp_sleep", Linkage::Import, &sleep_sig)
            .map_err(|e| e.to_string())?;

        // Setup function signature
        for _ in args {
            self.ctx.func.signature.params.push(AbiParam::new(int));
        }
        self.ctx.func.signature.returns.push(AbiParam::new(int));

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);

        // Prep builtins for this function
        let printf_fmt_val = module.declare_data_in_func(fmt_id, builder.func);
        let builtins = Builtins {
            printf: printf_id,
            printf_fmt: builder.ins().global_value(int, printf_fmt_val),
            dlisp_spawn: spawn_id,
            dlisp_sleep: sleep_id,
        };

        // Map arg names to variables
        let mut initial_scope = HashMap::new();
        for (i, arg_name) in args.iter().enumerate() {
            let val = builder.block_params(entry_block)[i];
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
                ptr_type: int,
            };

            for expr in body {
                result_val = trans_ctx.compile_expr(expr)?;
            }
        }

        builder.ins().return_(&[result_val]);

        builder.seal_all_blocks();
        builder.finalize();

        let id = module
            .declare_function(name, Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;

        module
            .define_function(id, &mut self.ctx)
            .map_err(|e| e.to_string())?;

        Ok(id)
    }

    // Generate the main entry point shim that calls dlisp_main
    pub fn compile_entry_point<M: Module>(
        &mut self,
        module: &mut M,
        user_main_name: &str,
    ) -> Result<(), String> {
        self.module_clear_context(module);

        let int = module.target_config().pointer_type();

        // Declare dlisp_main: void dlisp_main(void (*)(void))
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(int));
        let dlisp_main_id = module
            .declare_function("dlisp_main", Linkage::Import, &sig)
            .map_err(|e| e.to_string())?;

        // Declare user main (to get its address)
        // User main signature: void -> void (or whatever we defined, currently generated as void -> void usually? or void -> int?)
        // In module_compile_func we defined sig as params: args, returns: int.
        // If main has 0 args, it is void -> int.
        // dlisp_main expects void -> void?
        // Rust fn() is technically void -> void in C ABI terms if not specified?
        // Wait, dlisp_main signature in Rust: `fn(extern "C" fn())`.
        // User main generated by us: extern "C" fn user_main() -> i64.
        // It returns a value.
        // We might need to wrap it or cast it.
        // Or just change dlisp_main to take `extern "C" fn() -> i64`?
        // Let's assume generic fn ptr for now or cast.
        // In Cranelift, a function pointer is just an integer/address.

        let mut user_sig = module.make_signature();
        user_sig.returns.push(AbiParam::new(int)); // It returns a value
        let user_main_id = module
            .declare_function(user_main_name, Linkage::Export, &user_sig) // It was exported
            .map_err(|e| e.to_string())?;

        // Define main
        // int main() { ... }
        self.ctx
            .func
            .signature
            .returns
            .push(AbiParam::new(types::I32)); // standard C main returns int

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        let entry = builder.create_block();
        builder.switch_to_block(entry);

        // Get address of user main
        let local_user_main = module.declare_func_in_func(user_main_id, builder.func);
        let user_main_addr = builder.ins().func_addr(int, local_user_main);

        // Call dlisp_main(user_main_addr)
        let local_dlisp_main = module.declare_func_in_func(dlisp_main_id, builder.func);
        builder.ins().call(local_dlisp_main, &[user_main_addr]);

        // Return 0
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

    // Clear context helper needs access to module
    fn module_clear_context<M: Module>(&mut self, module: &mut M) {
        module.clear_context(&mut self.ctx);
    }
}

struct FunctionTranslationContext<'a, 'func, M: Module> {
    builder: &'a mut FunctionBuilder<'func>,
    module: &'a mut M,
    builtins: &'a Builtins,
    scopes: Vec<HashMap<String, Variable>>,
    ptr_type: Type,
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
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Ok(self.builder.use_var(*var));
            }
        }

        // If not found in scopes, try to find it as a global function (e.g. for spawn)
        // We assume 0 args for now or generic void signature just to get the address?
        // Wait, to get address we usually need to know it exists.
        // We can speculatively declare it with a generic signature to get its ID/address.
        // But functions in DLisp are dynamic.
        // If we are referring to a top-level defined function, we can declare it.
        // Let's assume it returns pointer-sized int (our standard) and takes unknown args?
        // Cranelift requires exact signature for calls, but for address?
        // We can use the standard (int) -> int signature we use everywhere.

        let mut sig = self.module.make_signature();
        // We don't know the arity here easily without looking up.
        // But since we just want the address, maybe signature doesn't matter for `func_addr`?
        // It does matter for declaring it.
        // For now, let's assume it's a value referencing a function we compiled or will compile.
        // NOTE: In a real system we'd look up the function arity map.
        // Here we just use a dummy signature (void -> void or int -> int) to get a handle.
        // Ideally we shouldn't declare if it doesn't exist.

        // Strategy: Declaration with speculatively 0-arity signature.
        sig.returns.push(AbiParam::new(self.ptr_type));
        let func_id = self
            .module
            .declare_function(name, Linkage::Export, &sig)
            .map_err(|e| e.to_string())?;
        let local_func = self.module.declare_func_in_func(func_id, self.builder.func);
        let func_addr = self.builder.ins().func_addr(self.ptr_type, local_func);
        return Ok(func_addr);

        // Warning: This effectively means ALL unknown symbols are treated as global function addresses.
        // This defeats variable shadowing checks kind of, but only if they are missing.
        // It suppresses "Undefined variable".
        // Ideally we should verify if "name" is actually a known function.
        // But the compiler struct (AOTCompiler) knows that, CodeGen doesn't have the list.
        // For the MVP of spawn support, this enables getting the pointer.
    }

    fn compile_list(&mut self, list: &[Value]) -> Result<IrValue, String> {
        if list.is_empty() {
            return Ok(self.builder.ins().iconst(self.ptr_type, 0)); // nil?
        }
        if let Value::Symbol(ref op) = list[0] {
            // Check if 'op' is a variable (parameter) -> Indirect call
            if self.resolve_variable(op).is_ok() {
                return self.compile_indirect_call(op, list);
            }

            match op.as_str() {
                "let" => self.compile_let(list),
                "lambda" => self.compile_lambda(list),
                "spawn" => self.compile_spawn(list),
                "print" | "+" | "-" | "*" | "sleep" => self.compile_builtin(op, list),
                _ => self.compile_function_call(op, list),
            }
        } else {
            Err("JIT function calls not supported yet".to_string())
        }
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
        // (lambda (args) body...)
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

        // Generate unique name
        // Generate unique name
        let lambda_name = format!("lambda_{}", LAMBDA_COUNTER.fetch_add(1, Ordering::Relaxed));

        // Setup new context for lambda compilation
        let mut ctx = codegen::Context::new();
        let mut builder_context = FunctionBuilderContext::new();
        let int = self.ptr_type;

        // Signature
        for _ in &arg_names {
            ctx.func.signature.params.push(AbiParam::new(int));
        }
        ctx.func.signature.returns.push(AbiParam::new(int));

        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_context);
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);

            // Import builtins into this new function
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
            };

            let mut initial_scope = HashMap::new();
            for (i, arg_name) in arg_names.iter().enumerate() {
                let val = builder.block_params(entry_block)[i];
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
                    ptr_type: int,
                };

                for expr in body {
                    result_val = trans_ctx.compile_expr(expr)?;
                }
            }

            builder.ins().return_(&[result_val]);
            builder.seal_all_blocks();
            builder.finalize();
        }

        // Declare and define
        let id = self
            .module
            .declare_function(&lambda_name, Linkage::Export, &ctx.func.signature)
            .map_err(|e| e.to_string())?;

        self.module
            .define_function(id, &mut ctx)
            .map_err(|e| e.to_string())?;

        // Get function pointer in CURRENT function
        let local_func = self.module.declare_func_in_func(id, self.builder.func);
        let func_addr = self.builder.ins().func_addr(self.ptr_type, local_func);

        Ok(func_addr)
    }

    fn compile_indirect_call(
        &mut self,
        func_var_name: &str,
        list: &[Value],
    ) -> Result<IrValue, String> {
        let func_ptr = self.resolve_variable(func_var_name)?;

        let mut args = Vec::new();
        for arg in &list[1..] {
            args.push(self.compile_expr(arg)?);
        }

        let mut sig = self.module.make_signature();
        for _ in &args {
            sig.params.push(AbiParam::new(self.ptr_type));
        }
        sig.returns.push(AbiParam::new(self.ptr_type));

        let sig_ref = self.builder.import_signature(sig);

        let call = self.builder.ins().call_indirect(sig_ref, func_ptr, &args);
        let result = self.builder.inst_results(call)[0];
        Ok(result)
    }

    fn compile_spawn(&mut self, list: &[Value]) -> Result<IrValue, String> {
        if list.len() != 2 {
            return Err("spawn requires exactly one argument (function)".to_string());
        }
        // Compile argument to get function pointer
        let func_ptr = self.compile_expr(&list[1])?;

        let local_spawn = self
            .module
            .declare_func_in_func(self.builtins.dlisp_spawn, self.builder.func);

        self.builder.ins().call(local_spawn, &[func_ptr]);

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
            "+" | "-" | "*" => {
                if list.len() == 3 {
                    let lhs = self.compile_expr(&list[1])?;
                    let rhs = self.compile_expr(&list[2])?;
                    match op {
                        "+" => Ok(self.builder.ins().iadd(lhs, rhs)),
                        "-" => Ok(self.builder.ins().isub(lhs, rhs)),
                        "*" => Ok(self.builder.ins().imul(lhs, rhs)),
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
