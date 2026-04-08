use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use dlisp_core::ast::Value;
use dlisp_core::codegen::builtins::declare_builtins;
use dlisp_core::codegen::context::{Builtins, FunctionTranslationContext};
use dlisp_core::codegen::forms::lambda::compile_lambda;
use std::collections::HashMap;

#[test]
fn test_add_lambda_jit() {
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

    // Register dummy builtins
    builder.symbol("dlisp_make_int", dummy_make_int as *const u8);
    builder.symbol("dlisp_make_bool", dummy_make_bool as *const u8);
    builder.symbol("dlisp_add", dummy_add as *const u8);
    builder.symbol("dlisp_sub", dummy_sub as *const u8);
    builder.symbol("dlisp_lt", dummy_lt as *const u8);
    builder.symbol("dlisp_is_truthy", dummy_is_truthy as *const u8);
    builder.symbol("dlisp_gc_malloc", dummy_malloc as *const u8);
    builder.symbol("dlisp_print", dummy_print as *const u8);
    builder.symbol("dlisp_make_nil", dummy_make_nil as *const u8);
    builder.symbol("dlisp_make_closure", dummy_make_closure as *const u8);

    let mut module = JITModule::new(builder);
    let builtins_defs = declare_builtins(&mut module).unwrap();

    // Define context
    let mut ctx = module.make_context();
    let mut builder_context = cranelift::frontend::FunctionBuilderContext::new();

    let int = module.target_config().pointer_type();

    // Set up signature to return a pointer
    ctx.func.signature.returns.push(AbiParam::new(int));

    let mut builder =
        cranelift::frontend::FunctionBuilder::new(&mut ctx.func, &mut builder_context);

    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);

    // Dummy printf_fmt
    // let int = module.target_config().pointer_type();
    let fmt_val = builder.ins().iconst(int, 0);

    let builtins = Builtins {
        funcs: builtins_defs,
        printf_fmt: fmt_val,
    };

    // We are simulating JIT compiling a top level expression, but lambda compilation needs a context.
    // However, our `compile_lambda` is designed to compile a lambda expression *inside* another function.
    // The Fibonacci function `(defun fib (n) ...)` is syntax sugar for `(let ((fib (lambda (n) ...))) ...)`
    // OR we can just manually build the AST for the lambda and compile it.

    // AST: (lambda (n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2)))))
    // Wait, recursive call to `fib` requires `fib` to be in scope.
    // In our `compile_lambda`, we handle captured variables.
    // But `fib` here is a global function usually? Or a letrec?

    // Let's test a simpler recursion: A Y-combinator style or just assume global?
    // The current JIT implementation in `interpreter.rs` handles globals via `resolve_variable`.
    // But our test environment doesn't have the interpreter's `Environment`.

    // Actually, `compile_lambda` wraps the code in a function.

    // Let's verify `test_jit_if_gt` style basic logic first.
    // AST: (lambda (a b) (+ a b))
    let lambda_ast = Value::List(vec![
        Value::Symbol("lambda".to_string()),
        Value::List(vec![
            Value::Symbol("a".to_string()),
            Value::Symbol("b".to_string()),
        ]),
        Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Symbol("a".to_string()),
            Value::Symbol("b".to_string()),
        ]),
    ]);

    let mut trans_ctx = FunctionTranslationContext {
        builder: &mut builder,
        module: &mut module,
        builtins: &builtins,
        scopes: vec![HashMap::new()],
        captured_vars: HashMap::new(),
        env_param: None,
        ptr_type: int,
        global_functions: &HashMap::new(),
        global_variables: &HashMap::new(),
    };

    // Compile lambda returns a Closure Pointer (simulated)
    let closure_ptr = compile_lambda(
        &mut trans_ctx,
        match &lambda_ast {
            Value::List(l) => l,
            _ => panic!(),
        },
    )
    .unwrap();

    builder.ins().return_(&[closure_ptr]);
    builder.finalize();

    let id = module
        .declare_function("test_add_lambda", Linkage::Export, &ctx.func.signature)
        .unwrap();
    module.define_function(id, &mut ctx).unwrap();
    module.clear_context(&mut ctx);
    module.finalize_definitions().unwrap();

    let _code = module.get_finalized_function(id);
    // Logic to call this is complex (need to allocate closure struct etc in host memory if we want to run it).
    // But success here means the codegen paths are valid.
}

// Dummy implementations for linker
extern "C" fn dummy_make_int(i: i64) -> *mut u64 {
    i as *mut u64
}
extern "C" fn dummy_make_bool(i: i8) -> *mut u64 {
    i as *mut u64
}
extern "C" fn dummy_add(a: *mut u64, b: *mut u64) -> *mut u64 {
    (a as u64 + b as u64) as *mut u64
}
extern "C" fn dummy_sub(a: *mut u64, b: *mut u64) -> *mut u64 {
    (a as u64 - b as u64) as *mut u64
}
extern "C" fn dummy_lt(a: *mut u64, b: *mut u64) -> *mut u64 {
    if (a as u64) < (b as u64) {
        std::ptr::dangling_mut::<u64>()
    } else {
        std::ptr::null_mut::<u64>()
    }
}
extern "C" fn dummy_is_truthy(a: *mut u64) -> i32 {
    if a as u64 != 0 { 1 } else { 0 }
}
extern "C" fn dummy_malloc(size: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align(size, 8).unwrap();
    unsafe { std::alloc::alloc(layout) }
}
extern "C" fn dummy_print(_: *mut u64) {}
extern "C" fn dummy_make_nil() -> *mut u64 {
    std::ptr::null_mut::<u64>()
}
extern "C" fn dummy_make_closure(_env: *mut u64, _func_ptr: *const u8) -> *mut u64 {
    dummy_malloc(16) as *mut u64
}
