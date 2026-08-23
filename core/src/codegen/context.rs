use crate::ast::Value;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_module::{DataId, Linkage, Module};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct Builtins {
    pub funcs: crate::codegen::builtins::BuiltinDefinitions,
    pub printf_fmt: IrValue,
}

pub struct FunctionTranslationContext<'a, 'func, M: Module> {
    pub builder: &'a mut FunctionBuilder<'func>,
    pub module: &'a mut M,
    pub builtins: &'a Builtins,
    pub scopes: Vec<HashMap<String, Variable>>,
    // Map captured var name to offset (bytes) in env struct
    pub captured_vars: HashMap<String, u32>,
    pub env_param: Option<IrValue>,
    pub ptr_type: Type,
    pub global_functions: &'a HashMap<String, crate::codegen::FunctionMetadata>,
    pub global_variables: &'a HashMap<String, DataId>,
}

impl<'a, 'func, M: Module> FunctionTranslationContext<'a, 'func, M> {
    pub fn make_nil(&mut self) -> Result<IrValue, String> {
        let func = self
            .module
            .declare_func_in_func(self.builtins.funcs.dlisp_make_nil, self.builder.func);
        let call = self.builder.ins().call(func, &[]);
        Ok(self.builder.inst_results(call)[0])
    }

    pub fn compile_expr(&mut self, val: &Value) -> Result<IrValue, String> {
        match val {
            Value::Integer(n) => {
                let val = self.builder.ins().iconst(types::I64, *n);
                let func = self
                    .module
                    .declare_func_in_func(self.builtins.funcs.dlisp_make_int, self.builder.func);
                let call = self.builder.ins().call(func, &[val]);
                Ok(self.builder.inst_results(call)[0])
            }
            Value::String(s) => {
                // Define string data
                let mut hasher = DefaultHasher::new();
                s.hash(&mut hasher);
                let hash = hasher.finish();

                let mut data_ctx = cranelift_module::DataDescription::new();
                let mut bytes = s.clone().into_bytes();
                bytes.push(0); // Null terminator
                data_ctx.define(bytes.into_boxed_slice());

                // define_data might err if already defined.
                // Check if we need to define it. A workaround is to ignore error if it says duplicate?
                // Or tracking defined strings.
                // However, cranelift_module doesn't expose "is_defined".
                // We'll just try define, and simple map_err.
                // IF it fails, it might be because it's already defined.
                // But define_data is supposed to be called once.
                // To avoid multiple definitions: use a unique name per call site? Or deduplicate?
                // Deduplication is better. But complex here.
                // Let's rely on name uniqueness by including a counter?
                // Or just try define and ignore error? No, define_data overwrites?
                // Actually, if we use the hash, it's the SAME content. So we can just define it once?
                // But define_data fails if called twice on same ID.
                // NOTE: For now, I'll suffix with a random number or similar if I could.
                // But I can't easily.
                // Correct approach: Try to define. If error, assume it's because it exists?
                // Actually, `module.define_data` checks `!self.declarations[data_id].defined`.

                // Let's assume unique strings for now by appending a counter helper?
                // FunctionTranslationContext doesn't carry a mutable counter that persists across functions.
                // CodeGen does. But we are inside FunctionTranslationContext.

                // Quick hack: Use a random suffix via pointer address of the Value reference?
                let ptr_addr = s.as_ptr() as usize;
                let unique_name = format!("str_{}_{}", hash, ptr_addr);
                let data_id = self
                    .module
                    .declare_data(&unique_name, Linkage::Local, false, false)
                    .map_err(|e| e.to_string())?;

                self.module
                    .define_data(data_id, &data_ctx)
                    .map_err(|e| e.to_string())?;

                let global_val = self.module.declare_data_in_func(data_id, self.builder.func);
                let ptr = self.builder.ins().global_value(self.ptr_type, global_val);

                let func = self
                    .module
                    .declare_func_in_func(self.builtins.funcs.dlisp_make_string, self.builder.func);
                let call = self.builder.ins().call(func, &[ptr]);
                Ok(self.builder.inst_results(call)[0])
            }
            Value::Float(f) => {
                let val = self.builder.ins().f64const(*f);
                let func = self
                    .module
                    .declare_func_in_func(self.builtins.funcs.dlisp_make_float, self.builder.func);
                let call = self.builder.ins().call(func, &[val]);
                Ok(self.builder.inst_results(call)[0])
            }
            Value::Bool(b) => {
                let val = self.builder.ins().iconst(types::I8, if *b { 1 } else { 0 });
                let func = self
                    .module
                    .declare_func_in_func(self.builtins.funcs.dlisp_make_bool, self.builder.func);
                let call = self.builder.ins().call(func, &[val]);
                Ok(self.builder.inst_results(call)[0])
            }
            Value::Nil => self.make_nil(),
            Value::Symbol(s) => self.resolve_variable(s),
            Value::Keyword(s) => {
                let mut hasher = DefaultHasher::new();
                s.hash(&mut hasher);
                let hash = hasher.finish();

                let mut data_ctx = cranelift_module::DataDescription::new();
                let mut bytes = s.clone().into_bytes();
                bytes.push(0); // Null terminator
                data_ctx.define(bytes.into_boxed_slice());

                let ptr_addr = s.as_ptr() as usize;
                let unique_name = format!("key_{}_{}", hash, ptr_addr);
                let data_id = self
                    .module
                    .declare_data(&unique_name, Linkage::Local, false, false)
                    .map_err(|e| e.to_string())?;

                self.module
                    .define_data(data_id, &data_ctx)
                    .map_err(|e| e.to_string())?;

                let global_val = self.module.declare_data_in_func(data_id, self.builder.func);
                let ptr = self.builder.ins().global_value(self.ptr_type, global_val);

                let func = self.module.declare_func_in_func(
                    self.builtins.funcs.dlisp_make_keyword,
                    self.builder.func,
                );
                let call = self.builder.ins().call(func, &[ptr]);
                Ok(self.builder.inst_results(call)[0])
            }
            Value::List(list) => self.compile_list(list),
            Value::Vector(vec) => {
                let capacity = vec.len();
                let cap_val = self.builder.ins().iconst(self.ptr_type, capacity as i64);
                let func = self
                    .module
                    .declare_func_in_func(self.builtins.funcs.dlisp_make_vector, self.builder.func);
                let call = self.builder.ins().call(func, &[cap_val]);
                let vec_ptr = self.builder.inst_results(call)[0];

                for arg in vec {
                    let arg_val = self.compile_expr(arg)?;
                    let push_func = self.module.declare_func_in_func(
                        self.builtins.funcs.dlisp_vector_push,
                        self.builder.func,
                    );
                    // dlisp_vector_push returns nothing; it mutates the vector in place.
                    self.builder.ins().call(push_func, &[vec_ptr, arg_val]);
                }
                Ok(vec_ptr)
            }
            Value::Map(map) => {
                let func = self
                    .module
                    .declare_func_in_func(self.builtins.funcs.dlisp_make_map, self.builder.func);
                let call = self.builder.ins().call(func, &[]);
                let mut map_ptr = self.builder.inst_results(call)[0];

                for (k, v) in map.iter() {
                    let k_val = self.compile_expr(k)?;
                    let v_val = self.compile_expr(v)?;
                    let assoc_func = self.module.declare_func_in_func(
                        self.builtins.funcs.dlisp_map_assoc,
                        self.builder.func,
                    );
                    let assoc_call = self
                        .builder
                        .ins()
                        .call(assoc_func, &[map_ptr, k_val, v_val]);
                    map_ptr = self.builder.inst_results(assoc_call)[0]; // Updated map
                }
                Ok(map_ptr)
            }
            _ => Err(format!("Unsupported Value type for JIT: {:?}", val)),
        }
    }

