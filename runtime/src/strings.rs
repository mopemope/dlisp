use std::ffi::CStr;

use crate::constructors::{dlisp_make_int, dlisp_make_string};
use crate::value::{DlispValue, ValueType};

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_str(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if val.is_null() {
            let c_str = std::ffi::CString::new("").unwrap();
            return dlisp_make_string(c_str.into_raw());
        }

        let s = match (*val).type_ {
            ValueType::Int => (*val).payload.int_val.to_string(),
            ValueType::Float => (*val).payload.float_val.to_string(),
            ValueType::Bool => (*val).payload.bool_val.to_string(),
            ValueType::Nil => "nil".to_string(),
            ValueType::String => {
                let c_str = CStr::from_ptr((*val).payload.str_val);
                c_str.to_string_lossy().to_string()
            }
            ValueType::Symbol => {
                let c_str = CStr::from_ptr((*val).payload.str_val);
                c_str.to_string_lossy().to_string()
            }
            ValueType::Keyword => {
                let c_str = CStr::from_ptr((*val).payload.str_val);
                format!(":{}", c_str.to_string_lossy())
            }
            ValueType::List => {
                // Simplified representation for now
                "list".to_string()
            }
            ValueType::Vector => "vector".to_string(),
            ValueType::Map => "map".to_string(),
            _ => "unknown".to_string(),
        };
        let c_str = std::ffi::CString::new(s).unwrap();
        dlisp_make_string(c_str.into_raw())
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_length(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if (*val).type_ != ValueType::String {
            eprintln!("Type Error: string-length requires string");
            std::process::abort();
        }
        let c_str = CStr::from_ptr((*val).payload.str_val);
        let s = c_str.to_string_lossy();
        let len = s.chars().count() as i64;
        dlisp_make_int(len)
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_substring(
    s_val: *mut DlispValue,
    start_val: *mut DlispValue,
    end_val: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if (*s_val).type_ != ValueType::String
            || (*start_val).type_ != ValueType::Int
            || (*end_val).type_ != ValueType::Int
        {
            eprintln!("Type Error: substring requires string, int, int");
            std::process::abort();
        }
        let c_str = CStr::from_ptr((*s_val).payload.str_val);
        let s = c_str.to_string_lossy();
        let start = (*start_val).payload.int_val as usize;
        let end = (*end_val).payload.int_val as usize;

        let chars: Vec<char> = s.chars().collect();
        if start > chars.len() || end > chars.len() || start > end {
            let empty = std::ffi::CString::new("").unwrap();
            return dlisp_make_string(empty.into_raw());
        }

        let sub: String = chars[start..end].iter().collect();
        let c_sub = std::ffi::CString::new(sub).unwrap();
        dlisp_make_string(c_sub.into_raw())
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_append(
    a: *mut DlispValue,
    b: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if (*a).type_ != ValueType::String || (*b).type_ != ValueType::String {
            eprintln!("Type Error: string-append requires strings");
            std::process::abort();
        }
        let s1 = CStr::from_ptr((*a).payload.str_val).to_string_lossy();
        let s2 = CStr::from_ptr((*b).payload.str_val).to_string_lossy();
        let combined = format!("{}{}", s1, s2);
        let c_combined = std::ffi::CString::new(combined).unwrap();
        dlisp_make_string(c_combined.into_raw())
    }
}
