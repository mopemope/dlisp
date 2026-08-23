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

        // Register runtime symbols directly so tests and the CLI see the same ABI.
        builder.symbol("dlisp_make_int", dlisp_runtime::dlisp_make_int as *const u8);
        builder.symbol(
            "dlisp_make_float",
            dlisp_runtime::dlisp_make_float as *const u8,
        );
        builder.symbol(
            "dlisp_make_bool",
            dlisp_runtime::dlisp_make_bool as *const u8,
        );
        builder.symbol("dlisp_make_nil", dlisp_runtime::dlisp_make_nil as *const u8);
        builder.symbol(
            "dlisp_make_string",
            dlisp_runtime::dlisp_make_string as *const u8,
        );
        builder.symbol(
            "dlisp_make_symbol",
            dlisp_runtime::dlisp_make_symbol as *const u8,
        );
        builder.symbol(
            "dlisp_make_keyword",
            dlisp_runtime::dlisp_make_keyword as *const u8,
        );
        builder.symbol(
            "dlisp_make_cons",
            dlisp_runtime::dlisp_make_cons as *const u8,
        );
        builder.symbol("dlisp_cons", dlisp_runtime::dlisp_cons as *const u8);
        builder.symbol(
            "dlisp_make_closure",
            dlisp_runtime::dlisp_make_closure as *const u8,
        );
        builder.symbol("dlisp_make_map", dlisp_runtime::dlisp_make_map as *const u8);
        builder.symbol(
            "dlisp_map_assoc",
            dlisp_runtime::dlisp_map_assoc as *const u8,
        );
        builder.symbol("dlisp_map_get", dlisp_runtime::dlisp_map_get as *const u8);
        builder.symbol("dlisp_keys", dlisp_runtime::dlisp_keys as *const u8);
        builder.symbol("dlisp_get", dlisp_runtime::dlisp_get as *const u8);
        builder.symbol("dlisp_car", dlisp_runtime::dlisp_car as *const u8);
        builder.symbol("dlisp_cdr", dlisp_runtime::dlisp_cdr as *const u8);
        builder.symbol(
            "dlisp_vector_to_list",
            dlisp_runtime::lists::dlisp_vector_to_list as *const u8,
        );
        builder.symbol("dlisp_print", dlisp_runtime::dlisp_print as *const u8);
        builder.symbol("dlisp_add", dlisp_runtime::dlisp_add as *const u8);
        builder.symbol("dlisp_sub", dlisp_runtime::dlisp_sub as *const u8);
        builder.symbol("dlisp_mul", dlisp_runtime::dlisp_mul as *const u8);
        builder.symbol("dlisp_gt", dlisp_runtime::dlisp_gt as *const u8);
        builder.symbol("dlisp_lt", dlisp_runtime::dlisp_lt as *const u8);
        builder.symbol("dlisp_eq", dlisp_runtime::dlisp_eq as *const u8);
        builder.symbol(
            "dlisp_is_truthy",
            dlisp_runtime::dlisp_is_truthy as *const u8,
        );
        builder.symbol("dlisp_spawn", dlisp_runtime::dlisp_spawn as *const u8);
        builder.symbol("dlisp_sleep", dlisp_runtime::dlisp_sleep as *const u8);
        builder.symbol(
            "dlisp_gc_malloc",
            dlisp_runtime::dlisp_gc_malloc as *const u8,
        );
        builder.symbol("dlisp_div", dlisp_runtime::dlisp_div as *const u8);
        builder.symbol("dlisp_mod", dlisp_runtime::dlisp_mod as *const u8);
        builder.symbol("dlisp_gte", dlisp_runtime::dlisp_gte as *const u8);
        builder.symbol("dlisp_lte", dlisp_runtime::dlisp_lte as *const u8);
        builder.symbol("dlisp_neq", dlisp_runtime::dlisp_neq as *const u8);
        builder.symbol("dlisp_str", dlisp_runtime::dlisp_str as *const u8);
        builder.symbol(
            "dlisp_string_length",
            dlisp_runtime::dlisp_string_length as *const u8,
        );
        builder.symbol(
            "dlisp_substring",
            dlisp_runtime::dlisp_substring as *const u8,
        );
        builder.symbol(
            "dlisp_string_append",
            dlisp_runtime::dlisp_string_append as *const u8,
        );
        builder.symbol("dlisp_nil_p", dlisp_runtime::dlisp_nil_p as *const u8);
        builder.symbol("dlisp_list_p", dlisp_runtime::dlisp_list_p as *const u8);
        builder.symbol("dlisp_number_p", dlisp_runtime::dlisp_number_p as *const u8);
        builder.symbol("dlisp_string_p", dlisp_runtime::dlisp_string_p as *const u8);
        builder.symbol("dlisp_symbol_p", dlisp_runtime::dlisp_symbol_p as *const u8);
        builder.symbol(
            "dlisp_keyword_p",
            dlisp_runtime::dlisp_keyword_p as *const u8,
        );
        builder.symbol("dlisp_map_p", dlisp_runtime::dlisp_map_p as *const u8);
        builder.symbol("dlisp_vector_p", dlisp_runtime::dlisp_vector_p as *const u8);
        builder.symbol(
            "dlisp_append",
            dlisp_runtime::lists::dlisp_append as *const u8,
        );
        builder.symbol(
            "dlisp_reverse",
            dlisp_runtime::lists::dlisp_reverse as *const u8,
        );
        builder.symbol("dlisp_last", dlisp_runtime::lists::dlisp_last as *const u8);
        builder.symbol(
            "dlisp_butlast",
            dlisp_runtime::lists::dlisp_butlast as *const u8,
        );
        builder.symbol(
            "dlisp_flatten",
            dlisp_runtime::lists::dlisp_flatten as *const u8,
        );
        builder.symbol("dlisp_take", dlisp_runtime::lists::dlisp_take as *const u8);
        builder.symbol("dlisp_drop", dlisp_runtime::lists::dlisp_drop as *const u8);
        builder.symbol(
            "dlisp_is_empty",
            dlisp_runtime::predicates::dlisp_is_empty as *const u8,
        );
        builder.symbol(
            "dlisp_string_split",
            dlisp_runtime::strings::dlisp_string_split as *const u8,
        );
        builder.symbol(
            "dlisp_string_replace",
            dlisp_runtime::strings::dlisp_string_replace as *const u8,
        );
        builder.symbol(
            "dlisp_string_upper",
            dlisp_runtime::strings::dlisp_string_upper as *const u8,
        );
        builder.symbol(
            "dlisp_string_lower",
            dlisp_runtime::strings::dlisp_string_lower as *const u8,
        );
        builder.symbol(
            "dlisp_string_trim",
            dlisp_runtime::strings::dlisp_string_trim as *const u8,
        );
        builder.symbol(
            "dlisp_string_trim_left",
            dlisp_runtime::strings::dlisp_string_trim_left as *const u8,
        );
        builder.symbol(
            "dlisp_string_trim_right",
            dlisp_runtime::strings::dlisp_string_trim_right as *const u8,
        );
        builder.symbol(
            "dlisp_string_starts_with",
            dlisp_runtime::strings::dlisp_string_starts_with as *const u8,
        );
        builder.symbol(
            "dlisp_string_ends_with",
            dlisp_runtime::strings::dlisp_string_ends_with as *const u8,
        );
        builder.symbol(
            "dlisp_string_contains",
            dlisp_runtime::strings::dlisp_string_contains as *const u8,
        );
        builder.symbol(
            "dlisp_string_index_of",
            dlisp_runtime::strings::dlisp_string_index_of as *const u8,
        );
        builder.symbol(
            "dlisp_string_to_number",
            dlisp_runtime::strings::dlisp_string_to_number as *const u8,
        );
        builder.symbol(
            "dlisp_number_to_string",
            dlisp_runtime::strings::dlisp_number_to_string as *const u8,
        );
        builder.symbol(
            "dlisp_char_at",
            dlisp_runtime::strings::dlisp_char_at as *const u8,
        );
        builder.symbol("dlisp_type_of", dlisp_runtime::dlisp_type_of as *const u8);
        builder.symbol(
            "dlisp_make_vector",
            dlisp_runtime::vectors::dlisp_make_vector as *const u8,
        );
        builder.symbol(
            "dlisp_vector_push",
            dlisp_runtime::vectors::dlisp_vector_push as *const u8,
        );
        builder.symbol(
            "dlisp_vector_get",
            dlisp_runtime::vectors::dlisp_vector_get as *const u8,
        );
        builder.symbol(
            "dlisp_vector_count",
            dlisp_runtime::vectors::dlisp_vector_count as *const u8,
        );
        builder.symbol(
            "dlisp_vector_copy",
            dlisp_runtime::vectors::dlisp_vector_copy as *const u8,
        );
        builder.symbol("dlisp_conj", dlisp_runtime::dlisp_conj as *const u8);
        builder.symbol(
            "dlisp_map",
            dlisp_runtime::higher_order::dlisp_map as *const u8,
        );
        builder.symbol(
            "dlisp_filter",
            dlisp_runtime::higher_order::dlisp_filter as *const u8,
        );
        builder.symbol(
            "dlisp_reduce",
            dlisp_runtime::higher_order::dlisp_reduce as *const u8,
        );
        builder.symbol(
            "dlisp_some",
            dlisp_runtime::higher_order::dlisp_some as *const u8,
        );
        builder.symbol(
            "dlisp_every",
            dlisp_runtime::higher_order::dlisp_every as *const u8,
        );
        builder.symbol(
            "dlisp_find",
            dlisp_runtime::higher_order::dlisp_find as *const u8,
        );
        builder.symbol(
            "dlisp_for_each",
            dlisp_runtime::higher_order::dlisp_for_each as *const u8,
        );
        builder.symbol(
            "dlisp_file_exists",
            dlisp_runtime::io::dlisp_file_exists as *const u8,
        );
        builder.symbol("dlisp_is_dir", dlisp_runtime::io::dlisp_is_dir as *const u8);
        builder.symbol(
            "dlisp_is_file",
            dlisp_runtime::io::dlisp_is_file as *const u8,
        );
        builder.symbol(
            "dlisp_delete_file",
            dlisp_runtime::io::dlisp_delete_file as *const u8,
        );
        builder.symbol(
            "dlisp_list_dir",
            dlisp_runtime::io::dlisp_list_dir as *const u8,
        );
        builder.symbol(
            "dlisp_getenv",
            dlisp_runtime::sys::dlisp_getenv as *const u8,
        );
        builder.symbol(
            "dlisp_setenv",
            dlisp_runtime::sys::dlisp_setenv as *const u8,
        );
        builder.symbol("dlisp_cwd", dlisp_runtime::sys::dlisp_cwd as *const u8);
        builder.symbol(
            "dlisp_set_cwd",
            dlisp_runtime::sys::dlisp_set_cwd as *const u8,
        );
        builder.symbol("dlisp_args", dlisp_runtime::sys::dlisp_args as *const u8);
        builder.symbol("dlisp_exit", dlisp_runtime::sys::dlisp_exit as *const u8);
        builder.symbol("dlisp_sh", dlisp_runtime::os::dlisp_sh as *const u8);

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
        self.compile_with_rest(name, args, None, body)
    }

    pub fn compile_with_rest(
        &mut self,
        name: &str,
        args: &[String],
        rest_param: Option<String>,
        body: &[Value],
    ) -> Result<*const u8, String> {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let id =
                self.codegen
                    .compile_with_rest(&mut self.module, name, args, rest_param, body)?;

            self.module
                .finalize_definitions()
                .map_err(|e| e.to_string())?;

            Ok::<*const u8, String>(self.module.get_finalized_function(id))
        }));

        match result {
            Ok(res) => res,
            Err(_) => Err("JIT compilation failed".to_string()),
        }
    }

    pub fn register_signature(&mut self, name: &str, args: &[String], rest_param: Option<String>) {
        self.codegen
            .register_function_metadata(&mut self.module, name, args, rest_param);
    }

    pub fn compile_batch_with_rest(
        &mut self,
        defs: &[crate::ast::FuncDef],
    ) -> Result<Vec<(String, *const u8)>, String> {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut ids = Vec::with_capacity(defs.len());
            for (name, args, rest_param, body) in defs {
                let id = self.codegen.compile_with_rest(
                    &mut self.module,
                    name,
                    args,
                    rest_param.clone(),
                    body,
                )?;
                ids.push((name.clone(), id));
            }

            self.module
                .finalize_definitions()
                .map_err(|e| e.to_string())?;

            Ok::<Vec<(String, *const u8)>, String>(
                ids.into_iter()
                    .map(|(name, id)| (name, self.module.get_finalized_function(id)))
                    .collect(),
            )
        }));

        match result {
            Ok(res) => res,
            Err(_) => Err("JIT batch compilation failed".to_string()),
        }
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
