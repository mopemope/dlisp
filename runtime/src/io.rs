use crate::value::{DlispValue, ValueType};
use crate::{dlisp_make_bool, dlisp_make_nil, dlisp_make_string};
use std::ffi::{CStr, CString};

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_file_exists(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if val.is_null() || (*val).type_ != ValueType::String {
            eprintln!("Type Error: file-exists? requires string");
            std::process::abort();
        }
        let c_str = CStr::from_ptr((*val).payload.str_val);
        let path = c_str.to_string_lossy().to_string();

        let result = tokio::runtime::Handle::current()
            .block_on(async move { tokio::fs::try_exists(path).await });

        match result {
            Ok(exists) => dlisp_make_bool(exists),
            Err(_) => dlisp_make_bool(false),
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_is_dir(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if val.is_null() || (*val).type_ != ValueType::String {
            eprintln!("Type Error: is-dir? requires string");
            std::process::abort();
        }
        let c_str = CStr::from_ptr((*val).payload.str_val);
        let path = c_str.to_string_lossy().to_string();

        let result = tokio::runtime::Handle::current()
            .block_on(async move { tokio::fs::metadata(path).await });

        match result {
            Ok(meta) => dlisp_make_bool(meta.is_dir()),
            Err(_) => dlisp_make_bool(false),
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_is_file(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if val.is_null() || (*val).type_ != ValueType::String {
            eprintln!("Type Error: is-file? requires string");
            std::process::abort();
        }
        let c_str = CStr::from_ptr((*val).payload.str_val);
        let path = c_str.to_string_lossy().to_string();

        let result = tokio::runtime::Handle::current()
            .block_on(async move { tokio::fs::metadata(path).await });

        match result {
            Ok(meta) => dlisp_make_bool(meta.is_file()),
            Err(_) => dlisp_make_bool(false),
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_list_dir(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if val.is_null() || (*val).type_ != ValueType::String {
            eprintln!("Type Error: list-dir requires string");
            std::process::abort();
        }
        let c_str = CStr::from_ptr((*val).payload.str_val);
        let path = c_str.to_string_lossy().to_string();

        let result = tokio::runtime::Handle::current().block_on(async move {
            let mut entries = tokio::fs::read_dir(path).await?;
            let mut results = Vec::new();
            while let Some(entry) = entries.next_entry().await? {
                if let Ok(name) = entry.file_name().into_string() {
                    results.push(name);
                }
            }
            Ok::<Vec<String>, std::io::Error>(results)
        });

        match result {
            Ok(files) => {
                let mut list = dlisp_make_nil();
                for file in files.into_iter().rev() {
                    let c_res =
                        CString::new(file).unwrap_or_else(|_| CString::new("<invalid>").unwrap());
                    let lisp_str = dlisp_make_string(c_res.into_raw());
                    list = crate::dlisp_make_cons(lisp_str, list);
                }
                list
            }
            Err(_) => dlisp_make_nil(),
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_delete_file(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if val.is_null() || (*val).type_ != ValueType::String {
            eprintln!("Type Error: delete-file requires string");
            std::process::abort();
        }
        let c_str = CStr::from_ptr((*val).payload.str_val);
        let path = c_str.to_string_lossy().to_string();

        let result = tokio::runtime::Handle::current().block_on(async move {
            let meta = tokio::fs::metadata(&path).await?;
            if meta.is_dir() {
                tokio::fs::remove_dir_all(&path).await
            } else {
                tokio::fs::remove_file(&path).await
            }
        });

        match result {
            Ok(_) => dlisp_make_bool(true),
            Err(_) => dlisp_make_bool(false),
        }
    }
}
