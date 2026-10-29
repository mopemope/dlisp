use crate::errors::{gc_string, raise_message};
use crate::value::{DlispValue, ValueType};
use std::ffi::CStr;
use std::process::Command;

unsafe fn result_map(status: i64, stdout: &str, stderr: &str) -> *mut DlispValue {
    unsafe {
        let mut map = crate::dlisp_make_map();
        for (key, value) in [
            (c"status", crate::dlisp_make_int(status)),
            (c"stdout", gc_string(stdout)),
            (c"stderr", gc_string(stderr)),
        ] {
            let keyword = crate::dlisp_make_keyword(key.as_ptr().cast_mut());
            map = crate::dlisp_map_assoc(map, keyword, value);
        }
        map
    }
}

/// # Safety
/// `val` must be null or a valid value pointer containing a list of strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_sh(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let mut args = Vec::new();
        let mut current = val;
        while !current.is_null() && (*current).type_ == ValueType::List {
            let node = (*current).payload.list_val;
            if node.is_null() {
                break;
            }
            let item = (*node).car;
            if item.is_null() || (*item).type_ != ValueType::String {
                return raise_message("sh arguments must be strings");
            }
            args.push(
                CStr::from_ptr((*item).payload.str_val)
                    .to_string_lossy()
                    .into_owned(),
            );
            current = (*node).cdr;
        }
        if !current.is_null() && (*current).type_ != ValueType::Nil {
            return raise_message("sh requires a list of strings");
        }
        if args.is_empty() {
            return raise_message("sh requires at least 1 argument (command)");
        }
        let mut command = if args.len() == 1 {
            let mut command = Command::new("sh");
            command.arg("-c").arg(&args[0]);
            command
        } else {
            let mut command = Command::new(&args[0]);
            command.args(&args[1..]);
            command
        };
        match command.output() {
            Ok(output) => {
                if output.stdout.contains(&0) || output.stderr.contains(&0) {
                    return raise_message("sh output contains NUL");
                }
                let status = output.status.code().map(i64::from).unwrap_or(-1);
                result_map(
                    status,
                    &String::from_utf8_lossy(&output.stdout),
                    &String::from_utf8_lossy(&output.stderr),
                )
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                result_map(-1, "", &format!("Command not found: {error}"))
            }
            Err(error) => raise_message(&format!("sh command execution failed: {error}")),
        }
    }
}
