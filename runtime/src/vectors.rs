use crate::value::{DlispValue, ValueType, VectorData};
use crate::{dlisp_gc_malloc, dlisp_make_int, dlisp_make_nil};

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_make_vector(capacity: usize) -> *mut DlispValue {
    unsafe {
        let vec_data = dlisp_gc_malloc(std::mem::size_of::<VectorData>()) as *mut VectorData;
        (*vec_data).len = 0;
        (*vec_data).cap = capacity;
        if capacity > 0 {
            let data = dlisp_gc_malloc(capacity * std::mem::size_of::<*mut DlispValue>())
                as *mut *mut DlispValue;
            (*vec_data).data = data;
        } else {
            (*vec_data).data = std::ptr::null_mut();
        }

        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_vector(vec_data);
        ptr
    }
}

/// Pushes a value onto the end of a vector.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `vec` and `val` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_vector_push(vec: *mut DlispValue, val: *mut DlispValue) {
    unsafe {
        if vec.is_null() || (*vec).type_ != ValueType::Vector {
            eprintln!("Type Error: vector push requires vector");
            std::process::abort();
        }
        let vec_data = (*vec).payload.vector_val;
        if vec_data.is_null() {
            eprintln!("Internal Error: vector data is null");
            std::process::abort();
        }

        let len = (*vec_data).len;
        let cap = (*vec_data).cap;

        if len >= cap {
            // Reallocate (simple strategy: double capacity)
            let new_cap = if cap == 0 { 4 } else { cap * 2 };
            let new_data = dlisp_gc_malloc(new_cap * std::mem::size_of::<*mut DlispValue>())
                as *mut *mut DlispValue;

            if !(*vec_data).data.is_null() {
                std::ptr::copy_nonoverlapping((*vec_data).data, new_data, len);
            }

            (*vec_data).data = new_data;
            (*vec_data).cap = new_cap;
        }

        let data = (*vec_data).data;
        *data.add(len) = val;
        (*vec_data).len = len + 1;
    }
}

/// Gets an element from a vector at the specified index.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `vec` and `index_val` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_vector_get(
    vec: *mut DlispValue,
    index_val: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if vec.is_null() || (*vec).type_ != ValueType::Vector {
            eprintln!("Type Error: vector get requires vector");
            std::process::abort();
        }
        if index_val.is_null() || (*index_val).type_ != ValueType::Int {
            eprintln!("Type Error: vector get requires integer index");
            std::process::abort();
        }

        let index = (*index_val).payload.int_val;
        if index < 0 {
            return dlisp_make_nil();
        }
        let index = index as usize;

        let vec_data = (*vec).payload.vector_val;
        if vec_data.is_null() {
            return dlisp_make_nil();
        }

        if index >= (*vec_data).len {
            return dlisp_make_nil(); // Or error? Clojure returns nil.
        }

        let data = (*vec_data).data;
        *data.add(index)
    }
}

/// Returns the number of elements in a vector.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `vec` points to a valid `DlispValue` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_vector_count(vec: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if vec.is_null() {
            return dlisp_make_int(0);
        }
        if (*vec).type_ != ValueType::Vector {
            return dlisp_make_int(0);
        }
        let vec_data = (*vec).payload.vector_val;
        if vec_data.is_null() {
            return dlisp_make_int(0);
        }

        dlisp_make_int((*vec_data).len as i64)
    }
}

/// Returns a shallow copy of the vector.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `vec` points to a valid `DlispValue` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_vector_copy(vec: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if vec.is_null() || (*vec).type_ != ValueType::Vector {
            eprintln!("Type Error: vector copy requires vector");
            std::process::abort();
        }
        let original_vec_data = (*vec).payload.vector_val;
        if original_vec_data.is_null() {
            return dlisp_make_vector(0); // Copy of an empty vector is an empty vector
        }

        let original_len = (*original_vec_data).len;
        let original_cap = (*original_vec_data).cap;
        let original_data = (*original_vec_data).data;

        let new_vec_data = dlisp_gc_malloc(std::mem::size_of::<VectorData>()) as *mut VectorData;
        (*new_vec_data).len = original_len;
        (*new_vec_data).cap = original_cap; // Keep same capacity for now

        if original_len > 0 {
            let new_data = dlisp_gc_malloc(original_cap * std::mem::size_of::<*mut DlispValue>())
                as *mut *mut DlispValue;
            std::ptr::copy_nonoverlapping(original_data, new_data, original_len);
            (*new_vec_data).data = new_data;
        } else {
            (*new_vec_data).data = std::ptr::null_mut();
        }

        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_vector(new_vec_data);
        ptr
    }
}
