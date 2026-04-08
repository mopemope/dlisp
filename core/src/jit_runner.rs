use crate::ast::Value;
use dlisp_runtime::value::{DlispValue, ListData, ValueType, VectorData};
use std::ffi::{CStr, CString, c_void};

unsafe fn value_to_runtime(value: &Value) -> Option<*mut DlispValue> {
    match value {
        Value::Integer(v) => Some(dlisp_runtime::dlisp_make_int(*v)),
        Value::Float(v) => Some(dlisp_runtime::dlisp_make_float(*v)),
        Value::Bool(v) => Some(dlisp_runtime::dlisp_make_bool(*v)),
        Value::Nil => Some(dlisp_runtime::dlisp_make_nil()),
        Value::String(v) => {
            let c = CString::new(v.as_str()).ok()?;
            Some(unsafe { dlisp_runtime::dlisp_make_string(c.into_raw()) })
        }
        Value::Symbol(v) => {
            let c = CString::new(v.as_str()).ok()?;
            Some(unsafe { dlisp_runtime::dlisp_make_symbol(c.into_raw()) })
        }
        Value::Keyword(v) => {
            let c = CString::new(v.as_str()).ok()?;
            Some(unsafe { dlisp_runtime::dlisp_make_keyword(c.into_raw()) })
        }
        Value::List(items) => {
            let mut list = dlisp_runtime::dlisp_make_nil();
            for item in items.iter().rev() {
                let item_ptr = unsafe { value_to_runtime(item) }?;
                list = unsafe { dlisp_runtime::dlisp_make_cons(item_ptr, list) };
            }
            Some(list)
        }
        Value::Vector(items) => {
            let vec = dlisp_runtime::vectors::dlisp_make_vector(items.len());
            for item in items {
                let item_ptr = unsafe { value_to_runtime(item) }?;
                unsafe { dlisp_runtime::vectors::dlisp_vector_push(vec, item_ptr) };
            }
            Some(vec)
        }
        _ => None,
    }
}

unsafe fn runtime_to_value(ptr: *mut DlispValue) -> Option<Value> {
    if ptr.is_null() {
        return Some(Value::Nil);
    }

    let value = unsafe { &*ptr };
    match value.type_ {
        ValueType::Int => Some(Value::Integer(unsafe { value.payload.int_val })),
        ValueType::Float => Some(Value::Float(unsafe { value.payload.float_val })),
        ValueType::Bool => Some(Value::Bool(unsafe { value.payload.bool_val })),
        ValueType::Nil => Some(Value::Nil),
        ValueType::String => {
            let c = unsafe { CStr::from_ptr(value.payload.str_val) };
            Some(Value::String(c.to_string_lossy().into_owned()))
        }
        ValueType::Symbol => {
            let c = unsafe { CStr::from_ptr(value.payload.str_val) };
            Some(Value::Symbol(c.to_string_lossy().into_owned()))
        }
        ValueType::Keyword => {
            let c = unsafe { CStr::from_ptr(value.payload.str_val) };
            Some(Value::Keyword(c.to_string_lossy().into_owned()))
        }
        ValueType::List => unsafe { list_to_value(ptr) },
        ValueType::Vector => unsafe { vector_to_value(ptr) },
        _ => None,
    }
}

unsafe fn list_to_value(mut ptr: *mut DlispValue) -> Option<Value> {
    let mut items = Vec::new();
    loop {
        if ptr.is_null() {
            return Some(Value::List(items));
        }

        let value = unsafe { &*ptr };
        match value.type_ {
            ValueType::Nil => return Some(Value::List(items)),
            ValueType::List => {
                let list: &ListData = unsafe { &*value.payload.list_val };
                items.push(unsafe { runtime_to_value(list.car) }?);
                ptr = list.cdr;
            }
            _ => return None,
        }
    }
}

unsafe fn vector_to_value(ptr: *mut DlispValue) -> Option<Value> {
    let value = unsafe { &*ptr };
    if value.type_ != ValueType::Vector {
        return None;
    }

    let vec_data: &VectorData = unsafe { &*value.payload.vector_val };
    let mut items = Vec::with_capacity(vec_data.len);
    for i in 0..vec_data.len {
        let item_ptr = unsafe { *vec_data.data.add(i) };
        items.push(unsafe { runtime_to_value(item_ptr) }?);
    }
    Some(Value::Vector(items))
}

