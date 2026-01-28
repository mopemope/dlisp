use crate::ast::Value;
use crate::codegen::CodeGen;
use cranelift_jit::{JITBuilder, JITModule};

#[allow(dead_code)]
pub struct JIT {
    codegen: CodeGen,
    module: JITModule,
}

impl Default for JIT {
    fn default() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
        let module = JITModule::new(builder);

        Self {
            codegen: CodeGen::new(),
            module,
        }
    }
}

impl JIT {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compile(
        &mut self,
        name: &str,
        args: &[String],
        body: &[Value],
    ) -> Result<*const u8, String> {
        let id = self.codegen.compile(&mut self.module, name, args, body)?;

        self.module
            .finalize_definitions()
            .map_err(|e| e.to_string())?;

        let code = self.module.get_finalized_function(id);
        Ok(code)
    }

    pub fn compile_hello(&mut self) -> Result<fn() -> i64, String> {
        let body = vec![Value::Integer(42)];
        let id = self
            .codegen
            .compile(&mut self.module, "hello", &[], &body)?;

        self.module
            .finalize_definitions()
            .map_err(|e| e.to_string())?;

        let code = self.module.get_finalized_function(id);
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
