use crate::ast::Value;
use crate::codegen::CodeGen;
use cranelift::prelude::{Configurable, settings};
use cranelift_module::default_libcall_names;
use cranelift_object::{ObjectBuilder, ObjectModule};

pub struct AOTCompiler {
    codegen: CodeGen,
    module: ObjectModule,
}

impl Default for AOTCompiler {
    fn default() -> Self {
        let mut flag_builder = settings::builder();
        // Enable verifier
        flag_builder.set("enable_verifier", "true").unwrap();
        // use default ISA
        let isa_builder = cranelift_native::builder().expect("host machine is not supported");
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .expect("Failed to create ISA");

        let builder = ObjectBuilder::new(isa, "dlisp_program", default_libcall_names())
            .expect("Falid to create ObjectBuilder");
        let module = ObjectModule::new(builder);

        Self {
            codegen: CodeGen::new(),
            module,
        }
    }
}

impl AOTCompiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compile(mut self, ast: Vec<Value>) -> Result<Vec<u8>, String> {
        let mut has_main = false;

        #[allow(clippy::collapsible_if)]
        for expr in ast {
            if let Value::List(ref l) = expr {
                if let Some(Value::Symbol(s)) = l.first() {
                    if s == "defun" {
                        // (defun name (args) body...)
                        if l.len() < 3 {
                            return Err("defun requires at least 3 arguments".to_string());
                        }
                        let mut name = match &l[1] {
                            Value::Symbol(n) => n.clone(),
                            _ => return Err("defun name must be a symbol".to_string()),
                        };
                        let args_list = match &l[2] {
                            Value::List(args) => args,
                            _ => return Err("defun args must be a list".to_string()),
                        };
                        let body = &l[3..];

                        let mut arg_names = Vec::new();
                        for arg in args_list {
                            match arg {
                                Value::Symbol(n) => arg_names.push(n.clone()),
                                _ => return Err("defun arg must be a symbol".to_string()),
                            }
                        }

                        if name == "main" {
                            name = "dlisp_user_main".to_string();
                            has_main = true;
                        }

                        self.codegen
                            .compile(&mut self.module, &name, &arg_names, body)?;
                    }
                }
            }
        }

        if has_main {
            self.codegen
                .compile_entry_point(&mut self.module, "dlisp_user_main")?;
        }

        let product = self.module.finish();
        let bytes = product.emit().map_err(|e| e.to_string())?;
        Ok(bytes)
    }
}
