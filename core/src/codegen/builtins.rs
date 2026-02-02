use cranelift::prelude::*;
use cranelift_module::{FuncId, Linkage, Module};

#[derive(Clone, Copy)]
pub struct BuiltinDefinitions {
    pub printf: FuncId,
    pub dlisp_spawn: FuncId,
    pub dlisp_sleep: FuncId,
    pub gc_malloc: FuncId,
    pub dlisp_make_int: FuncId,
    pub dlisp_make_string: FuncId,
    pub dlisp_make_symbol: FuncId,
    pub dlisp_make_cons: FuncId,
    pub dlisp_make_float: FuncId,
    pub dlisp_make_bool: FuncId,
    pub dlisp_make_nil: FuncId,
    pub dlisp_car: FuncId,
    pub dlisp_cdr: FuncId,
    pub dlisp_print: FuncId,
    pub dlisp_add: FuncId,
    pub dlisp_sub: FuncId,
    pub dlisp_mul: FuncId,
    pub dlisp_gt: FuncId,
    pub dlisp_is_truthy: FuncId,
    pub dlisp_lt: FuncId,
    pub dlisp_eq: FuncId,
    pub dlisp_read_file: FuncId,
    pub dlisp_make_vector: FuncId,
    pub dlisp_vector_push: FuncId,
    pub dlisp_vector_get: FuncId,
    pub dlisp_vector_count: FuncId,
    pub dlisp_vector_copy: FuncId,
}