    pub fn resolve_variable(&mut self, name: &str) -> Result<IrValue, String> {
        // 1. Local Scopes (Stack)
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Ok(self.builder.use_var(*var));
            }
        }

        // 2. Captured Variables (Env)
        #[allow(clippy::collapsible_if)]
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

        // 3. Global Variables
        if let Some(data_id) = self.global_variables.get(name) {
            let global_val = self
                .module
                .declare_data_in_func(*data_id, self.builder.func);
            let addr = self.builder.ins().global_value(self.ptr_type, global_val);
            return Ok(self
                .builder
                .ins()
                .load(self.ptr_type, MemFlags::new(), addr, 0));
        }

        // 4. Global Function (Allocation of Closure)
        let (sig, linkage) = if let Some(known_sig) = self.global_functions.get(name) {
            (known_sig.signature.clone(), Linkage::Export)
        } else {
            let mut s = self.module.make_signature();
            s.params.push(AbiParam::new(self.ptr_type)); // env
            s.returns.push(AbiParam::new(self.ptr_type));
            (s, Linkage::Import)
        };
        let func_id = self
            .module
            .declare_function(name, linkage, &sig)
            .map_err(|e| e.to_string())?;
        let local_func = self.module.declare_func_in_func(func_id, self.builder.func);
        let func_addr = self.builder.ins().func_addr(self.ptr_type, local_func);

        // Allocate Closure via runtime
        let null_val = self.builder.ins().iconst(self.ptr_type, 0);
        let local_make_closure = self
            .module
            .declare_func_in_func(self.builtins.funcs.dlisp_make_closure, self.builder.func);
        let call = self
            .builder
            .ins()
            .call(local_make_closure, &[null_val, func_addr]);
        let closure_ptr = self.builder.inst_results(call)[0];

        Ok(closure_ptr)
    }

    pub fn is_variable_bound(&self, name: &str) -> bool {
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
        if self.global_variables.contains_key(name) {
            return true;
        }
        false
    }

    pub fn compile_list(&mut self, list: &[Value]) -> Result<IrValue, String> {
        if list.is_empty() {
            return self.make_nil();
        }

        if let Value::Symbol(ref op) = list[0] {
            match op.as_str() {
                "let" => crate::codegen::forms::let_expr::compile_let(self, list, false),
                "let*" => crate::codegen::forms::let_expr::compile_let(self, list, true),
                "setq" => self.compile_setq(list),
                "defvar" => self.compile_defvar(list),
                "if" => crate::codegen::forms::if_expr::compile_if(self, list),
                "progn" | "do" => crate::codegen::forms::control::compile_progn(self, list),
                "when" => crate::codegen::forms::control::compile_when(self, list, false),
                "unless" => crate::codegen::forms::control::compile_when(self, list, true),
                "and" => crate::codegen::forms::control::compile_and(self, list),
                "or" => crate::codegen::forms::control::compile_or(self, list),
                "cond" => crate::codegen::forms::control::compile_cond(self, list),
                "lambda" => crate::codegen::forms::lambda::compile_lambda(self, list),
                "spawn" => crate::codegen::forms::spawn::compile_spawn(self, list),
                "quote" => {
                    if list.len() != 2 {
                        return Err("quote requires exactly one argument".to_string());
                    }
                    self.compile_quoted_value(&list[1])
                }
                op if crate::codegen::COMPILED_BUILTINS.contains(&op) => {
                    crate::codegen::forms::builtins::compile_builtin(self, op, list)
                }
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

    fn compile_indirect_call(
        &mut self,
        func_var_name: &str,
        list: &[Value],
    ) -> Result<IrValue, String> {
        let closure_dlisp_val = self.resolve_variable(func_var_name)?;

        // closure_dlisp_val is a DlispValue* (type_ at offset 0, payload at offset 8).
        // payload.closure_val is a ClosureData* pointer stored at DlispValue offset 8.
        let closure_data_ptr =
            self.builder
                .ins()
                .load(self.ptr_type, MemFlags::new(), closure_dlisp_val, 8);

        // ClosureData layout: { env: *mut DlispValue (offset 0), func_ptr: *const c_void (offset 8) }
        let env_ptr = self
            .builder
            .ins()
            .load(self.ptr_type, MemFlags::new(), closure_data_ptr, 0);
        let func_ptr = self
            .builder
            .ins()
            .load(self.ptr_type, MemFlags::new(), closure_data_ptr, 8);

        let mut args = vec![env_ptr];
        if let Some(meta) = self.global_functions.get(func_var_name) {
            args.extend(self.compile_call_args(&list[1..], meta)?);
        } else {
            for arg in &list[1..] {
                args.push(self.compile_expr(arg)?);
            }
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

    fn compile_quoted_value(&mut self, val: &Value) -> Result<IrValue, String> {
        match val {
            Value::Integer(_) | Value::String(_) => self.compile_expr(val),
            Value::List(list) => {
                if list.is_empty() {
                    return self.make_nil();
                }

                // (car . cdr)
                let car_val = self.compile_quoted_value(&list[0])?;

                // Construct the "rest" as a list or dotted pair?
                // For a proper list, the rest is a list.
                // But quoted list structure is recursive.
                // Value::List is a Vec<Value>.
                // (1 2 3) -> (cons 1 (cons 2 (cons 3 nil)))

                let cdr_val = if list.len() > 1 {
                    let rest = Value::List(list[1..].to_vec());
                    self.compile_quoted_value(&rest)?
                } else {
                    self.make_nil()?
                };

                let func = self
                    .module
                    .declare_func_in_func(self.builtins.funcs.dlisp_make_cons, self.builder.func);
                let call = self.builder.ins().call(func, &[car_val, cdr_val]);
                Ok(self.builder.inst_results(call)[0])
            }
            Value::Symbol(s) => {
                // Define symbol data as string
                let mut hasher = DefaultHasher::new();
                s.hash(&mut hasher);
                let hash = hasher.finish();

                let mut data_ctx = cranelift_module::DataDescription::new();
                let mut bytes = s.clone().into_bytes();
                bytes.push(0); // Null terminator
                data_ctx.define(bytes.into_boxed_slice());

                let ptr_addr = s.as_ptr() as usize;
                let unique_name = format!("sym_{}_{}", hash, ptr_addr);
                let data_id = self
                    .module
                    .declare_data(&unique_name, Linkage::Local, false, false)
                    .map_err(|e| e.to_string())?;

                self.module
                    .define_data(data_id, &data_ctx)
                    .map_err(|e| e.to_string())?;

                let global_val = self.module.declare_data_in_func(data_id, self.builder.func);
                let ptr = self.builder.ins().global_value(self.ptr_type, global_val);

                let func = self
                    .module
                    .declare_func_in_func(self.builtins.funcs.dlisp_make_symbol, self.builder.func);
                let call = self.builder.ins().call(func, &[ptr]);
                Ok(self.builder.inst_results(call)[0])
            }
            Value::Keyword(s) => {
                let mut hasher = DefaultHasher::new();
                s.hash(&mut hasher);
                let hash = hasher.finish();

                let mut data_ctx = cranelift_module::DataDescription::new();
                let mut bytes = s.clone().into_bytes();
                bytes.push(0); // Null terminator
                data_ctx.define(bytes.into_boxed_slice());

                let ptr_addr = s.as_ptr() as usize;
                let unique_name = format!("key_{}_{}", hash, ptr_addr);
                let data_id = self
                    .module
                    .declare_data(&unique_name, Linkage::Local, false, false)
                    .map_err(|e| e.to_string())?;

                self.module
                    .define_data(data_id, &data_ctx)
                    .map_err(|e| e.to_string())?;

                let global_val = self.module.declare_data_in_func(data_id, self.builder.func);
                let ptr = self.builder.ins().global_value(self.ptr_type, global_val);

                let func = self.module.declare_func_in_func(
                    self.builtins.funcs.dlisp_make_keyword,
                    self.builder.func,
                );
                let call = self.builder.ins().call(func, &[ptr]);
                Ok(self.builder.inst_results(call)[0])
            }
            _ => Err(format!("Unsupported quoted value: {:?}", val)),
        }
    }

    fn compile_function_call(&mut self, op: &str, list: &[Value]) -> Result<IrValue, String> {
        let mut args = vec![self.builder.ins().iconst(self.ptr_type, 0)];

        let (sig, linkage) = if let Some(meta) = self.global_functions.get(op) {
            args.extend(self.compile_call_args(&list[1..], meta)?);
            (meta.signature.clone(), Linkage::Export)
        } else {
            for arg in &list[1..] {
                args.push(self.compile_expr(arg)?);
            }

            let mut sig = self.module.make_signature();
            for _ in &args {
                sig.params.push(AbiParam::new(self.ptr_type));
            }
            sig.returns.push(AbiParam::new(self.ptr_type));
            (sig, Linkage::Import)
        };

        let func_id = self
            .module
            .declare_function(op, linkage, &sig)
            .map_err(|e| e.to_string())?;

        let local_func = self.module.declare_func_in_func(func_id, self.builder.func);
        let call = self.builder.ins().call(local_func, &args);
        let result = self.builder.inst_results(call)[0];
        Ok(result)
    }

    pub fn compile_top_level_expr(&mut self, expr: &Value) -> Result<IrValue, String> {
        if let Value::List(list) = expr
            && let Some(Value::Symbol(op)) = list.first()
        {
            return match op.as_str() {
                "defvar" => self.compile_defvar(list),
                "setq" => self.compile_setq(list),
                _ => self.compile_expr(expr),
            };
        }
        self.compile_expr(expr)
    }

    fn compile_call_args(
        &mut self,
        args: &[Value],
        meta: &crate::codegen::FunctionMetadata,
    ) -> Result<Vec<IrValue>, String> {
        if meta.rest_param.is_none() && args.len() != meta.fixed_args.len() {
            return Err(format!(
                "Function expects {} arguments, got {}",
                meta.fixed_args.len(),
                args.len()
            ));
        }

        if meta.rest_param.is_some() && args.len() < meta.fixed_args.len() {
            return Err(format!(
                "Function expects at least {} arguments, got {}",
                meta.fixed_args.len(),
                args.len()
            ));
        }

        let mut compiled =
            Vec::with_capacity(meta.fixed_args.len() + usize::from(meta.rest_param.is_some()));
        for arg in &args[..meta.fixed_args.len()] {
            compiled.push(self.compile_expr(arg)?);
        }

        if meta.rest_param.is_some() {
            compiled.push(self.compile_list_from_exprs(&args[meta.fixed_args.len()..])?);
        }

        Ok(compiled)
    }

    fn compile_list_from_exprs(&mut self, exprs: &[Value]) -> Result<IrValue, String> {
        let mut list_val = self.make_nil()?;
        for expr in exprs.iter().rev() {
            let item = self.compile_expr(expr)?;
            let func = self
                .module
                .declare_func_in_func(self.builtins.funcs.dlisp_make_cons, self.builder.func);
            let call = self.builder.ins().call(func, &[item, list_val]);
            list_val = self.builder.inst_results(call)[0];
        }
        Ok(list_val)
    }

    fn assign_symbol(&mut self, name: &str, val: IrValue) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var) = scope.get(name).copied() {
                self.builder.def_var(var, val);
                return Ok(());
            }
        }

        if let Some(&offset) = self.captured_vars.get(name)
            && let Some(env) = self.env_param
        {
            self.builder
                .ins()
                .store(MemFlags::new(), val, env, offset as i32);
            return Ok(());
        }

        if let Some(data_id) = self.global_variables.get(name) {
            let global_val = self
                .module
                .declare_data_in_func(*data_id, self.builder.func);
            let addr = self.builder.ins().global_value(self.ptr_type, global_val);
            self.builder.ins().store(MemFlags::new(), val, addr, 0);
            return Ok(());
        }

        Err(format!("Undefined symbol: {}", name))
    }

    fn compile_setq(&mut self, list: &[Value]) -> Result<IrValue, String> {
        let args = &list[1..];
        if args.is_empty() || !args.len().is_multiple_of(2) {
            return Err("setq requires pairs of (symbol value) arguments".to_string());
        }

        let mut last_val = self.make_nil()?;
        for i in (0..args.len()).step_by(2) {
            let symbol = match &args[i] {
                Value::Symbol(s) => s,
                _ => return Err("setq even arguments must be symbols".to_string()),
            };
            let val = self.compile_expr(&args[i + 1])?;
            self.assign_symbol(symbol, val)?;
            last_val = val;
        }
        Ok(last_val)
    }

    fn compile_defvar(&mut self, list: &[Value]) -> Result<IrValue, String> {
        let args = &list[1..];
        if args.is_empty() || args.len() > 3 {
            return Err(
                "defvar requires 1 to 3 arguments (symbol, [init-value, [doc-string]])".to_string(),
            );
        }

        let symbol_name = match &args[0] {
            Value::Symbol(s) => s,
            _ => return Err("defvar first argument must be a symbol".to_string()),
        };

        if args.len() >= 2 {
            let data_id = *self
                .global_variables
                .get(symbol_name)
                .ok_or_else(|| format!("Undefined symbol: {}", symbol_name))?;

            let global_val = self.module.declare_data_in_func(data_id, self.builder.func);
            let addr = self.builder.ins().global_value(self.ptr_type, global_val);
            let current = self
                .builder
                .ins()
                .load(self.ptr_type, MemFlags::new(), addr, 0);
            let zero = self.builder.ins().iconst(self.ptr_type, 0);
            let is_unbound = self.builder.ins().icmp(IntCC::Equal, current, zero);

            let store_block = self.builder.create_block();
            let cont_block = self.builder.create_block();
            self.builder
                .ins()
                .brif(is_unbound, store_block, &[], cont_block, &[]);

            self.builder.switch_to_block(store_block);
            let init_val = self.compile_expr(&args[1])?;
            self.builder.ins().store(MemFlags::new(), init_val, addr, 0);
            self.builder.ins().jump(cont_block, &[]);
            self.builder.seal_block(store_block);

            self.builder.switch_to_block(cont_block);
            self.builder.seal_block(cont_block);
        }

        self.compile_quoted_value(&Value::Symbol(symbol_name.clone()))
    }
}
