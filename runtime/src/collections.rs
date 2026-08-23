use crate::cmp::dlisp_eq;
use crate::constructors::{dlisp_make_cons, dlisp_make_nil};
use crate::maps::dlisp_hash_value;
use crate::value::{DlispValue, ValueType};
use crate::vectors::{dlisp_vector_copy, dlisp_vector_get, dlisp_vector_push};

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_get(
    collection: *mut DlispValue,
    key: *mut DlispValue,
    default: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if collection.is_null() {
            return default;
        }

        match (*collection).type_ {
            ValueType::Map => {
                let map_data = (*collection).payload.map_val;
                if map_data.is_null() || (*map_data).len == 0 {
                    return default;
                }

                let cap = (*map_data).cap;
                let hash = dlisp_hash_value(key) as usize;
                let mut idx = hash % cap;
                let start_idx = idx;

                loop {
                    let opt_ptr = (*map_data).elements.add(idx);
                    if (*opt_ptr).is_none() {
                        return default;
                    } else if let Some((existing_key, existing_val)) = *opt_ptr {
                        let eq_val = dlisp_eq(existing_key, key);
                        if !eq_val.is_null()
                            && (*eq_val).type_ == ValueType::Bool
                            && (*eq_val).payload.bool_val
                        {
                            return existing_val;
                        }
                    }
                    idx = (idx + 1) % cap;
                    if idx == start_idx {
                        return default;
                    }
                }
            }
            ValueType::Vector => dlisp_vector_get(collection, key),
            ValueType::Nil => default,
            _ => dlisp_make_nil(),
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_conj(
    collection: *mut DlispValue,
    item: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if collection.is_null() || (*collection).type_ == ValueType::Nil {
            return dlisp_make_cons(item, dlisp_make_nil());
        }

        match (*collection).type_ {
            ValueType::List => dlisp_make_cons(item, collection),
            ValueType::Vector => {
                let copied = dlisp_vector_copy(collection);
                dlisp_vector_push(copied, item);
                copied
            }
            _ => {
                eprintln!("Type Error: conj requires list, vector, or nil");
                std::process::abort();
            }
        }
    }
}