pub fn declare_builtins<M: Module>(module: &mut M) -> Result<BuiltinDefinitions, String> {
    let int = module.target_config().pointer_type();

    let mut sig = module.make_signature();
    sig.params.push(AbiParam::new(int));
    sig.params.push(AbiParam::new(int));
    sig.returns.push(AbiParam::new(types::I32));
    let printf_id = module
        .declare_function("printf", Linkage::Import, &sig)
        .map_err(|e| e.to_string())?;

    // dlisp_spawn(Closure*)
    let mut spawn_sig = module.make_signature();
    spawn_sig.params.push(AbiParam::new(int));
    let spawn_id = module
        .declare_function("dlisp_spawn", Linkage::Import, &spawn_sig)
        .map_err(|e| e.to_string())?;

    let mut sleep_sig = module.make_signature();
    sleep_sig.params.push(AbiParam::new(int));
    sleep_sig.returns.push(AbiParam::new(int));
    let sleep_id = module
        .declare_function("dlisp_sleep", Linkage::Import, &sleep_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_gc_malloc(size_t) -> void*
    let mut malloc_sig = module.make_signature();
    malloc_sig.params.push(AbiParam::new(int));
    malloc_sig.returns.push(AbiParam::new(int));
    let malloc_id = module
        .declare_function("dlisp_gc_malloc", Linkage::Import, &malloc_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_make_int(i64) -> DlispValue*
    let mut make_int_sig = module.make_signature();
    make_int_sig.params.push(AbiParam::new(types::I64));
    make_int_sig.returns.push(AbiParam::new(int));
    let make_int_id = module
        .declare_function("dlisp_make_int", Linkage::Import, &make_int_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_make_string(char*) -> DlispValue*
    let mut make_string_sig = module.make_signature();
    make_string_sig.params.push(AbiParam::new(int));
    make_string_sig.returns.push(AbiParam::new(int));
    let make_string_id = module
        .declare_function("dlisp_make_string", Linkage::Import, &make_string_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_make_symbol(char*) -> DlispValue*
    let mut make_symbol_sig = module.make_signature();
    make_symbol_sig.params.push(AbiParam::new(int));
    make_symbol_sig.returns.push(AbiParam::new(int));
    let make_symbol_id = module
        .declare_function("dlisp_make_symbol", Linkage::Import, &make_symbol_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_make_cons(DlispValue*, DlispValue*) -> DlispValue*
    let mut make_cons_sig = module.make_signature();
    make_cons_sig.params.push(AbiParam::new(int));
    make_cons_sig.params.push(AbiParam::new(int));
    make_cons_sig.returns.push(AbiParam::new(int));
    let make_cons_id = module
        .declare_function("dlisp_make_cons", Linkage::Import, &make_cons_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_make_float(f64) -> DlispValue*
    let mut make_float_sig = module.make_signature();
    make_float_sig.params.push(AbiParam::new(types::F64));
    make_float_sig.returns.push(AbiParam::new(int));
    let make_float_id = module
        .declare_function("dlisp_make_float", Linkage::Import, &make_float_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_make_bool(bool) -> DlispValue*
    let mut make_bool_sig = module.make_signature();
    make_bool_sig.params.push(AbiParam::new(types::I8)); // bool as i8
    make_bool_sig.returns.push(AbiParam::new(int));
    let make_bool_id = module
        .declare_function("dlisp_make_bool", Linkage::Import, &make_bool_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_make_nil() -> DlispValue*
    let mut make_nil_sig = module.make_signature();
    make_nil_sig.returns.push(AbiParam::new(int));
    let make_nil_id = module
        .declare_function("dlisp_make_nil", Linkage::Import, &make_nil_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_make_vector(capacity: usize) -> DlispValue*
    let mut make_vector_sig = module.make_signature();
    make_vector_sig.params.push(AbiParam::new(int)); // capacity
    make_vector_sig.returns.push(AbiParam::new(int));
    let make_vector_id = module
        .declare_function("dlisp_make_vector", Linkage::Import, &make_vector_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_vector_push(vec: DlispValue*, val: DlispValue*)
    let mut vector_push_sig = module.make_signature();
    vector_push_sig.params.push(AbiParam::new(int));
    vector_push_sig.params.push(AbiParam::new(int));
    let vector_push_id = module
        .declare_function("dlisp_vector_push", Linkage::Import, &vector_push_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_vector_get(vec: DlispValue*, index: usize) -> DlispValue*
    let mut vector_get_sig = module.make_signature();
    vector_get_sig.params.push(AbiParam::new(int));
    vector_get_sig.params.push(AbiParam::new(int));
    vector_get_sig.returns.push(AbiParam::new(int));
    let vector_get_id = module
        .declare_function("dlisp_vector_get", Linkage::Import, &vector_get_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_vector_count(vec: DlispValue*) -> usize
    let mut vector_count_sig = module.make_signature();
    vector_count_sig.params.push(AbiParam::new(int));
    vector_count_sig.returns.push(AbiParam::new(int));
    let vector_count_id = module
        .declare_function("dlisp_vector_count", Linkage::Import, &vector_count_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_vector_copy(vec: DlispValue*) -> DlispValue*
    let mut vector_copy_sig = module.make_signature();
    vector_copy_sig.params.push(AbiParam::new(int));
    vector_copy_sig.returns.push(AbiParam::new(int));
    let vector_copy_id = module
        .declare_function("dlisp_vector_copy", Linkage::Import, &vector_copy_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_print(DlispValue*)
    let mut print_sig = module.make_signature();
    print_sig.params.push(AbiParam::new(int));
    let print_id = module
        .declare_function("dlisp_print", Linkage::Import, &print_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_car(DlispValue*) -> DlispValue*
    let mut car_sig = module.make_signature();
    car_sig.params.push(AbiParam::new(int));
    car_sig.returns.push(AbiParam::new(int));
    let car_id = module
        .declare_function("dlisp_car", Linkage::Import, &car_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_cdr(DlispValue*) -> DlispValue*
    let mut cdr_sig = module.make_signature();
    cdr_sig.params.push(AbiParam::new(int));
    cdr_sig.returns.push(AbiParam::new(int));
    let cdr_id = module
        .declare_function("dlisp_cdr", Linkage::Import, &cdr_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_add(DlispValue*, DlispValue*) -> DlispValue*
    let mut add_sig = module.make_signature();
    add_sig.params.push(AbiParam::new(int));
    add_sig.params.push(AbiParam::new(int));
    add_sig.returns.push(AbiParam::new(int));
    let add_id = module
        .declare_function("dlisp_add", Linkage::Import, &add_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_sub(DlispValue*, DlispValue*) -> DlispValue*
    let mut sub_sig = module.make_signature();
    sub_sig.params.push(AbiParam::new(int));
    sub_sig.params.push(AbiParam::new(int));
    sub_sig.returns.push(AbiParam::new(int));
    let sub_id = module
        .declare_function("dlisp_sub", Linkage::Import, &sub_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_mul(DlispValue*, DlispValue*) -> DlispValue*
    let mut mul_sig = module.make_signature();
    mul_sig.params.push(AbiParam::new(int));
    mul_sig.params.push(AbiParam::new(int));
    mul_sig.returns.push(AbiParam::new(int));
    let mul_id = module
        .declare_function("dlisp_mul", Linkage::Import, &mul_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_gt(DlispValue*, DlispValue*) -> DlispValue*
    let mut gt_sig = module.make_signature();
    gt_sig.params.push(AbiParam::new(int));
    gt_sig.params.push(AbiParam::new(int));
    gt_sig.returns.push(AbiParam::new(int));
    let gt_id = module
        .declare_function("dlisp_gt", Linkage::Import, &gt_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_lt(DlispValue*, DlispValue*) -> DlispValue*
    let mut lt_sig = module.make_signature();
    lt_sig.params.push(AbiParam::new(int));
    lt_sig.params.push(AbiParam::new(int));
    lt_sig.returns.push(AbiParam::new(int));
    let lt_id = module
        .declare_function("dlisp_lt", Linkage::Import, &lt_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_eq(DlispValue*, DlispValue*) -> DlispValue*
    let mut eq_sig = module.make_signature();
    eq_sig.params.push(AbiParam::new(int));
    eq_sig.params.push(AbiParam::new(int));
    eq_sig.returns.push(AbiParam::new(int));
    let eq_id = module
        .declare_function("dlisp_eq", Linkage::Import, &eq_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_is_truthy(DlispValue*) -> i32 (c_int)
    let mut truthy_sig = module.make_signature();
    truthy_sig.params.push(AbiParam::new(int));
    truthy_sig.returns.push(AbiParam::new(types::I32));
    let truthy_id = module
        .declare_function("dlisp_is_truthy", Linkage::Import, &truthy_sig)
        .map_err(|e| e.to_string())?;

    // dlisp_read_file(DlispValue*) -> DlispValue*
    let mut read_file_sig = module.make_signature();
    read_file_sig.params.push(AbiParam::new(int));
    read_file_sig.returns.push(AbiParam::new(int));
    let read_file_id = module
        .declare_function("dlisp_read_file", Linkage::Import, &read_file_sig)
        .map_err(|e| e.to_string())?;

    Ok(BuiltinDefinitions {
        printf: printf_id,
        dlisp_spawn: spawn_id,
        dlisp_sleep: sleep_id,
        gc_malloc: malloc_id,
        dlisp_make_int: make_int_id,
        dlisp_make_string: make_string_id,
        dlisp_make_symbol: make_symbol_id,
        dlisp_make_cons: make_cons_id,
        dlisp_make_float: make_float_id,
        dlisp_make_bool: make_bool_id,
        dlisp_make_nil: make_nil_id,
        dlisp_car: car_id,
        dlisp_cdr: cdr_id,
        dlisp_print: print_id,
        dlisp_add: add_id,
        dlisp_sub: sub_id,
        dlisp_mul: mul_id,
        dlisp_gt: gt_id,
        dlisp_is_truthy: truthy_id,
        dlisp_lt: lt_id,
        dlisp_eq: eq_id,
        dlisp_read_file: read_file_id,
        dlisp_make_vector: make_vector_id,
        dlisp_vector_push: vector_push_id,
        dlisp_vector_get: vector_get_id,
        dlisp_vector_count: vector_count_id,
        dlisp_vector_copy: vector_copy_id,
    })
}
