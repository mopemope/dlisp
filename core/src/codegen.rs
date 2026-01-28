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
        for expr in body {
            result_val = compile_expr(&mut builder, module, &builtins, expr, &variables, int)?;
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

fn compile_expr<M: Module>(
    builder: &mut FunctionBuilder,
    module: &mut M,
    builtins: &Builtins,
    val: &Value,
    variables: &HashMap<String, Variable>,
    int: Type,
) -> Result<IrValue, String> {
    match val {
        Value::Integer(n) => Ok(builder.ins().iconst(int, *n)),
        Value::Symbol(s) => {
            if let Some(var) = variables.get(s) {
                Ok(builder.use_var(*var))
            } else {
                Err(format!(
                    "Undefined variable (or not supported in JIT): {}",
                    s
                ))
            }
        }
        Value::List(list) => {
            if list.is_empty() {
                return Ok(builder.ins().iconst(int, 0)); // nil?
            }
            if let Value::Symbol(ref op) = list[0] {
                // Builtins and Ops
                match op.as_str() {
                    "print" => {
                        if list.len() != 2 {
                            return Err("print takes 1 arg".to_string());
                        }
                        let arg_val =
                            compile_expr(builder, module, builtins, &list[1], variables, int)?;

                        // Prepare verification of function declaration
                        let local_printf =
                            module.declare_func_in_func(builtins.printf, builder.func);

                        builder
                            .ins()
                            .call(local_printf, &[builtins.printf_fmt, arg_val]);
                        Ok(arg_val)
                    }
                    "+" | "-" | "*" => {
                        if list.len() == 3 {
                            let lhs =
                                compile_expr(builder, module, builtins, &list[1], variables, int)?;
                            let rhs =
                                compile_expr(builder, module, builtins, &list[2], variables, int)?;
                            match op.as_str() {
                                "+" => Ok(builder.ins().iadd(lhs, rhs)),
                                "-" => Ok(builder.ins().isub(lhs, rhs)),
                                "*" => Ok(builder.ins().imul(lhs, rhs)),
                                _ => unreachable!(),
                            }
                        } else {
                            Err(format!("Binary ops require 2 args: {}", op))
                        }
                    }
                    _ => {
                        // Function call
                        let mut args = Vec::new();
                        for arg in &list[1..] {
                            args.push(compile_expr(
                                builder, module, builtins, arg, variables, int,
                            )?);
                        }

                        let mut sig = module.make_signature();
                        for _ in &args {
                            sig.params.push(AbiParam::new(int));
                        }
                        sig.returns.push(AbiParam::new(int));

                        let func_id = module
                            .declare_function(op, Linkage::Export, &sig)
                            .map_err(|e| e.to_string())?;

                        let local_func = module.declare_func_in_func(func_id, builder.func);
                        let call = builder.ins().call(local_func, &args);
                        let result = builder.inst_results(call)[0];
                        Ok(result)
                    }
                }
            } else {
                Err("JIT function calls not supported yet".to_string())
            }
        }
        _ => Err(format!("Unsupported Value type for JIT: {:?}", val)),
    }
}
