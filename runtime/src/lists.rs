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

// --- Compiled list helpers (used by JIT/AOT codegen) ---

use crate::constructors::dlisp_make_cons;
use crate::vectors::{dlisp_make_vector, dlisp_vector_push};

pub(crate) unsafe fn build_list(elems: &[*mut DlispValue]) -> *mut DlispValue {
    unsafe {
        let mut acc = dlisp_make_nil();
        for e in elems.iter().rev() {
            acc = dlisp_make_cons(*e, acc);
        }
        acc
    }
}

unsafe fn list_to_vec(val: *mut DlispValue) -> Option<Vec<*mut DlispValue>> {
    unsafe {
        let mut out = Vec::new();
        let mut cur = val;
        loop {
            if cur.is_null() || (*cur).type_ == ValueType::Nil {
                break;
            }
            if (*cur).type_ != ValueType::List {
                return None;
            }
            let data = (*cur).payload.list_val;
            if data.is_null() {
                break;
            }
            out.push((*data).car);
            cur = (*data).cdr;
        }
        Some(out)
    }
}

unsafe fn vector_to_vec(val: *mut DlispValue) -> Vec<*mut DlispValue> {
    unsafe {
        let mut out = Vec::new();
        let vec_data = (*val).payload.vector_val;
        if !vec_data.is_null() {
            let len = (*vec_data).len;
            let data = (*vec_data).data;
            for i in 0..len {
                out.push(*data.add(i));
            }
        }
        out
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_append(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let mut elems: Vec<*mut DlispValue> = Vec::new();
        for v in [a, b] {
            if v.is_null() || (*v).type_ == ValueType::Nil {
                continue;
            }
            match (*v).type_ {
                ValueType::List => match list_to_vec(v) {
                    Some(mut l) => elems.append(&mut l),
                    None => {
                        eprintln!("Type Error: append requires lists or vectors");
                        std::process::abort();
                    }
                },
                ValueType::Vector => elems.append(&mut vector_to_vec(v)),
                _ => {
                    eprintln!("Type Error: append requires lists or vectors");
                    std::process::abort();
                }
            }
        }
        build_list(&elems)
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_reverse(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if val.is_null() || (*val).type_ == ValueType::Nil {
            return dlisp_make_nil();
        }
        match (*val).type_ {
            ValueType::List => {
                let mut elems = list_to_vec(val).unwrap_or_default();
                elems.reverse();
                build_list(&elems)
            }
            ValueType::Vector => {
                let mut elems = vector_to_vec(val);
                elems.reverse();
                let rv = dlisp_make_vector(elems.len());
                for e in &elems {
                    dlisp_vector_push(rv, *e);
                }
                rv
            }
            _ => {
                eprintln!("Type Error: reverse requires a list or vector");
                std::process::abort();
            }
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_last(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if val.is_null() || (*val).type_ == ValueType::Nil {
            return dlisp_make_nil();
        }
        match (*val).type_ {
            ValueType::List => match list_to_vec(val) {
                Some(elems) => elems.last().copied().unwrap_or_else(|| dlisp_make_nil()),
                None => dlisp_make_nil(),
            },
            ValueType::Vector => {
                let elems = vector_to_vec(val);
                elems.last().copied().unwrap_or_else(|| dlisp_make_nil())
            }
            _ => {
                eprintln!("Type Error: last requires a list or vector argument");
                std::process::abort();
            }
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_butlast(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if val.is_null() || (*val).type_ == ValueType::Nil {
            return dlisp_make_nil();
        }
        match (*val).type_ {
            ValueType::List => {
                let mut elems = list_to_vec(val).unwrap_or_default();
                if elems.is_empty() {
                    return dlisp_make_nil();
                }
                elems.pop();
                build_list(&elems)
            }
            ValueType::Vector => {
                let elems = vector_to_vec(val);
                if elems.is_empty() {
                    return dlisp_make_nil();
                }
                let rv = dlisp_make_vector(elems.len() - 1);
                for e in &elems[..elems.len() - 1] {
                    dlisp_vector_push(rv, *e);
                }
                rv
            }
            _ => {
                eprintln!("Type Error: butlast requires a list or vector argument");
                std::process::abort();
            }
        }
    }
}

unsafe fn flatten_into(val: *mut DlispValue, out: &mut Vec<*mut DlispValue>) {
    unsafe {
        if val.is_null() {
            return;
        }
        match (*val).type_ {
            ValueType::List | ValueType::Vector => {
                let elems = if (*val).type_ == ValueType::Vector {
                    vector_to_vec(val)
                } else {
                    list_to_vec(val).unwrap_or_default()
                };
                for e in &elems {
                    flatten_into(*e, out);
                }
            }
            ValueType::Nil => {}
            _ => out.push(val),
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_flatten(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let mut out: Vec<*mut DlispValue> = Vec::new();
        flatten_into(val, &mut out);
        build_list(&out)
    }
}

unsafe fn unbox_count(n_ptr: *mut DlispValue, op: &str) -> usize {
    unsafe {
        if n_ptr.is_null() || (*n_ptr).type_ != ValueType::Int {
            eprintln!("Type Error: {} first argument must be an integer", op);
            std::process::abort();
        }
        let n = (*n_ptr).payload.int_val;
        if n < 0 {
            eprintln!(
                "Type Error: {} first argument must be a non-negative integer",
                op
            );
            std::process::abort();
        }
        n as usize
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_take(
    n_ptr: *mut DlispValue,
    val: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        let n = unbox_count(n_ptr, "take");
        if val.is_null() || (*val).type_ == ValueType::Nil {
            return dlisp_make_nil();
        }
        match (*val).type_ {
            ValueType::List => {
                let elems = list_to_vec(val).unwrap_or_default();
                let take_len = elems.len().min(n);
                build_list(&elems[..take_len])
            }
            ValueType::Vector => {
                let elems = vector_to_vec(val);
                let take_len = elems.len().min(n);
                let rv = dlisp_make_vector(take_len);
                for e in &elems[..take_len] {
                    dlisp_vector_push(rv, *e);
                }
                rv
            }
            _ => {
                eprintln!("Type Error: take second argument must be a list or vector");
                std::process::abort();
            }
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_drop(
    n_ptr: *mut DlispValue,
    val: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        let n = unbox_count(n_ptr, "drop");
        if val.is_null() || (*val).type_ == ValueType::Nil {
            return dlisp_make_nil();
        }
        match (*val).type_ {
            ValueType::List => {
                let elems = list_to_vec(val).unwrap_or_default();
                let drop_len = elems.len().min(n);
                build_list(&elems[drop_len..])
            }
            ValueType::Vector => {
                let elems = vector_to_vec(val);
                let drop_len = elems.len().min(n);
                let rv = dlisp_make_vector(elems.len() - drop_len);
                for e in &elems[drop_len..] {
                    dlisp_vector_push(rv, *e);
                }
                rv
            }
            _ => {
                eprintln!("Type Error: drop second argument must be a list or vector");
                std::process::abort();
            }
        }
    }
}
