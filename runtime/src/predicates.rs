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
                ValueType::NativePtr => "native_ptr",
            }
        };
        let c_str = CString::new(name).unwrap();
        dlisp_make_string(c_str.into_raw())
    }
}
