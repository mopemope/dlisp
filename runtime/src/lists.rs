use crate::constructors::dlisp_make_nil;
use crate::value::{DlispValue, ValueType};

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `list` points to a valid `DlispValue` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_car(list: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if list.is_null() {
            return dlisp_make_nil();
        }
        let val = *list;
        if val.type_ != ValueType::List {
            if val.type_ == ValueType::Nil {
                return dlisp_make_nil();
            }
            return dlisp_make_nil();
        }
        let list_data = *val.payload.list_val;
        list_data.car
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `list` points to a valid `DlispValue` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_cdr(list: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if list.is_null() {
            return dlisp_make_nil();
        }
        let val = *list;
        if val.type_ != ValueType::List {
            if val.type_ == ValueType::Nil {
                return dlisp_make_nil();
            }
            return dlisp_make_nil();
        }
        let list_data = *val.payload.list_val;
        list_data.cdr
    }
}
