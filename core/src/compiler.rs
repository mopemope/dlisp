use crate::ast::Value;
use crate::codegen::CodeGen;
use crate::environment::Environment;
use crate::interpreter::{Interpreter, default_env};
use cranelift::prelude::{Configurable, settings};
use cranelift_module::default_libcall_names;
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::cell::RefCell;
use std::rc::Rc;

pub enum OptimizationLevel {
    None,
    Speed,
    SpeedAndSize,
}

pub struct CompilerOptions {
    pub optimization_level: OptimizationLevel,
    pub enable_verifier: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            optimization_level: OptimizationLevel::None,
            enable_verifier: true,
        }
    }
}

pub struct AOTCompiler {
    codegen: CodeGen,
    module: ObjectModule,
    interpreter: Interpreter,
    env: Rc<RefCell<Environment>>,
}

impl AOTCompiler {
    pub fn new() -> Self {
        Self::with_options(CompilerOptions::default())
    }

    pub fn with_options(options: CompilerOptions) -> Self {
        let mut flag_builder = settings::builder();

        // Enable verifier
        if options.enable_verifier {
            flag_builder.set("enable_verifier", "true").unwrap();
        } else {
            flag_builder.set("enable_verifier", "false").unwrap();
        }

        // Optimization level
        match options.optimization_level {
            OptimizationLevel::None => {
                flag_builder.set("opt_level", "none").unwrap();
            }
            OptimizationLevel::Speed => {
                flag_builder.set("opt_level", "speed").unwrap();
            }
            OptimizationLevel::SpeedAndSize => {
                flag_builder.set("opt_level", "speed_and_size").unwrap();
            }
        }

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
            interpreter: Interpreter::new(),
            env: default_env(),
        }
    }

    pub async fn compile(mut self, ast: Vec<Value>) -> Result<Vec<u8>, String> {
        let mut has_main = false;
        let mut expanded_ast = Vec::new();

        #[allow(clippy::collapsible_if)]
        for expr in ast {
            // Expand macros first!
            let expanded = self
                .interpreter
                .expand(expr, &mut self.env)
                .await
                .map_err(|e| format!("Macro expansion error: {}", e))?;

            // If it's a defmacro, we EVALUATE it in the compiler's interpreter so subsequent code can use it.
            // But we do NOT emit code for it (macros are compile-time).
            if let Value::List(ref l) = expanded {
                if let Some(Value::Symbol(s)) = l.first() {
                    if s == "defmacro" {
                        self.interpreter
                            .eval(expanded, &mut self.env)
                            .await
                            .map_err(|e| format!("Macro definition error: {}", e))?;
                        continue;
                    }
                }
            }

            expanded_ast.push(expanded);
        }

        for expr in &expanded_ast {
            if let Value::List(l) = expr
                && let Some(Value::Symbol(s)) = l.first()
            {
                match s.as_str() {
                    "defvar" => {
                        if let Some(Value::Symbol(name)) = l.get(1) {
                            self.codegen
                                .declare_global_variable(&mut self.module, name)?;
                        }
                    }
                    "setq" => {
                        for idx in (1..l.len()).step_by(2) {
                            if let Some(Value::Symbol(name)) = l.get(idx) {
                                self.codegen
                                    .declare_global_variable(&mut self.module, name)?;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut init_exprs = Vec::new();
        for expr in expanded_ast {
            if let Value::List(ref l) = expr
                && let Some(Value::Symbol(s)) = l.first()
                && s == "defun"
            {
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
                let mut rest_param = None;
                let mut idx = 0;
                while idx < args_list.len() {
                    match &args_list[idx] {
                        Value::Symbol(n) if n == "&rest" => {
                            if idx + 1 >= args_list.len() {
                                return Err("&rest requires a parameter name".to_string());
                            }
                            match &args_list[idx + 1] {
                                Value::Symbol(rest_name) => {
                                    rest_param = Some(rest_name.clone());
                                }
                                _ => {
                                    return Err("&rest parameter must be a symbol".to_string());
                                }
                            }
                            if idx + 2 < args_list.len() {
                                return Err("&rest parameter must be last in the parameter list"
                                    .to_string());
                            }
                            break;
                        }
                        Value::Symbol(n) => arg_names.push(n.clone()),
                        _ => return Err("defun arg must be a symbol".to_string()),
                    }
                    idx += 1;
                }

                if name == "main" {
                    name = "dlisp_user_main".to_string();
                    has_main = true;
                }

                self.codegen.compile_with_rest(
                    &mut self.module,
                    &name,
                    &arg_names,
                    rest_param,
                    body,
                )?;
                continue;
            }

            init_exprs.push(expr);
        }

        let has_init = !init_exprs.is_empty();
        if has_init {
            self.codegen.compile_top_level_init(
                &mut self.module,
                "dlisp_user_init",
                &init_exprs,
            )?;
        }

        if has_main || has_init {
            self.codegen.compile_program_entry(
                &mut self.module,
                "dlisp_user_entry",
                has_init.then_some("dlisp_user_init"),
                has_main.then_some("dlisp_user_main"),
            )?;
            self.codegen.compile_entry_point(
                &mut self.module,
                has_main.then_some("dlisp_user_main"),
                has_init.then_some("dlisp_user_init"),
            )?;
        }

        let product = self.module.finish();
        let bytes = product.emit().map_err(|e| e.to_string())?;
        Ok(bytes)
    }
}

impl Default for AOTCompiler {
    fn default() -> Self {
        Self::with_options(CompilerOptions::default())
    }
}
