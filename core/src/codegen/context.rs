use crate::ast::Value;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_module::{FuncId, Linkage, Module};
use std::collections::HashMap;

pub struct Builtins {
    pub printf: FuncId,
    pub printf_fmt: IrValue,
    pub dlisp_spawn: FuncId,
    pub dlisp_sleep: FuncId,
    pub gc_malloc: FuncId,
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
    pub global_signatures: &'a HashMap<String, Signature>,
}

impl<'a, 'func, M: Module> FunctionTranslationContext<'a, 'func, M> {
    pub fn compile_expr(&mut self, val: &Value) -> Result<IrValue, String> {
        match val {
            Value::Integer(n) => Ok(self.builder.ins().iconst(self.ptr_type, *n)),
            Value::Symbol(s) => self.resolve_variable(s),
            Value::List(list) => self.compile_list(list),
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
        false
    }

    pub fn compile_list(&mut self, list: &[Value]) -> Result<IrValue, String> {
        if list.is_empty() {
            return Ok(self.builder.ins().iconst(self.ptr_type, 0)); // nil?
        }

        if let Value::Symbol(ref op) = list[0] {
            match op.as_str() {
                "let" => crate::codegen::forms::let_expr::compile_let(self, list),
                "if" => crate::codegen::forms::if_expr::compile_if(self, list),
                "lambda" => crate::codegen::forms::lambda::compile_lambda(self, list),
                "spawn" => crate::codegen::forms::spawn::compile_spawn(self, list),
                "print" | "+" | "-" | "*" | "sleep" | ">" => {
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
