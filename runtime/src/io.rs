use crate::errors::{gc_string, raise_message};
use crate::value::{DlispValue, ValueType};
use crate::{dlisp_make_bool, dlisp_make_nil};
use std::ffi::CStr;
use std::fs;
use std::io::ErrorKind;

unsafe fn path_arg(val: *mut DlispValue, message: &str) -> Result<String, *mut DlispValue> {
    unsafe {
        if val.is_null() || (*val).type_ != ValueType::String {
            return Err(raise_message(message));
        }
        Ok(CStr::from_ptr((*val).payload.str_val)
            .to_string_lossy()
            .into_owned())
    }
}

/// # Safety
/// `val` must be null or a valid value pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_file_exists(val: *mut DlispValue) -> *mut DlispValue {
    let path = match unsafe { path_arg(val, "file-exists? argument must be a string (path)") } {
        Ok(path) => path,
        Err(error) => return error,
    };
    match fs::exists(path) {
        Ok(exists) => dlisp_make_bool(exists),
        Err(error) => raise_message(&format!("file-exists? error: {error}")),
    }
}

unsafe fn metadata_predicate(val: *mut DlispValue, directory: bool) -> *mut DlispValue {
    let name = if directory { "is-dir?" } else { "is-file?" };
    let path = match unsafe { path_arg(val, &format!("{name} argument must be a string (path)")) } {
        Ok(path) => path,
        Err(error) => return error,
    };
    match fs::metadata(path) {
        Ok(metadata) => dlisp_make_bool(if directory {
            metadata.is_dir()
        } else {
            metadata.is_file()
        }),
        Err(error) if error.kind() == ErrorKind::NotFound => dlisp_make_bool(false),
        Err(error) => raise_message(&format!("{name} error: {error}")),
    }
}

/// # Safety
/// `val` must be null or a valid value pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_is_dir(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { metadata_predicate(val, true) }
}

/// # Safety
/// `val` must be null or a valid value pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_is_file(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { metadata_predicate(val, false) }
}

/// # Safety
/// `val` must be null or a valid value pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_list_dir(val: *mut DlispValue) -> *mut DlispValue {
    let path = match unsafe { path_arg(val, "list-dir argument must be a string (path)") } {
        Ok(path) => path,
        Err(error) => return error,
    };
    let entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(error) => return raise_message(&format!("list-dir error for path '{path}': {error}")),
    };
    let mut files = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => {
                if let Ok(name) = entry.file_name().into_string() {
                    files.push(name);
                }
            }
            Err(error) => return raise_message(&format!("list-dir error reading entry: {error}")),
        }
    }
    let mut list = dlisp_make_nil();
    for file in files.into_iter().rev() {
        list = unsafe { crate::dlisp_make_cons(gc_string(&file), list) };
    }
    list
}

/// # Safety
/// `val` must be null or a valid value pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_delete_file(val: *mut DlispValue) -> *mut DlispValue {
    let path = match unsafe { path_arg(val, "delete-file argument must be a string (path)") } {
        Ok(path) => path,
        Err(error) => return error,
    };
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return dlisp_make_bool(false),
        Err(error) => return raise_message(&format!("delete-file error: {error}")),
    };
    let result = if metadata.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };
    match result {
        Ok(()) => dlisp_make_bool(true),
        Err(error) => raise_message(&format!("delete-file error: {error}")),
    }
}

/// # Safety
/// `val` must be null or a valid value pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_read_file(val: *mut DlispValue) -> *mut DlispValue {
    let path = match unsafe { path_arg(val, "read-file requires a string argument") } {
        Ok(path) => path,
        Err(error) => return error,
    };
    match fs::read_to_string(path) {
        Ok(content) if content.contains('\0') => raise_message("read-file result contains NUL"),
        Ok(content) => gc_string(&content),
        Err(error) if error.kind() == ErrorKind::NotFound => dlisp_make_nil(),
        Err(error) => raise_message(&format!("read-file error: {error}")),
    }
}
