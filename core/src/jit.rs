use crate::ast::Value;
use cranelift::prelude::{Value as IrValue, *};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, Linkage, Module};
use std::collections::HashMap;

#[allow(dead_code)]
pub struct JIT {
    /// The function builder context, which is reused across multiple
    /// function compilations.
    builder_context: FunctionBuilderContext,

    /// The main Cranelift context, which contains the state for compiling
    /// a single function.
    ctx: codegen::Context,

    /// The data context, which contains the state for defining data
    /// objects.
    data_ctx: DataDescription,

    /// The module, with the jit backend, which manages the JIT'd
    /// functions.
    module: JITModule,
}

impl Default for JIT {
    fn default() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();

        // Symbol lookup needed for resolving native functions?
        // builder.symbol("...", ...);

        let module = JITModule::new(builder);

        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_ctx: DataDescription::new(),
            module,
        }
    }
}

impl JIT {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compile_hello(&mut self) -> Result<fn() -> i64, String> {
        let mut ctx = self.module.make_context();
        let mut func_ctx = FunctionBuilderContext::new();

        let int = self.module.target_config().pointer_type();
        ctx.func.signature.returns.push(AbiParam::new(int));

        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);

        // Return 42
        let val = builder.ins().iconst(int, 42);
        builder.ins().return_(&[val]);

        builder.seal_all_blocks();
        builder.finalize();

        let id = self
            .module
            .declare_function("hello", Linkage::Export, &ctx.func.signature)
            .map_err(|e| e.to_string())?;

        self.module
            .define_function(id, &mut ctx)
            .map_err(|e| e.to_string())?;

        self.module.clear_context(&mut ctx);
        self.module
            .finalize_definitions()
            .map_err(|e| e.to_string())?;

        let code = self.module.get_finalized_function(id);

        // unsafe call
        Ok(unsafe { std::mem::transmute::<_, fn() -> i64>(code) })
    }

    pub fn compile(
        &mut self,
        name: &str,
        args: &[String],
        body: &[Value],
    ) -> Result<*const u8, String> {
        let mut ctx = self.module.make_context();
        let mut func_ctx = FunctionBuilderContext::new();

        let int = self.module.target_config().pointer_type();
        // Assume all args and return are int for now
        for _ in args {
            ctx.func.signature.params.push(AbiParam::new(int));
        }
        ctx.func.signature.returns.push(AbiParam::new(int));

        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);

        // Map arg names to variables
        let mut variables = HashMap::new();
        for (i, arg_name) in args.iter().enumerate() {
            let val = builder.block_params(entry_block)[i];
            let var = Variable::new(i);
            builder.declare_var(var, int);
            builder.def_var(var, val);
            variables.insert(arg_name.clone(), var);
        }

        let mut result_val = builder.ins().iconst(int, 0);
        for expr in body {
            result_val = self.compile_expr(&mut builder, expr, &variables, int)?;
        }

        builder.ins().return_(&[result_val]);

        builder.seal_all_blocks();
        builder.finalize();

        let id = self
            .module
            .declare_function(name, Linkage::Export, &ctx.func.signature)
            .map_err(|e| e.to_string())?;

        self.module
            .define_function(id, &mut ctx)
            .map_err(|e| e.to_string())?;

        self.module.clear_context(&mut ctx);
        self.module
            .finalize_definitions()
            .map_err(|e| e.to_string())?;

        let code = self.module.get_finalized_function(id);
        Ok(code)
    }

    fn compile_expr(
        &self,
        builder: &mut FunctionBuilder,
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
                    // Binary ops
                    if list.len() == 3 {
                        let lhs = self.compile_expr(builder, &list[1], variables, int)?;
                        let rhs = self.compile_expr(builder, &list[2], variables, int)?;
                        match op.as_str() {
                            "+" => Ok(builder.ins().iadd(lhs, rhs)),
                            "-" => Ok(builder.ins().isub(lhs, rhs)),
                            "*" => Ok(builder.ins().imul(lhs, rhs)),
                            _ => Err(format!("Unsupported JIT operator: {}", op)),
                        }
                    } else {
                        Err(format!(
                            "JIT only supports binary ops context for now: {}",
                            op
                        ))
                    }
                } else {
                    Err("JIT function calls not supported yet".to_string())
                }
            }
            _ => Err(format!("Unsupported Value type for JIT: {:?}", val)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_hello() {
        let mut jit = JIT::new();
        let code_ptr = jit.compile_hello().unwrap();
        let result = code_ptr();
        assert_eq!(result, 42);
    }
}
