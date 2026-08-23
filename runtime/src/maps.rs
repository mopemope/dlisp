use std::ffi::CStr;
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::cmp::dlisp_eq;
use crate::constructors::dlisp_make_nil;
use crate::gc::dlisp_gc_malloc;
use crate::value::{DlispValue, MapData, ValuePayload, ValueType};

/// # Map Operations
/// # Safety
/// This function is unsafe because it uses raw pointers directly.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_make_map() -> *mut DlispValue {
    unsafe {
        let cap = 8;
        let map_val = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        let map_data = dlisp_gc_malloc(std::mem::size_of::<MapData>()) as *mut MapData;

        let elements_size = cap * std::mem::size_of::<Option<(*mut DlispValue, *mut DlispValue)>>();
        let elements =
            dlisp_gc_malloc(elements_size) as *mut Option<(*mut DlispValue, *mut DlispValue)>;

        // Initialize to None
        for i in 0..cap {
            std::ptr::write(elements.add(i), None);
        }

        (*map_data).len = 0;
        (*map_data).cap = cap;
        (*map_data).elements = elements;

        *map_val = DlispValue {
            type_: ValueType::Map,
            payload: ValuePayload { map_val: map_data },
        };
        map_val
    }
}

pub(crate) unsafe fn dlisp_hash_value(val: *mut DlispValue) -> u64 {
    let mut hasher = DefaultHasher::new();
    if val.is_null() {
        return 0;
    }
    unsafe {
        match (*val).type_ {
            ValueType::Int => {
                (*val).payload.int_val.hash(&mut hasher);
            }
            ValueType::Bool => {
                (*val).payload.bool_val.hash(&mut hasher);
            }
            ValueType::String | ValueType::Symbol | ValueType::Keyword => {
                let c_str = CStr::from_ptr((*val).payload.str_val);
                c_str.to_bytes().hash(&mut hasher);
            }
            _ => {
                // For now, simpler hash based on pointer address for other types
                (val as usize).hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

unsafe fn dlisp_map_insert(map_data: *mut MapData, key: *mut DlispValue, val: *mut DlispValue) {
    unsafe {
        let cap = (*map_data).cap;
        let hash = dlisp_hash_value(key) as usize;
        let mut idx = hash % cap;

        // Linear probing
        loop {
            let opt_ptr = (*map_data).elements.add(idx);
            if (*opt_ptr).is_none() {
                std::ptr::write(opt_ptr, Some((key, val)));
                (*map_data).len += 1;
                return;
            } else if let Some((existing_key, _)) = *opt_ptr {
                let eq_val = dlisp_eq(existing_key, key);
                if !eq_val.is_null()
                    && (*eq_val).type_ == ValueType::Bool
                    && (*eq_val).payload.bool_val
                {
                    // Update existing
                    std::ptr::write(opt_ptr, Some((key, val)));
                    return;
                }
            }
            idx = (idx + 1) % cap;
        }
    }
}

/// # Safety
/// This function is unsafe because it uses raw pointers directly.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_map_assoc(
    map_val: *mut DlispValue,
    key: *mut DlispValue,
    val: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if map_val.is_null() || (*map_val).type_ != ValueType::Map {
            eprintln!("Type Error: assoc requires a map as first argument");
            std::process::abort();
        }

        let old_map_data = (*map_val).payload.map_val;
        let old_cap = (*old_map_data).cap;
        let old_len = (*old_map_data).len;

        // Check if this key already exists to avoid over-counting
        let existing = dlisp_map_get(map_val, key);
        let is_update = !existing.is_null() && (*existing).type_ != ValueType::Nil;
        let new_len = if is_update { old_len } else { old_len + 1 };

        let new_cap = if new_len * 2 > old_cap {
            old_cap * 2
        } else {
            old_cap
        };

        let result_map_val = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        let result_map_data = dlisp_gc_malloc(std::mem::size_of::<MapData>()) as *mut MapData;
        let elements_size =
            new_cap * std::mem::size_of::<Option<(*mut DlispValue, *mut DlispValue)>>();
        let elements =
            dlisp_gc_malloc(elements_size) as *mut Option<(*mut DlispValue, *mut DlispValue)>;

        for i in 0..new_cap {
            std::ptr::write(elements.add(i), None);
        }

        (*result_map_data).len = 0;
        (*result_map_data).cap = new_cap;
        (*result_map_data).elements = elements;

        // Copy existing elements
        for i in 0..old_cap {
            let opt_ptr = (*old_map_data).elements.add(i);
            if let Some((k, v)) = *opt_ptr {
                dlisp_map_insert(result_map_data, k, v);
            }
        }

        // Insert new element
        dlisp_map_insert(result_map_data, key, val);

        *result_map_val = DlispValue {
            type_: ValueType::Map,
            payload: ValuePayload {
                map_val: result_map_data,
            },
        };
        result_map_val
    }
}

/// # Safety
/// This function is unsafe because it uses raw pointers directly.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_map_get(
    map_val: *mut DlispValue,
    key: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if map_val.is_null() || (*map_val).type_ != ValueType::Map {
            // Like core/src/builtins/map.rs `get`, return nil if not map
            return dlisp_make_nil();
        }
        let map_data = (*map_val).payload.map_val;
        if map_data.is_null() || (*map_data).len == 0 {
            return dlisp_make_nil();
        }

        let cap = (*map_data).cap;
        let hash = dlisp_hash_value(key) as usize;
        let mut idx = hash % cap;
        let start_idx = idx;

        loop {
            let opt_ptr = (*map_data).elements.add(idx);
            if (*opt_ptr).is_none() {
                return dlisp_make_nil();
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
                return dlisp_make_nil();
            }
        }
    }
}
