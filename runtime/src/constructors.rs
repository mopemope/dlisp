use std::ffi::c_char;

use crate::gc::dlisp_gc_malloc;
use crate::value::{ClosureData, DlispValue, ListData, ValueType};

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_make_int(val: i64) -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_int(val);
        ptr
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `s` points to a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_make_string(s: *mut c_char) -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_string(s);
        ptr
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `s` points to a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_make_keyword(s: *mut c_char) -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_keyword(s);
        ptr
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `s` points to a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_make_symbol(s: *mut c_char) -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_symbol(s);
        ptr
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_make_closure(
    env: *mut DlispValue,
    func_ptr: *const std::ffi::c_void,
) -> *mut DlispValue {
    unsafe {
        let closure_data_ptr =
            dlisp_gc_malloc(std::mem::size_of::<ClosureData>()) as *mut ClosureData;
        *closure_data_ptr = ClosureData { env, func_ptr };

        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        (*ptr).type_ = ValueType::Closure;
        (*ptr).payload.closure_val = closure_data_ptr;
        ptr
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `car` and `cdr` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_make_cons(
    car: *mut DlispValue,
    cdr: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        let list_data_ptr = dlisp_gc_malloc(std::mem::size_of::<ListData>()) as *mut ListData;
        *list_data_ptr = ListData { car, cdr };

        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        (*ptr).type_ = ValueType::List;
        (*ptr).payload.list_val = list_data_ptr;
        ptr
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `car` and `cdr` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_cons(car: *mut DlispValue, cdr: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if cdr.is_null() {
            return dlisp_make_cons(car, dlisp_make_nil());
        }

        match (*cdr).type_ {
            ValueType::List | ValueType::Nil => dlisp_make_cons(car, cdr),
            _ => {
                let tail = dlisp_make_cons(cdr, dlisp_make_nil());
                dlisp_make_cons(car, tail)
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_make_float(val: f64) -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_float(val);
        ptr
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_make_bool(val: bool) -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_bool(val);
        ptr
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_make_nil() -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_nil();
        ptr
    }
}
