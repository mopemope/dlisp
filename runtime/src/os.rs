use crate::value::{DlispValue, ValueType};
use crate::{dlisp_make_int, dlisp_make_string};
use std::ffi::{CStr, CString};

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_sh(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        // We expect a list of strings
        if val.is_null() || (*val).type_ != ValueType::List {
            eprintln!("Type Error: sh requires a list of strings");
            std::process::abort();
        }

        let mut args = Vec::new();
        let mut curr = val;

        while !curr.is_null() && (*curr).type_ == ValueType::List {
            let list_data = (*curr).payload.list_val;
            if list_data.is_null() {
                break;
            }
            let item = (*list_data).car;
            if item.is_null() {
                break;
            }
            if (*item).type_ != ValueType::String {
                eprintln!("Type Error: sh arguments must be strings");
                std::process::abort();
            }
            let c_str = CStr::from_ptr((*item).payload.str_val);
            args.push(c_str.to_string_lossy().to_string());
            curr = (*list_data).cdr;
        }

        if args.is_empty() {
            eprintln!("Type Error: sh requires at least 1 command argument");
            std::process::abort();
        }

        let result = tokio::runtime::Handle::current().block_on(async move {
            let mut command = if args.len() == 1 {
                let mut c = tokio::process::Command::new("sh");
                c.arg("-c").arg(&args[0]);
                c
            } else {
                let mut c = tokio::process::Command::new(&args[0]);
                c.args(&args[1..]);
                c
            };
            command.output().await
        });

        match result {
            Ok(output) => {
                let status_code =
                    output
                        .status
                        .code()
                        .unwrap_or(if output.status.success() { 0 } else { -1 })
                        as i64;
                let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

                let c_stdout = CString::new(stdout_str).unwrap();
                let c_stderr = CString::new(stderr_str).unwrap();

                let lisp_status = dlisp_make_int(status_code);
                let lisp_stdout = dlisp_make_string(c_stdout.into_raw());
                let lisp_stderr = dlisp_make_string(c_stderr.into_raw());

                // Create a list instead of Map, since Map creation via C-ABI is complex and not fully exposed.
                // Format: (status stdout stderr)
                let lst3 = crate::dlisp_make_cons(lisp_stderr, crate::dlisp_make_nil());
                let lst2 = crate::dlisp_make_cons(lisp_stdout, lst3);
                crate::dlisp_make_cons(lisp_status, lst2)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let c_stderr = CString::new(format!("Command not found: {}", e)).unwrap();
                let lisp_status = dlisp_make_int(-1);
                let lisp_stdout = dlisp_make_string(CString::new("").unwrap().into_raw());
                let lisp_stderr = dlisp_make_string(c_stderr.into_raw());

                let lst3 = crate::dlisp_make_cons(lisp_stderr, crate::dlisp_make_nil());
                let lst2 = crate::dlisp_make_cons(lisp_stdout, lst3);
                crate::dlisp_make_cons(lisp_status, lst2)
            }
            Err(e) => {
                eprintln!("sh command execution failed: {}", e);
                std::process::abort();
            }
        }
    }
}
