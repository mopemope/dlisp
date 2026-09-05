use std::ffi::CString;

use crate::constructors::{dlisp_make_bool, dlisp_make_string};
use crate::value::{DlispValue, ValueType};

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_nil_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { dlisp_make_bool(!val.is_null() && (*val).type_ == ValueType::Nil) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_list_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        dlisp_make_bool(
            !val.is_null() && ((*val).type_ == ValueType::List || (*val).type_ == ValueType::Nil),
        )
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_number_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        dlisp_make_bool(
            !val.is_null() && ((*val).type_ == ValueType::Int || (*val).type_ == ValueType::Float),
        )
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { dlisp_make_bool(!val.is_null() && (*val).type_ == ValueType::String) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_symbol_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { dlisp_make_bool(!val.is_null() && (*val).type_ == ValueType::Symbol) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_keyword_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { dlisp_make_bool(!val.is_null() && (*val).type_ == ValueType::Keyword) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_vector_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { dlisp_make_bool(!val.is_null() && (*val).type_ == ValueType::Vector) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_map_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { dlisp_make_bool(!val.is_null() && (*val).type_ == ValueType::Map) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_is_empty(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if val.is_null() || (*val).type_ == ValueType::Nil {
            return dlisp_make_bool(true);
        }
        let is_empty = match (*val).type_ {
            // A cons cell always holds at least one element; the empty list
            // is represented as Nil (handled above).
            ValueType::List => false,
            ValueType::Vector => {
                let vec_data = (*val).payload.vector_val;
                vec_data.is_null() || (*vec_data).len == 0
            }
            ValueType::Map => {
                let map_data = (*val).payload.map_val;
                map_data.is_null() || (*map_data).len == 0
            }
            ValueType::String => *(*val).payload.str_val == 0,
            _ => {
                eprintln!("Type Error: empty? requires a collection or string");
                std::process::abort();
            }
        };
        dlisp_make_bool(is_empty)
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_type_of(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let name = if val.is_null() {
            "nil"
        } else {
            match (*val).type_ {
                ValueType::Int => "integer",
                ValueType::Float => "float",
                ValueType::Bool => "boolean",
                ValueType::Nil => "nil",
                ValueType::String => "string",
                ValueType::Symbol => "symbol",
                ValueType::Keyword => "keyword",
                ValueType::List => "cons",
                ValueType::Vector => "vector",
                ValueType::Map => "map",
                ValueType::Closure => "closure",
                ValueType::Channel => "channel",
                ValueType::Atom => "atom",
                ValueType::NativePtr => "native_ptr",
                ValueType::Error => "error",
            }
        };
        let c_str = CString::new(name).unwrap();
        dlisp_make_string(c_str.into_raw())
    }
}
