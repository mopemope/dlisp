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

// --- Compiled string helpers (used by JIT/AOT codegen) ---

use crate::constructors::{dlisp_make_bool, dlisp_make_float, dlisp_make_nil};

unsafe fn require_string(val: *mut DlispValue, op: &str, arg_desc: &str) -> String {
    unsafe {
        if val.is_null() || (*val).type_ != ValueType::String {
            eprintln!("Type Error: {} {} must be a string", op, arg_desc);
            std::process::abort();
        }
        CStr::from_ptr((*val).payload.str_val)
            .to_string_lossy()
            .into_owned()
    }
}

unsafe fn alloc_string(s: &str) -> *mut DlispValue {
    unsafe {
        let bytes = s.as_bytes();
        let len = bytes.len();
        let ptr = crate::gc::dlisp_gc_malloc(len + 1) as *mut u8;
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
        *ptr.add(len) = 0;
        dlisp_make_string(ptr as *mut i8)
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_split(
    s_val: *mut DlispValue,
    sep_val: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        let s = require_string(s_val, "string-split", "first arg");
        let sep = require_string(sep_val, "string-split", "second arg");
        let parts: Vec<*mut DlispValue> = s.split(&sep).map(|part| alloc_string(part)).collect();
        crate::lists::build_list(&parts)
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_replace(
    s_val: *mut DlispValue,
    from_val: *mut DlispValue,
    to_val: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        let s = require_string(s_val, "string-replace", "first arg");
        let from = require_string(from_val, "string-replace", "second arg");
        let to = require_string(to_val, "string-replace", "third arg");
        alloc_string(&s.replace(&from, &to))
    }
}

unsafe fn case_string(val: *mut DlispValue, op: &str, upper: bool) -> *mut DlispValue {
    unsafe {
        let s = if val.is_null() || (*val).type_ != ValueType::String {
            eprintln!("Type Error: {} requires a string argument", op);
            std::process::abort();
        } else {
            CStr::from_ptr((*val).payload.str_val)
                .to_string_lossy()
                .into_owned()
        };
        alloc_string(&if upper {
            s.to_uppercase()
        } else {
            s.to_lowercase()
        })
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_upper(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { case_string(val, "string-upper", true) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_lower(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { case_string(val, "string-lower", false) }
}

unsafe fn trim_impl(val: *mut DlispValue, op: &str, side: u8) -> *mut DlispValue {
    unsafe {
        if val.is_null() || (*val).type_ != ValueType::String {
            eprintln!("Type Error: {} requires a string argument", op);
            std::process::abort();
        }
        let s = CStr::from_ptr((*val).payload.str_val)
            .to_string_lossy()
            .into_owned();
        let trimmed = match side {
            0 => s.trim().to_string(),
            1 => s.trim_start().to_string(),
            _ => s.trim_end().to_string(),
        };
        alloc_string(&trimmed)
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_trim(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { trim_impl(val, "string-trim", 0) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_trim_left(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { trim_impl(val, "string-trim-left", 1) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_trim_right(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { trim_impl(val, "string-trim-right", 2) }
}

unsafe fn binary_string_pred(
    a: *mut DlispValue,
    b: *mut DlispValue,
    op: &str,
    f: impl Fn(&str, &str) -> bool,
) -> *mut DlispValue {
    unsafe {
        let s1 = require_string(a, op, "first arg");
        let s2 = require_string(b, op, "second arg");
        dlisp_make_bool(f(&s1, &s2))
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_starts_with(
    a: *mut DlispValue,
    b: *mut DlispValue,
) -> *mut DlispValue {
    unsafe { binary_string_pred(a, b, "string-starts-with?", |s, p| s.starts_with(p)) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_ends_with(
    a: *mut DlispValue,
    b: *mut DlispValue,
) -> *mut DlispValue {
    unsafe { binary_string_pred(a, b, "string-ends-with?", |s, p| s.ends_with(p)) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_contains(
    a: *mut DlispValue,
    b: *mut DlispValue,
) -> *mut DlispValue {
    unsafe { binary_string_pred(a, b, "string-contains?", |s, p| s.contains(p)) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_index_of(
    a: *mut DlispValue,
    b: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        let s = require_string(a, "string-index-of", "first arg");
        let sub = require_string(b, "string-index-of", "second arg");
        match s.find(&sub) {
            Some(idx) => dlisp_make_int(idx as i64),
            None => dlisp_make_nil(),
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_to_number(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if val.is_null() || (*val).type_ != ValueType::String {
            eprintln!("Type Error: string->number requires a string argument");
            std::process::abort();
        }
        let s = CStr::from_ptr((*val).payload.str_val)
            .to_string_lossy()
            .into_owned();
        if let Ok(i) = s.parse::<i64>() {
            dlisp_make_int(i)
        } else if let Ok(f) = s.parse::<f64>() {
            dlisp_make_float(f)
        } else {
            dlisp_make_nil()
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_number_to_string(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if val.is_null() {
            eprintln!("Type Error: number->string requires a number argument");
            std::process::abort();
        }
        match (*val).type_ {
            ValueType::Int => alloc_string(&(*val).payload.int_val.to_string()),
            ValueType::Float => alloc_string(&(*val).payload.float_val.to_string()),
            _ => {
                eprintln!("Type Error: number->string requires a number argument");
                std::process::abort();
            }
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_char_at(
    s_val: *mut DlispValue,
    idx_val: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        let s = require_string(s_val, "char-at", "first arg");
        if idx_val.is_null() || (*idx_val).type_ != ValueType::Int {
            eprintln!("Type Error: char-at second arg must be an integer");
            std::process::abort();
        }
        let i = (*idx_val).payload.int_val;
        if i < 0 {
            eprintln!("Type Error: char-at index must be non-negative");
            std::process::abort();
        }
        let chars: Vec<char> = s.chars().collect();
        let idx = i as usize;
        if idx < chars.len() {
            alloc_string(&chars[idx].to_string())
        } else {
            dlisp_make_nil()
        }
    }
}
