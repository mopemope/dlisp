use crate::cmp::dlisp_is_truthy;
use crate::dlisp_make_nil;
use crate::lists::{build_list, list_to_vec, vector_to_vec};
use crate::value::{DlispValue, ValueType};
use crate::vectors::{dlisp_make_vector, dlisp_vector_push};

/// Signature of a compiled closure taking 1 Lisp argument.
/// The first argument is the environment pointer.
type ClosureFunc1 =
    unsafe extern "C" fn(env: *mut DlispValue, arg1: *mut DlispValue) -> *mut DlispValue;

/// Signature of a compiled closure taking 2 Lisp arguments.
type ClosureFunc2 = unsafe extern "C" fn(
    env: *mut DlispValue,
    arg1: *mut DlispValue,
    arg2: *mut DlispValue,
) -> *mut DlispValue;

/// Collects the elements of a list, vector, or nil into a Vec.
/// Returns `None` when `val` is not a collection.
///
/// # Safety
/// Dereferences raw pointers; caller must pass valid `DlispValue` pointers.
unsafe fn collection_to_elems(val: *mut DlispValue) -> Option<Vec<*mut DlispValue>> {
    unsafe {
        if val.is_null() || (*val).type_ == ValueType::Nil {
            return Some(Vec::new());
        }
        match (*val).type_ {
            ValueType::List => list_to_vec(val),
            ValueType::Vector => Some(vector_to_vec(val)),
            _ => None,
        }
    }
}

/// Builds the result container matching the input shape:
/// vectors stay vectors, everything else becomes a list (empty list is nil).
///
/// # Safety
/// Dereferences raw pointers returned by GC allocation helpers.
unsafe fn build_collection(is_vector: bool, elems: &[*mut DlispValue]) -> *mut DlispValue {
    unsafe {
        if is_vector {
            let rv = dlisp_make_vector(elems.len());
            for e in elems {
                dlisp_vector_push(rv, *e);
            }
            rv
        } else {
            build_list(elems)
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `func` is a valid `DlispValue` pointing to a closure,
/// and `coll` is a valid `DlispValue` pointing to a list, vector, or nil.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_map(
    func: *mut DlispValue,
    coll: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if func.is_null() || (*func).type_ != ValueType::Closure {
            return dlisp_make_nil(); // Error: func is not a closure
        }

        let closure = (*func).payload.closure_val;
        let func_ptr: ClosureFunc1 = std::mem::transmute((*closure).func_ptr);
        let env_ptr = (*closure).env;

        // Vectors map back to vectors, lists to lists (matching interpreter semantics).
        let is_vector = !coll.is_null() && (*coll).type_ == ValueType::Vector;

        let elems = match collection_to_elems(coll) {
            Some(elems) => elems,
            None => return dlisp_make_nil(),
        };

        let mapped: Vec<*mut DlispValue> =
            elems.iter().map(|&item| func_ptr(env_ptr, item)).collect();

        build_collection(is_vector, &mapped)
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `func` is a valid `DlispValue` pointing to a closure,
/// and `coll` is a valid `DlispValue` pointing to a list, vector, or nil.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_filter(
    func: *mut DlispValue,
    coll: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if func.is_null() || (*func).type_ != ValueType::Closure {
            return dlisp_make_nil();
        }

        let closure = (*func).payload.closure_val;
        let func_ptr: ClosureFunc1 = std::mem::transmute((*closure).func_ptr);
        let env_ptr = (*closure).env;

        let is_vector = !coll.is_null() && (*coll).type_ == ValueType::Vector;

        let elems = match collection_to_elems(coll) {
            Some(elems) => elems,
            None => return dlisp_make_nil(),
        };

        let kept: Vec<*mut DlispValue> = elems
            .into_iter()
            .filter(|&item| {
                let condition = func_ptr(env_ptr, item);
                !condition.is_null() && dlisp_is_truthy(condition) != 0
            })
            .collect();

        build_collection(is_vector, &kept)
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `func` is a valid `DlispValue` pointing to a closure,
/// `init` is a valid `DlispValue`, and `coll` is a list, vector, or nil.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_reduce(
    func: *mut DlispValue,
    init: *mut DlispValue,
    coll: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if func.is_null() || (*func).type_ != ValueType::Closure {
            return init; // Better than nil if func is invalid
        }

        let closure = (*func).payload.closure_val;
        let func_ptr: ClosureFunc2 = std::mem::transmute((*closure).func_ptr);
        let env_ptr = (*closure).env;

        let elems = match collection_to_elems(coll) {
            Some(elems) => elems,
            None => return init,
        };

        let mut acc = init;
        for item in elems {
            acc = func_ptr(env_ptr, acc, item);
        }

        acc
    }
}
