use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, Linkage, Module};

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
