#[cfg(test)]
mod tests {
    use crate::value::{DlispValue, ValueType};
    use crate::{dlisp_gc_init, dlisp_make_cons, dlisp_make_int, dlisp_make_string};
    use std::ffi::CString;

    #[test]
    fn test_value_layout() {
        // Verify size and alignment to match assumed C layout
        // DlispValue: tag (might be 4 or 8 bytes depending on enum repr) + padding + payload (8 bytes)
        // With #[repr(C)], enum defaults to c_int (usually 4 bytes).
        // On 64-bit, alignment is 8.
        // So: type(4) + pad(4) + payload(8) = 16 bytes.
        assert_eq!(std::mem::size_of::<DlispValue>(), 16);
        assert_eq!(std::mem::align_of::<DlispValue>(), 8);
    }

    #[test]
    fn test_make_int() {
        // Initialize GC not strictly needed for unit test if we mock or if single threaded test doesn't actually trigger GC,
        // but dlisp_make_int calls dlisp_gc_malloc which calls GC_malloc.
        // We might need to call dlisp_gc_init() once.
        dlisp_gc_init();

        let val_ptr = dlisp_make_int(42);
        unsafe {
            assert!(!val_ptr.is_null());
            let val = *val_ptr;
            assert_eq!(val.type_, ValueType::Int);
            assert_eq!(val.payload.int_val, 42);
        }
    }

    #[test]
    fn test_make_string() {
        dlisp_gc_init();
        let s = CString::new("hello").unwrap();
        // into_raw passes ownership to C, but here we just want a pointer for tests
        let s_ptr = s.as_ptr() as *mut i8;

        let val_ptr = unsafe { dlisp_make_string(s_ptr) };
        unsafe {
            assert!(!val_ptr.is_null());
            let val = *val_ptr;
            assert_eq!(val.type_, ValueType::String);
            assert_eq!(val.payload.str_val, s_ptr);
        }
    }

    #[test]
    fn test_make_cons() {
        dlisp_gc_init();
        let val1 = dlisp_make_int(1);
        let val2 = dlisp_make_int(2);

        let cons = unsafe { dlisp_make_cons(val1, val2) };
        unsafe {
            let val = *cons;
            assert_eq!(val.type_, ValueType::List);
            let list_data = *val.payload.list_val;

            let car = *list_data.car;
            let cdr = *list_data.cdr;

            assert_eq!(car.payload.int_val, 1);
            assert_eq!(cdr.payload.int_val, 2);
        }
    }
}