unsafe fn call_compiled_function(
    code_ptr: *const u8,
    args: &[*mut DlispValue],
) -> Option<*mut DlispValue> {
    type Fn0 = extern "C" fn(*mut c_void) -> *mut DlispValue;
    type Fn1 = extern "C" fn(*mut c_void, *mut DlispValue) -> *mut DlispValue;
    type Fn2 = extern "C" fn(*mut c_void, *mut DlispValue, *mut DlispValue) -> *mut DlispValue;
    type Fn3 = extern "C" fn(
        *mut c_void,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
    ) -> *mut DlispValue;
    type Fn4 = extern "C" fn(
        *mut c_void,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
    ) -> *mut DlispValue;
    type Fn5 = extern "C" fn(
        *mut c_void,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
    ) -> *mut DlispValue;
    type Fn6 = extern "C" fn(
        *mut c_void,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
    ) -> *mut DlispValue;
    type Fn7 = extern "C" fn(
        *mut c_void,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
    ) -> *mut DlispValue;
    type Fn8 = extern "C" fn(
        *mut c_void,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
        *mut DlispValue,
    ) -> *mut DlispValue;

    Some(match args.len() {
        0 => (unsafe { std::mem::transmute::<*const u8, Fn0>(code_ptr) })(std::ptr::null_mut()),
        1 => (unsafe { std::mem::transmute::<*const u8, Fn1>(code_ptr) })(
            std::ptr::null_mut(),
            args[0],
        ),
        2 => (unsafe { std::mem::transmute::<*const u8, Fn2>(code_ptr) })(
            std::ptr::null_mut(),
            args[0],
            args[1],
        ),
        3 => (unsafe { std::mem::transmute::<*const u8, Fn3>(code_ptr) })(
            std::ptr::null_mut(),
            args[0],
            args[1],
            args[2],
        ),
        4 => (unsafe { std::mem::transmute::<*const u8, Fn4>(code_ptr) })(
            std::ptr::null_mut(),
            args[0],
            args[1],
            args[2],
            args[3],
        ),
        5 => (unsafe { std::mem::transmute::<*const u8, Fn5>(code_ptr) })(
            std::ptr::null_mut(),
            args[0],
            args[1],
            args[2],
            args[3],
            args[4],
        ),
        6 => (unsafe { std::mem::transmute::<*const u8, Fn6>(code_ptr) })(
            std::ptr::null_mut(),
            args[0],
            args[1],
            args[2],
            args[3],
            args[4],
            args[5],
        ),
        7 => (unsafe { std::mem::transmute::<*const u8, Fn7>(code_ptr) })(
            std::ptr::null_mut(),
            args[0],
            args[1],
            args[2],
            args[3],
            args[4],
            args[5],
            args[6],
        ),
        8 => (unsafe { std::mem::transmute::<*const u8, Fn8>(code_ptr) })(
            std::ptr::null_mut(),
            args[0],
            args[1],
            args[2],
            args[3],
            args[4],
            args[5],
            args[6],
            args[7],
        ),
        _ => return None,
    })
}

/// Executes a JIT-compiled function with the given arguments.
///
/// # Safety
///
/// This function is unsafe because it transmutes a raw pointer to a function pointer
/// and executes it. The caller must ensure that `code_ptr` points to a valid
/// function with a signature matching the provided DLisp metadata.
pub unsafe fn run_jit_function(
    code_ptr: *const u8,
    fixed_arity: usize,
    has_rest: bool,
    args: &[Value],
) -> Option<Value> {
    dlisp_runtime::dlisp_gc_init();

    let mut runtime_args = Vec::with_capacity(fixed_arity + usize::from(has_rest));
    for arg in &args[..fixed_arity] {
        runtime_args.push(unsafe { value_to_runtime(arg) }?);
    }
    if has_rest {
        runtime_args.push(unsafe { value_to_runtime(&Value::List(args[fixed_arity..].to_vec())) }?);
    }

    let result = unsafe { call_compiled_function(code_ptr, &runtime_args) }?;
    unsafe { runtime_to_value(result) }
}
