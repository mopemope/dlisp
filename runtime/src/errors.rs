//! Catchable error propagation for compiled (JIT/AOT) code.
//!
//! `throw` stores the thrown value in thread-local state and returns a
//! reserved sentinel pointer instead of a normal value. Every user function
//! call site in compiled code compares the callee result against the sentinel
//! and returns it as-is to propagate the throw. `try` compares its body
//! result against the sentinel, takes the thrown value from thread-local
//! state, wraps it in an error marker (`ValueType::Error`), and runs the
//! handler with that value bound to the catch variable.

use std::cell::Cell;

use crate::gc::dlisp_gc_malloc;
use crate::value::{DlispValue, ValueType};

thread_local! {
    /// Value carried by the most recent `throw` on this thread (null if none).
    static THROWN: Cell<*mut DlispValue> = const { Cell::new(std::ptr::null_mut()) };
    /// True while a thrown value is pending on this thread.
    static THROWN_PENDING: Cell<bool> = const { Cell::new(false) };
}

thread_local! {
    // Payload deliberately null: the sentinel is only identified by address.
    static SENTINEL: DlispValue = DlispValue::new_error(std::ptr::null_mut());
}

/// Reserved marker pointer returned by `throw` and propagated by call sites.
/// Never a valid value pointer: user values are GC-allocated, the sentinel is
/// a thread-local static.
#[unsafe(no_mangle)]
pub extern "C" fn dlisp_throw_sentinel() -> *mut DlispValue {
    SENTINEL.with(|s| s as *const DlispValue as *mut DlispValue)
}

/// True when a thrown value is pending on this thread.
#[unsafe(no_mangle)]
pub extern "C" fn dlisp_thrown_pending() -> i64 {
    i64::from(THROWN_PENDING.with(Cell::get))
}

/// Fetches and clears the pending thrown value (null if none pending).
#[unsafe(no_mangle)]
pub extern "C" fn dlisp_take_thrown() -> *mut DlispValue {
    THROWN_PENDING.with(|p| p.set(false));
    THROWN.with(|t| t.replace(std::ptr::null_mut()))
}

/// Raises a catchable error with the given value: stores it in thread-local
/// state and returns the throw sentinel, which compiled call sites propagate.
#[unsafe(no_mangle)]
pub extern "C" fn dlisp_throw(value: *mut DlispValue) -> *mut DlispValue {
    THROWN.with(|t| t.set(value));
    THROWN_PENDING.with(|p| p.set(true));
    dlisp_throw_sentinel()
}

/// Allocates an error marker wrapping `inner` (`Value::Error` parity).
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure `inner` is null or a valid `DlispValue`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_make_error(inner: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_error(inner);
        ptr
    }
}

/// Returns a Bool value: true when `v` is an error marker.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_error_p(v: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let is_error = !v.is_null() && (*v).type_ == ValueType::Error;
        crate::constructors::dlisp_make_bool(is_error)
    }
}

/// Unwraps the value inside an error marker (nil for non-errors or null).
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_error_value(v: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if v.is_null() || (*v).type_ != ValueType::Error {
            return crate::constructors::dlisp_make_nil();
        }
        let inner = (*v).payload.ptr_val as *mut DlispValue;
        if inner.is_null() {
            return crate::constructors::dlisp_make_nil();
        }
        inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constructors::{dlisp_make_int, dlisp_make_nil};

    fn unbox_bool(p: *mut DlispValue) -> bool {
        unsafe {
            assert!(!p.is_null());
            assert_eq!((*p).type_, ValueType::Bool);
            (*p).payload.bool_val
        }
    }

    #[test]
    fn test_throw_and_take_thrown() {
        assert_eq!(dlisp_thrown_pending(), 0);
        let value = dlisp_make_int(42);
        let sentinel = dlisp_throw(value);
        assert_eq!(sentinel, dlisp_throw_sentinel());
        assert_eq!(dlisp_thrown_pending(), 1);
        let taken = dlisp_take_thrown();
        assert_eq!(taken, value);
        assert_eq!(dlisp_thrown_pending(), 0);
        assert!(dlisp_take_thrown().is_null());
    }

    #[test]
    fn test_error_wrap_and_unwrap() {
        unsafe {
            let inner = dlisp_make_int(7);
            let err = dlisp_make_error(inner);
            assert!(unbox_bool(dlisp_error_p(err)));
            assert_eq!((*err).type_, ValueType::Error);
            assert_eq!(dlisp_error_value(err), inner);

            // Non-error values unwrap to nil and error? is false.
            let plain = dlisp_make_int(1);
            assert!(!unbox_bool(dlisp_error_p(plain)));
            assert_eq!((*dlisp_error_value(plain)).type_, ValueType::Nil);
            assert_eq!(
                (*dlisp_error_value(std::ptr::null_mut())).type_,
                ValueType::Nil
            );
        }
    }

    #[test]
    fn test_error_wrap_nil_inner() {
        unsafe {
            let err = dlisp_make_error(std::ptr::null_mut());
            assert!(unbox_bool(dlisp_error_p(err)));
            let unwrapped = dlisp_error_value(err);
            assert_eq!((*unwrapped).type_, ValueType::Nil);
            let _ = dlisp_make_nil();
        }
    }

    #[test]
    fn test_type_of_error() {
        unsafe {
            let err = dlisp_make_error(std::ptr::null_mut());
            let name = crate::dlisp_type_of(err);
            let c_str = std::ffi::CStr::from_ptr((*name).payload.str_val);
            assert_eq!(c_str.to_string_lossy(), "error");
        }
    }
}
