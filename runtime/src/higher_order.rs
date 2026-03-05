use crate::value::{DlispValue, ValueType};
use crate::{dlisp_make_cons, dlisp_make_nil};

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

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_map(func: *mut DlispValue, list: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if func.is_null() || (*func).type_ != ValueType::Closure {
            return dlisp_make_nil(); // Error: func is not a closure
        }

        let closure = (*func).payload.closure_val;
        let func_ptr: ClosureFunc1 = std::mem::transmute((*closure).func_ptr);
        let env_ptr = (*closure).env;

        // We only support mapping over Lists for now in runtime, matching existing functions.
        // Or if it's a Vector we could support it, but let's do List primarily.

        if list.is_null() || (*list).type_ == ValueType::Nil {
            return dlisp_make_nil();
        }

        if (*list).type_ != ValueType::List {
            return dlisp_make_nil();
        }

        // We build the new list in forward order by keeping track of the tail.
        let mut head: *mut DlispValue = std::ptr::null_mut();
        let mut tail: *mut DlispValue = std::ptr::null_mut();

        let mut current = list;
        while !current.is_null() && (*current).type_ == ValueType::List {
            let list_data = (*current).payload.list_val;
            let item = (*list_data).car;

            // Call the closure with env and item
            let mapped_item = func_ptr(env_ptr, item);

            // Create a new cons cell
            let new_cons = dlisp_make_cons(mapped_item, dlisp_make_nil());

            if head.is_null() {
                head = new_cons;
                tail = new_cons;
            } else {
                // Attach to the end of the new list
                (*(*tail).payload.list_val).cdr = new_cons;
                tail = new_cons;
            }

            current = (*list_data).cdr;
        }

        if head.is_null() {
            dlisp_make_nil()
        } else {
            head
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_filter(func: *mut DlispValue, list: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if func.is_null() || (*func).type_ != ValueType::Closure {
            return dlisp_make_nil();
        }

        let closure = (*func).payload.closure_val;
        let func_ptr: ClosureFunc1 = std::mem::transmute((*closure).func_ptr);
        let env_ptr = (*closure).env;

        if list.is_null() || (*list).type_ == ValueType::Nil {
            return dlisp_make_nil();
        }

        if (*list).type_ != ValueType::List {
            return dlisp_make_nil();
        }

        let mut head: *mut DlispValue = std::ptr::null_mut();
        let mut tail: *mut DlispValue = std::ptr::null_mut();

        let mut current = list;
        while !current.is_null() && (*current).type_ == ValueType::List {
            let list_data = (*current).payload.list_val;
            let item = (*list_data).car;

            // Call the closure
            let condition = func_ptr(env_ptr, item);
            let is_truthy = !condition.is_null()
                && (*condition).type_ != ValueType::Nil
                && ((*condition).type_ != ValueType::Bool || (*condition).payload.bool_val);

            if is_truthy {
                let new_cons = dlisp_make_cons(item, dlisp_make_nil());
                if head.is_null() {
                    head = new_cons;
                    tail = new_cons;
                } else {
                    (*(*tail).payload.list_val).cdr = new_cons;
                    tail = new_cons;
                }
            }

            current = (*list_data).cdr;
        }

        if head.is_null() {
            dlisp_make_nil()
        } else {
            head
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_reduce(
    func: *mut DlispValue,
    init: *mut DlispValue,
    list: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if func.is_null() || (*func).type_ != ValueType::Closure {
            return init; // Better than nil if func is invalid
        }

        let closure = (*func).payload.closure_val;
        let func_ptr: ClosureFunc2 = std::mem::transmute((*closure).func_ptr);
        let env_ptr = (*closure).env;

        if list.is_null() || (*list).type_ == ValueType::Nil {
            return init;
        }

        if (*list).type_ != ValueType::List {
            return init;
        }

        let mut acc = init;
        let mut current = list;

        while !current.is_null() && (*current).type_ == ValueType::List {
            let list_data = (*current).payload.list_val;
            let item = (*list_data).car;

            // acc = func(env, acc, item)
            acc = func_ptr(env_ptr, acc, item);

            current = (*list_data).cdr;
        }

        acc
    }
}
