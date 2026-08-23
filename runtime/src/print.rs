use std::ffi::CStr;
use std::io::Write;

use crate::value::{DlispValue, ValueType};

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `val` points to a valid `DlispValue` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_print(val: *mut DlispValue) {
    unsafe {
        dlisp_print_value(val);
        println!(); // Newline for the main print
        let _ = std::io::stdout().flush();
    }
}

unsafe fn dlisp_print_value(val: *mut DlispValue) {
    if val.is_null() {
        print!("nil");
        return;
    }

    // Safety: we checked for null above.
    // However, since this is a recursive function dealing with raw pointers,
    // we must be careful. We rely on the GC and correct construction.
    unsafe {
        match (*val).type_ {
            ValueType::Int => {
                print!("{}", (*val).payload.int_val);
            }
            ValueType::Float => {
                print!("{}", (*val).payload.float_val);
            }
            ValueType::Bool => {
                print!("{}", (*val).payload.bool_val);
            }
            ValueType::Nil => {
                print!("nil");
            }
            ValueType::String => {
                let c_str = CStr::from_ptr((*val).payload.str_val);
                print!("{}", c_str.to_string_lossy());
            }
            ValueType::List => {
                print!("(");
                let mut curr = val;
                let mut first = true;
                loop {
                    if curr.is_null() || (*curr).type_ == ValueType::Nil {
                        break;
                    }

                    if (*curr).type_ != ValueType::List {
                        // Dotted pair
                        print!(" . ");
                        dlisp_print_value(curr);
                        break;
                    }

                    let list_data = (*curr).payload.list_val;
                    if list_data.is_null() {
                        break;
                    }

                    if !first {
                        print!(" ");
                    }
                    dlisp_print_value((*list_data).car);

                    curr = (*list_data).cdr;
                    first = false;
                }
                print!(")");
            }
            ValueType::Symbol => {
                let c_str = CStr::from_ptr((*val).payload.str_val);
                print!("{}", c_str.to_string_lossy());
            }
            ValueType::Keyword => {
                let c_str = CStr::from_ptr((*val).payload.str_val);
                print!(":{}", c_str.to_string_lossy());
            }
            ValueType::Vector => {
                print!("[");
                let vec_data = (*val).payload.vector_val;
                if !vec_data.is_null() {
                    let len = (*vec_data).len;
                    let data = (*vec_data).data;
                    for i in 0..len {
                        if i > 0 {
                            print!(" ");
                        }
                        let elem = *data.add(i);
                        dlisp_print_value(elem);
                    }
                }
                print!("]");
            }
            ValueType::Closure => {
                print!("<closure>");
            }
            ValueType::NativePtr => {
                print!("<native_ptr>");
            }
            ValueType::Map => {
                print!("{{");
                let map_data = (*val).payload.map_val;
                if !map_data.is_null() {
                    let cap = (*map_data).cap;
                    let elements = (*map_data).elements;
                    let mut first = true;
                    for i in 0..cap {
                        let opt_ptr = elements.add(i);
                        if let Some((elem_k, elem_v)) = *opt_ptr {
                            if !first {
                                print!(" ");
                            }
                            dlisp_print_value(elem_k);
                            print!(" ");
                            dlisp_print_value(elem_v);
                            first = false;
                        }
                    }
                }
                print!("}}");
            }
        }
    }
}
