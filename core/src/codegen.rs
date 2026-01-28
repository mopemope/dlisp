use crate::ast::Value;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};
use std::collections::HashMap;

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
        };

        // Map arg names to variables
        let mut variables = HashMap::new();
        for (i, arg_name) in args.iter().enumerate() {
            let val = builder.block_params(entry_block)[i];
            let var = builder.declare_var(int);
            builder.def_var(var, val);
            variables.insert(arg_name.clone(), var);
        }

        let mut result_val = builder.ins().iconst(int, 0);

        {
            let mut trans_ctx = FunctionTranslationContext {
                builder: &mut builder,
                module,
                builtins: &builtins,
                variables: &variables,
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

    // Clear context helper needs access to module
    fn module_clear_context<M: Module>(&mut self, module: &mut M) {
        module.clear_context(&mut self.ctx);
    }
}

struct FunctionTranslationContext<'a, 'func, M: Module> {
    builder: &'a mut FunctionBuilder<'func>,
    module: &'a mut M,
    builtins: &'a Builtins,
    variables: &'a HashMap<String, Variable>,
    ptr_type: Type,
}

impl<'a, 'func, M: Module> FunctionTranslationContext<'a, 'func, M> {
    fn compile_expr(&mut self, val: &Value) -> Result<IrValue, String> {
        match val {
            Value::Integer(n) => Ok(self.builder.ins().iconst(self.ptr_type, *n)),
            Value::Symbol(s) => {
                if let Some(var) = self.variables.get(s) {
                    Ok(self.builder.use_var(*var))
                } else {
                    Err(format!(
                        "Undefined variable (or not supported in JIT): {}",
                        s
                    ))
                }
            }
            Value::List(list) => self.compile_list(list),
            _ => Err(format!("Unsupported Value type for JIT: {:?}", val)),
        }
    }

    fn compile_list(&mut self, list: &[Value]) -> Result<IrValue, String> {
        if list.is_empty() {
            return Ok(self.builder.ins().iconst(self.ptr_type, 0)); // nil?
        }
        if let Value::Symbol(ref op) = list[0] {
            // Check if 'op' is a variable (parameter) - Indirect call logic not implemented
            if self.variables.contains_key(op) {
                return Err(format!(
                    "JIT does not support indirect function calls (variable '{}')",
                    op
                ));
            }

            // Builtins and Ops
            match op.as_str() {
                "print" | "+" | "-" | "*" => self.compile_builtin(op, list),
                _ => self.compile_function_call(op, list),
            }
        } else {
            Err("JIT function calls not supported yet".to_string())
        }
    }

    fn compile_builtin(&mut self, op: &str, list: &[Value]) -> Result<IrValue, String> {
        match op {
            "print" => {
                if list.len() != 2 {
                    return Err("print takes 1 arg".to_string());
                }
                let arg_val = self.compile_expr(&list[1])?;

                // Prepare verification of function declaration
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
        // Function call
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
