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
        #[allow(unused_mut)]
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();

        // Register runtime symbols for JIT to work in-process.
        // We resolve these dynamically to avoid a hard dependency on the runtime crate in the core library.
        let symbols = [
            "dlisp_make_int",
            "dlisp_make_float",
            "dlisp_make_bool",
            "dlisp_make_nil",
            "dlisp_make_string",
            "dlisp_make_symbol",
            "dlisp_make_cons",
            "dlisp_car",
            "dlisp_cdr",
            "dlisp_print",
            "dlisp_add",
            "dlisp_sub",
            "dlisp_mul",
            "dlisp_gt",
            "dlisp_is_truthy",
            "dlisp_spawn",
            "dlisp_sleep",
            "dlisp_gc_malloc",
        ];

        unsafe extern "C" {
            fn dlsym(
                handle: *mut std::ffi::c_void,
                symbol: *const std::ffi::c_char,
            ) -> *mut std::ffi::c_void;
        }

        let rtld_default = std::ptr::null_mut(); // RTLD_DEFAULT on Linux/most POSIX
        for name in symbols {
            let c_name = std::ffi::CString::new(name).unwrap();
            let addr = unsafe { dlsym(rtld_default, c_name.as_ptr()) };
            if !addr.is_null() {
                builder.symbol(name, addr as *const u8);
            }
        }

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
        // SAFETY: The compiled code matches the signature fn() -> i64
        Ok(unsafe { std::mem::transmute::<*const u8, fn() -> i64>(code) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_jit_hello() {
        let mut jit = JIT::new();
        let code_ptr = jit.compile_hello().unwrap();
        let result = code_ptr();
        assert_eq!(result, 42);
    }
}
