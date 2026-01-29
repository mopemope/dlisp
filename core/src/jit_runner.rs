use crate::ast::Value;

pub unsafe fn run_jit_function(code_ptr: *const u8, args: &[Value]) -> Option<Value> {
    let all_ints = args.iter().all(|v| matches!(v, Value::Integer(_)));
    if !all_ints {
        return None;
    }

    match args.len() {
        0 => {
            let func_ptr: extern "C" fn(*mut std::ffi::c_void) -> i64 =
                unsafe { std::mem::transmute(code_ptr) };
            Some(Value::Integer(func_ptr(std::ptr::null_mut())))
        }
        1 => {
            let a1 = match args[0] {
                Value::Integer(i) => i,
                _ => 0,
            };
            let func_ptr: extern "C" fn(*mut std::ffi::c_void, i64) -> i64 =
                unsafe { std::mem::transmute(code_ptr) };
            Some(Value::Integer(func_ptr(std::ptr::null_mut(), a1)))
        }
        2 => {
            let a1 = match args[0] {
                Value::Integer(i) => i,
                _ => 0,
            };
            let a2 = match args[1] {
                Value::Integer(i) => i,
                _ => 0,
            };
            let func_ptr: extern "C" fn(*mut std::ffi::c_void, i64, i64) -> i64 =
                unsafe { std::mem::transmute(code_ptr) };
            Some(Value::Integer(func_ptr(std::ptr::null_mut(), a1, a2)))
        }
        3 => {
            let a1 = match args[0] {
                Value::Integer(i) => i,
                _ => 0,
            };
            let a2 = match args[1] {
                Value::Integer(i) => i,
                _ => 0,
            };
            let a3 = match args[2] {
                Value::Integer(i) => i,
                _ => 0,
            };
            let func_ptr: extern "C" fn(*mut std::ffi::c_void, i64, i64, i64) -> i64 =
                unsafe { std::mem::transmute(code_ptr) };
            Some(Value::Integer(func_ptr(std::ptr::null_mut(), a1, a2, a3)))
        }
        _ => None,
    }
}
