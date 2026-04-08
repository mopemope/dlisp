use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use dlisp_core::ast::Value;
use dlisp_core::codegen::builtins::declare_builtins;
use dlisp_core::codegen::context::{Builtins, FunctionTranslationContext};
use std::collections::HashMap;

// Dummy runtime stubs for linking - these won't be called, just resolve symbols
extern "C" fn dummy_ptr(_: *mut u64) -> *mut u64 {
    std::ptr::null_mut::<u64>()
}
extern "C" fn dummy_ptr2(_: *mut u64, _: *mut u64) -> *mut u64 {
    std::ptr::null_mut::<u64>()
}
extern "C" fn dummy_nullary() -> *mut u64 {
    std::ptr::null_mut::<u64>()
}

fn create_jit_module() -> JITModule {
    let mut flag_builder = settings::builder();
    flag_builder.set("use_colocated_libcalls", "false").unwrap();
    flag_builder.set("is_pic", "false").unwrap();
    let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
        panic!("host machine is not supported: {}", msg);
    });
    let isa = isa_builder
        .finish(settings::Flags::new(flag_builder))
        .unwrap();
    let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

    // Register all required symbols as dummies
    let unary_symbols = [
        "dlisp_make_string",
        "dlisp_make_symbol",
        "dlisp_make_nil",
        "dlisp_file_exists",
        "dlisp_is_dir",
        "dlisp_is_file",
        "dlisp_list_dir",
        "dlisp_delete_file",
        "dlisp_getenv",
        "dlisp_set_cwd",
        "dlisp_exit",
        "dlisp_sh",
        "dlisp_str",
        "dlisp_car",
        "dlisp_cdr",
        "dlisp_print",
        "dlisp_read_file",
        "dlisp_nil_p",
        "dlisp_list_p",
        "dlisp_number_p",
        "dlisp_string_p",
        "dlisp_symbol_p",
        "dlisp_vector_p",
        "dlisp_type_of",
        "dlisp_string_length",
        "dlisp_make_vector",
        "dlisp_vector_count",
        "dlisp_vector_copy",
        "dlisp_gc_malloc",
    ];
    for name in unary_symbols {
        builder.symbol(name, dummy_ptr as *const u8);
    }

    let binary_symbols = [
        "dlisp_setenv",
        "dlisp_make_cons",
        "dlisp_add",
        "dlisp_sub",
        "dlisp_mul",
        "dlisp_gt",
        "dlisp_lt",
        "dlisp_eq",
        "dlisp_div",
        "dlisp_mod",
        "dlisp_gte",
        "dlisp_lte",
        "dlisp_neq",
        "dlisp_string_append",
        "dlisp_vector_get",
        "dlisp_vector_push",
    ];
    for name in binary_symbols {
        builder.symbol(name, dummy_ptr2 as *const u8);
    }

    let nullary_symbols = ["dlisp_cwd", "dlisp_args"];
    for name in nullary_symbols {
        builder.symbol(name, dummy_nullary as *const u8);
    }

    // dlisp_make_int takes i64 -> ptr, dlisp_make_bool takes i8 -> ptr, dlisp_make_float takes f64 -> ptr
    // dlisp_is_truthy takes ptr -> i32, dlisp_spawn takes ptr (void return conceptually)
    // dlisp_sleep takes ptr -> ptr, dlisp_substring takes (ptr, ptr, ptr) -> ptr
    // printf takes (ptr, ptr) -> i32
    builder.symbol("dlisp_make_int", dummy_ptr as *const u8);
    builder.symbol("dlisp_make_float", dummy_ptr as *const u8);
    builder.symbol("dlisp_make_bool", dummy_ptr as *const u8);
    builder.symbol("dlisp_is_truthy", dummy_ptr as *const u8);
    builder.symbol("dlisp_spawn", dummy_ptr as *const u8);
    builder.symbol("dlisp_sleep", dummy_ptr as *const u8);
    builder.symbol("dlisp_substring", dummy_ptr as *const u8);
    builder.symbol("printf", dummy_ptr as *const u8);

    JITModule::new(builder)
}

