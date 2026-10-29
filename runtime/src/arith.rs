use crate::constructors::{dlisp_make_float, dlisp_make_int};
use crate::errors::raise_message;
use crate::value::{DlispValue, ValueType};

#[derive(Clone, Copy)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

unsafe fn arithmetic(a: *mut DlispValue, b: *mut DlispValue, op: Op) -> *mut DlispValue {
    unsafe {
        if a.is_null() || b.is_null() {
            return raise_message("Arguments must be numbers");
        }
        if (*a).type_ == ValueType::Int && (*b).type_ == ValueType::Int {
            let a = (*a).payload.int_val;
            let b = (*b).payload.int_val;
            if matches!(op, Op::Div | Op::Mod) && b == 0 {
                return raise_message("Division by zero");
            }
            let result = match op {
                Op::Add => a.checked_add(b),
                Op::Sub => a.checked_sub(b),
                Op::Mul => a.checked_mul(b),
                Op::Div => a.checked_div(b),
                Op::Mod => a.checked_rem(b),
            };
            return result
                .map(|value| dlisp_make_int(value))
                .unwrap_or_else(|| raise_message("Integer overflow"));
        }
        let number = |value: *mut DlispValue| match (*value).type_ {
            ValueType::Int => Some((*value).payload.int_val as f64),
            ValueType::Float => Some((*value).payload.float_val),
            _ => None,
        };
        let (Some(a), Some(b)) = (number(a), number(b)) else {
            return raise_message("Arguments must be numbers");
        };
        if matches!(op, Op::Div | Op::Mod) && b == 0.0 {
            return raise_message("Division by zero");
        }
        dlisp_make_float(match op {
            Op::Add => a + b,
            Op::Sub => a - b,
            Op::Mul => a * b,
            Op::Div => a / b,
            Op::Mod => a % b,
        })
    }
}

/// # Safety
/// `value` must be null or a valid value pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_require_number(value: *mut DlispValue) -> *mut DlispValue {
    if !value.is_null() && matches!(unsafe { (*value).type_ }, ValueType::Int | ValueType::Float) {
        value
    } else {
        raise_message("Arguments must be numbers")
    }
}

/// # Safety
/// Both arguments must be null or valid value pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_add(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe { arithmetic(a, b, Op::Add) }
}

/// # Safety
/// Both arguments must be null or valid value pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_sub(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe { arithmetic(a, b, Op::Sub) }
}

/// # Safety
/// Both arguments must be null or valid value pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_mul(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe { arithmetic(a, b, Op::Mul) }
}

/// # Safety
/// Both arguments must be null or valid value pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_div(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe { arithmetic(a, b, Op::Div) }
}

/// # Safety
/// Both arguments must be null or valid value pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_mod(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe { arithmetic(a, b, Op::Mod) }
}
