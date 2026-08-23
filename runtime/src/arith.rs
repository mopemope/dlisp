use crate::constructors::{dlisp_make_float, dlisp_make_int};
use crate::value::{DlispValue, ValueType};

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_add(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => {
                dlisp_make_int((*a).payload.int_val + (*b).payload.int_val)
            }
            (ValueType::Float, ValueType::Float) => {
                dlisp_make_float((*a).payload.float_val + (*b).payload.float_val)
            }
            (ValueType::Int, ValueType::Float) => {
                dlisp_make_float((*a).payload.int_val as f64 + (*b).payload.float_val)
            }
            (ValueType::Float, ValueType::Int) => {
                dlisp_make_float((*a).payload.float_val + (*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: + requires numbers");
                std::process::abort();
            }
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_sub(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => {
                dlisp_make_int((*a).payload.int_val - (*b).payload.int_val)
            }
            (ValueType::Float, ValueType::Float) => {
                dlisp_make_float((*a).payload.float_val - (*b).payload.float_val)
            }
            (ValueType::Int, ValueType::Float) => {
                dlisp_make_float((*a).payload.int_val as f64 - (*b).payload.float_val)
            }
            (ValueType::Float, ValueType::Int) => {
                dlisp_make_float((*a).payload.float_val - (*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: - requires numbers");
                std::process::abort();
            }
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_mul(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => {
                dlisp_make_int((*a).payload.int_val * (*b).payload.int_val)
            }
            (ValueType::Float, ValueType::Float) => {
                dlisp_make_float((*a).payload.float_val * (*b).payload.float_val)
            }
            (ValueType::Int, ValueType::Float) => {
                dlisp_make_float((*a).payload.int_val as f64 * (*b).payload.float_val)
            }
            (ValueType::Float, ValueType::Int) => {
                dlisp_make_float((*a).payload.float_val * (*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: * requires numbers");
                std::process::abort();
            }
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_div(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => {
                if (*b).payload.int_val == 0 {
                    eprintln!("Runtime Error: Division by zero");
                    std::process::abort();
                }
                dlisp_make_int((*a).payload.int_val / (*b).payload.int_val)
            }
            (ValueType::Float, ValueType::Float) => {
                dlisp_make_float((*a).payload.float_val / (*b).payload.float_val)
            }
            (ValueType::Int, ValueType::Float) => {
                dlisp_make_float((*a).payload.int_val as f64 / (*b).payload.float_val)
            }
            (ValueType::Float, ValueType::Int) => {
                dlisp_make_float((*a).payload.float_val / (*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: / requires numbers");
                std::process::abort();
            }
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_mod(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => {
                if (*b).payload.int_val == 0 {
                    eprintln!("Runtime Error: Modulo by zero");
                    std::process::abort();
                }
                dlisp_make_int((*a).payload.int_val % (*b).payload.int_val)
            }
            (ValueType::Float, ValueType::Float) => {
                dlisp_make_float((*a).payload.float_val % (*b).payload.float_val)
            }
            (ValueType::Int, ValueType::Float) => {
                dlisp_make_float((*a).payload.int_val as f64 % (*b).payload.float_val)
            }
            (ValueType::Float, ValueType::Int) => {
                dlisp_make_float((*a).payload.float_val % (*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: % requires numbers");
                std::process::abort();
            }
        }
    }
}
