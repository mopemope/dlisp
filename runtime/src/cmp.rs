use std::ffi::CStr;

use crate::constructors::dlisp_make_bool;
use crate::lists::{list_to_vec, vector_to_vec};
use crate::maps::dlisp_map_get;
use crate::value::{DlispValue, ValueType};

/// Returns true when the boxed values `x` and `y` compare equal
/// according to [`dlisp_eq`].
///
/// # Safety
/// Dereferences raw pointers; caller must pass valid `DlispValue` pointers.
unsafe fn values_eq(x: *mut DlispValue, y: *mut DlispValue) -> bool {
    unsafe {
        let eq_val = dlisp_eq(x, y);
        !eq_val.is_null() && (*eq_val).type_ == ValueType::Bool && (*eq_val).payload.bool_val
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_gt(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let result = match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => (*a).payload.int_val > (*b).payload.int_val,
            (ValueType::Float, ValueType::Float) => (*a).payload.float_val > (*b).payload.float_val,
            (ValueType::Int, ValueType::Float) => {
                ((*a).payload.int_val as f64) > (*b).payload.float_val
            }
            (ValueType::Float, ValueType::Int) => {
                (*a).payload.float_val > ((*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: > requires numbers");
                std::process::abort();
            }
        };

        if result {
            dlisp_make_bool(true)
        } else {
            dlisp_make_bool(false)
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_lt(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let result = match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => (*a).payload.int_val < (*b).payload.int_val,
            (ValueType::Float, ValueType::Float) => (*a).payload.float_val < (*b).payload.float_val,
            (ValueType::Int, ValueType::Float) => {
                ((*a).payload.int_val as f64) < (*b).payload.float_val
            }
            (ValueType::Float, ValueType::Int) => {
                (*a).payload.float_val < ((*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: < requires numbers");
                std::process::abort();
            }
        };

        if result {
            dlisp_make_bool(true)
        } else {
            dlisp_make_bool(false)
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_eq(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let result = match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => (*a).payload.int_val == (*b).payload.int_val,
            (ValueType::Float, ValueType::Float) => {
                ((*a).payload.float_val - (*b).payload.float_val).abs() < f64::EPSILON
            }
            (ValueType::Int, ValueType::Float) => {
                ((*a).payload.int_val as f64 - (*b).payload.float_val).abs() < f64::EPSILON
            }
            (ValueType::Float, ValueType::Int) => {
                ((*a).payload.float_val - (*b).payload.int_val as f64).abs() < f64::EPSILON
            }
            (ValueType::Map, ValueType::Map) => {
                let m1 = (*a).payload.map_val;
                let m2 = (*b).payload.map_val;
                if m1 == m2 {
                    true
                } else if m1.is_null() || m2.is_null() || (*m1).len != (*m2).len {
                    false
                } else {
                    let len1 = (*m1).len;
                    let mut matches = 0;
                    for i in 0..(*m1).cap {
                        if let Some((k1, v1)) = *(*m1).elements.add(i) {
                            let v2 = dlisp_map_get(b, k1);
                            if v2.is_null() || (*v2).type_ == ValueType::Nil {
                                break;
                            }
                            let eq_val = dlisp_eq(v1, v2);
                            if eq_val.is_null()
                                || (*eq_val).type_ != ValueType::Bool
                                || !(*eq_val).payload.bool_val
                            {
                                break;
                            }
                            matches += 1;
                        }
                    }
                    matches == len1
                }
            }
            (ValueType::Symbol, ValueType::Symbol) => {
                let s1 = CStr::from_ptr((*a).payload.str_val);
                let s2 = CStr::from_ptr((*b).payload.str_val);
                s1 == s2
            }
            (ValueType::Keyword, ValueType::Keyword) => {
                let s1 = CStr::from_ptr((*a).payload.str_val);
                let s2 = CStr::from_ptr((*b).payload.str_val);
                s1 == s2
            }
            (ValueType::String, ValueType::String) => {
                let s1 = CStr::from_ptr((*a).payload.str_val);
                let s2 = CStr::from_ptr((*b).payload.str_val);
                s1 == s2
            }
            (ValueType::Nil, ValueType::Nil) => true,
            // Deep structural equality on collections, matching the
            // interpreter `=` builtin. Mixed container types are not equal.
            (ValueType::List, ValueType::List) => {
                let e1 = list_to_vec(a).unwrap_or_default();
                let e2 = list_to_vec(b).unwrap_or_default();
                e1.len() == e2.len() && e1.iter().zip(e2.iter()).all(|(x, y)| values_eq(*x, *y))
            }
            (ValueType::Vector, ValueType::Vector) => {
                let e1 = vector_to_vec(a);
                let e2 = vector_to_vec(b);
                e1.len() == e2.len() && e1.iter().zip(e2.iter()).all(|(x, y)| values_eq(*x, *y))
            }
            // Handles compare by identity (same wrapper object).
            (ValueType::Channel, ValueType::Channel) => (*a).opaque_ptr() == (*b).opaque_ptr(),
            (ValueType::Atom, ValueType::Atom) => (*a).opaque_ptr() == (*b).opaque_ptr(),
            _ => false,
        };

        if result {
            dlisp_make_bool(true)
        } else {
            dlisp_make_bool(false)
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_gte(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let result = match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => (*a).payload.int_val >= (*b).payload.int_val,
            (ValueType::Float, ValueType::Float) => {
                (*a).payload.float_val >= (*b).payload.float_val
            }
            (ValueType::Int, ValueType::Float) => {
                ((*a).payload.int_val as f64) >= (*b).payload.float_val
            }
            (ValueType::Float, ValueType::Int) => {
                (*a).payload.float_val >= ((*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: >= requires numbers");
                std::process::abort();
            }
        };

        if result {
            dlisp_make_bool(true)
        } else {
            dlisp_make_bool(false)
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_lte(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let result = match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => (*a).payload.int_val <= (*b).payload.int_val,
            (ValueType::Float, ValueType::Float) => {
                (*a).payload.float_val <= (*b).payload.float_val
            }
            (ValueType::Int, ValueType::Float) => {
                ((*a).payload.int_val as f64) <= (*b).payload.float_val
            }
            (ValueType::Float, ValueType::Int) => {
                (*a).payload.float_val <= ((*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: <= requires numbers");
                std::process::abort();
            }
        };

        if result {
            dlisp_make_bool(true)
        } else {
            dlisp_make_bool(false)
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_neq(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let eq = dlisp_eq(a, b);
        let is_eq = dlisp_is_truthy(eq) != 0;
        dlisp_make_bool(!is_eq)
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `val` points to a valid `DlispValue` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_is_truthy(val: *mut DlispValue) -> i32 {
    // Safety: val is checked for null. Dereferencing raw pointer.
    if val.is_null() {
        return 0;
    }
    unsafe {
        if (*val).type_ == ValueType::Nil {
            return 0;
        }
        if (*val).type_ == ValueType::Bool {
            return if (*val).payload.bool_val { 1 } else { 0 };
        }
        if (*val).type_ == ValueType::Int {
            // Historical/Compatibility: 0 is falsey?
            // Strict Lisp: only nil is false (and maybe false).
            // Let's keep 0 as falsey for now if we relied on it, but ideally we move to Bool/Nil.
            // For backwards compat with Phase 2 implementation where 0 was false.
            if (*val).payload.int_val == 0 {
                return 0;
            }
        }
        1
    }
}