fn compile_expr_in_function(module: &mut JITModule, ast: &Value, func_name: &str) {
    let builtins_defs = declare_builtins(module).unwrap();
    let mut ctx = module.make_context();
    let mut builder_context = cranelift::frontend::FunctionBuilderContext::new();

    let int = module.target_config().pointer_type();
    ctx.func.signature.returns.push(AbiParam::new(int));

    let mut builder =
        cranelift::frontend::FunctionBuilder::new(&mut ctx.func, &mut builder_context);
    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);

    let fmt_val = builder.ins().iconst(int, 0);
    let builtins = Builtins {
        funcs: builtins_defs,
        printf_fmt: fmt_val,
    };

    let mut trans_ctx = FunctionTranslationContext {
        builder: &mut builder,
        module,
        builtins: &builtins,
        scopes: vec![HashMap::new()],
        captured_vars: HashMap::new(),
        env_param: None,
        ptr_type: int,
        global_functions: &HashMap::new(),
        global_variables: &HashMap::new(),
    };

    let result = trans_ctx.compile_expr(ast).unwrap();
    builder.ins().return_(&[result]);
    builder.finalize();

    let id = module
        .declare_function(func_name, Linkage::Export, &ctx.func.signature)
        .unwrap();
    module.define_function(id, &mut ctx).unwrap();
    module.clear_context(&mut ctx);
    module.finalize_definitions().unwrap();
}

// === IO Functions ===

#[test]
fn test_codegen_file_exists() {
    let mut module = create_jit_module();
    let ast = Value::List(vec![
        Value::Symbol("file-exists?".to_string()),
        Value::String("Cargo.toml".to_string()),
    ]);
    compile_expr_in_function(&mut module, &ast, "test_file_exists");
}

#[test]
fn test_codegen_is_dir() {
    let mut module = create_jit_module();
    let ast = Value::List(vec![
        Value::Symbol("is-dir?".to_string()),
        Value::String("src".to_string()),
    ]);
    compile_expr_in_function(&mut module, &ast, "test_is_dir");
}

#[test]
fn test_codegen_is_file() {
    let mut module = create_jit_module();
    let ast = Value::List(vec![
        Value::Symbol("is-file?".to_string()),
        Value::String("Cargo.toml".to_string()),
    ]);
    compile_expr_in_function(&mut module, &ast, "test_is_file");
}

#[test]
fn test_codegen_list_dir() {
    let mut module = create_jit_module();
    let ast = Value::List(vec![
        Value::Symbol("list-dir".to_string()),
        Value::String(".".to_string()),
    ]);
    compile_expr_in_function(&mut module, &ast, "test_list_dir");
}

#[test]
fn test_codegen_delete_file() {
    let mut module = create_jit_module();
    let ast = Value::List(vec![
        Value::Symbol("delete-file".to_string()),
        Value::String("nonexistent".to_string()),
    ]);
    compile_expr_in_function(&mut module, &ast, "test_delete_file");
}

// === SYS Functions ===

#[test]
fn test_codegen_getenv() {
    let mut module = create_jit_module();
    let ast = Value::List(vec![
        Value::Symbol("getenv".to_string()),
        Value::String("HOME".to_string()),
    ]);
    compile_expr_in_function(&mut module, &ast, "test_getenv");
}

#[test]
fn test_codegen_setenv() {
    let mut module = create_jit_module();
    let ast = Value::List(vec![
        Value::Symbol("setenv".to_string()),
        Value::String("TEST_KEY".to_string()),
        Value::String("TEST_VAL".to_string()),
    ]);
    compile_expr_in_function(&mut module, &ast, "test_setenv");
}

#[test]
fn test_codegen_cwd() {
    let mut module = create_jit_module();
    let ast = Value::List(vec![Value::Symbol("cwd".to_string())]);
    compile_expr_in_function(&mut module, &ast, "test_cwd");
}

#[test]
fn test_codegen_set_cwd() {
    let mut module = create_jit_module();
    let ast = Value::List(vec![
        Value::Symbol("set-cwd".to_string()),
        Value::String("/tmp".to_string()),
    ]);
    compile_expr_in_function(&mut module, &ast, "test_set_cwd");
}

#[test]
fn test_codegen_args() {
    let mut module = create_jit_module();
    let ast = Value::List(vec![Value::Symbol("args".to_string())]);
    compile_expr_in_function(&mut module, &ast, "test_args");
}

#[test]
fn test_codegen_exit() {
    let mut module = create_jit_module();
    let ast = Value::List(vec![Value::Symbol("exit".to_string()), Value::Integer(0)]);
    compile_expr_in_function(&mut module, &ast, "test_exit");
}

// === OS Functions ===

#[test]
fn test_codegen_sh_single_arg() {
    let mut module = create_jit_module();
    let ast = Value::List(vec![
        Value::Symbol("sh".to_string()),
        Value::String("echo hello".to_string()),
    ]);
    compile_expr_in_function(&mut module, &ast, "test_sh_single");
}

#[test]
fn test_codegen_sh_multi_arg() {
    let mut module = create_jit_module();
    let ast = Value::List(vec![
        Value::Symbol("sh".to_string()),
        Value::String("echo".to_string()),
        Value::String("hello".to_string()),
        Value::String("world".to_string()),
    ]);
    compile_expr_in_function(&mut module, &ast, "test_sh_multi");
}
