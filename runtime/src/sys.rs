use crate::value::{DlispValue, ValueType};
use crate::{dlisp_make_bool, dlisp_make_nil, dlisp_make_string};
use std::ffi::{CStr, CString};

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_getenv(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if val.is_null() || (*val).type_ != ValueType::String {
            eprintln!("Type Error: getenv requires a string argument");
            std::process::abort();
        }
        let c_str = CStr::from_ptr((*val).payload.str_val);
        let name = c_str.to_string_lossy().to_string();

        match std::env::var(&name) {
            Ok(value) => {
                let c_val = CString::new(value).unwrap_or_else(|_| CString::new("").unwrap());
                dlisp_make_string(c_val.into_raw())
            }
            Err(_) => dlisp_make_nil(),
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_setenv(
    name_val: *mut DlispValue,
    value_val: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if name_val.is_null() || (*name_val).type_ != ValueType::String {
            eprintln!("Type Error: setenv name must be a string");
            std::process::abort();
        }
        if value_val.is_null() || (*value_val).type_ != ValueType::String {
            eprintln!("Type Error: setenv value must be a string");
            std::process::abort();
        }
        let c_name = CStr::from_ptr((*name_val).payload.str_val);
        let name = c_name.to_string_lossy().to_string();
        let c_value = CStr::from_ptr((*value_val).payload.str_val);
        let value = c_value.to_string_lossy().to_string();

        // SAFETY: setenv is unsafe in multi-threaded contexts, but
        // we maintain the same pattern as the interpreter builtins.
        std::env::set_var(&name, &value);
        dlisp_make_bool(true)
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_cwd() -> *mut DlispValue {
    unsafe {
        match std::env::current_dir() {
            Ok(path) => {
                let path_str = path.to_string_lossy().to_string();
                let c_str = CString::new(path_str).unwrap_or_else(|_| CString::new("").unwrap());
                dlisp_make_string(c_str.into_raw())
            }
            Err(_) => dlisp_make_nil(),
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_set_cwd(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if val.is_null() || (*val).type_ != ValueType::String {
            eprintln!("Type Error: set-cwd requires a string argument");
            std::process::abort();
        }
        let c_str = CStr::from_ptr((*val).payload.str_val);
        let path = c_str.to_string_lossy().to_string();

        match std::env::set_current_dir(&path) {
            Ok(_) => dlisp_make_bool(true),
            Err(_) => dlisp_make_bool(false),
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_args() -> *mut DlispValue {
    unsafe {
        let args: Vec<String> = std::env::args().collect();
        let mut list = dlisp_make_nil();
        for arg in args.into_iter().rev() {
            let c_str = CString::new(arg).unwrap_or_else(|_| CString::new("").unwrap());
            let lisp_str = dlisp_make_string(c_str.into_raw());
            list = crate::dlisp_make_cons(lisp_str, list);
        }
        list
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_exit(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let code = if val.is_null() || (*val).type_ == ValueType::Nil {
            0
        } else if (*val).type_ == ValueType::Int {
            (*val).payload.int_val as i32
        } else {
            eprintln!("Type Error: exit requires an integer or nil argument");
            std::process::abort();
        };
        std::process::exit(code);
    }
}
