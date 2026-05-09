#[cfg(test)]
mod tests {
    use crate::value::{DlispValue, ValueType};
    use crate::{
        dlisp_cons, dlisp_eq, dlisp_gc_init, dlisp_keyword_p, dlisp_make_cons, dlisp_make_int,
        dlisp_make_keyword, dlisp_make_map, dlisp_make_string, dlisp_map_assoc, dlisp_map_get,
        dlisp_map_p, dlisp_type_of,
    };
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

    #[test]
    fn test_cons_to_non_list_makes_proper_list() {
        dlisp_gc_init();
        let val1 = dlisp_make_int(1);
        let val2 = dlisp_make_int(2);

        let cons = unsafe { dlisp_cons(val1, val2) };
        unsafe {
            assert_eq!((*cons).type_, ValueType::List);
            let first = *(*cons).payload.list_val;
            assert_eq!((*first.car).payload.int_val, 1);
            assert_eq!((*first.cdr).type_, ValueType::List);

            let second = *(*first.cdr).payload.list_val;
            assert_eq!((*second.car).payload.int_val, 2);
            assert_eq!((*second.cdr).type_, ValueType::Nil);
        }
    }

    // --- Keyword tests ---

    #[test]
    fn test_make_keyword() {
        dlisp_gc_init();
        let s = CString::new("hello").unwrap();
        let s_ptr = s.as_ptr() as *mut i8;

        let val_ptr = unsafe { dlisp_make_keyword(s_ptr) };
        unsafe {
            assert!(!val_ptr.is_null());
            assert_eq!((*val_ptr).type_, ValueType::Keyword);
        }
    }

    #[test]
    fn test_keyword_predicate() {
        dlisp_gc_init();
        let s = CString::new("test").unwrap();
        let kw = unsafe { dlisp_make_keyword(s.as_ptr() as *mut i8) };
        let result = unsafe { dlisp_keyword_p(kw) };
        unsafe {
            assert_eq!((*result).type_, ValueType::Bool);
            assert!((*result).payload.bool_val);
        }

        // Non-keyword should return false
        let int_val = dlisp_make_int(42);
        let result2 = unsafe { dlisp_keyword_p(int_val) };
        unsafe {
            assert_eq!((*result2).type_, ValueType::Bool);
            assert!(!(*result2).payload.bool_val);
        }
    }

    #[test]
    fn test_keyword_equality() {
        dlisp_gc_init();
        let s1 = CString::new("same").unwrap();
        let s2 = CString::new("same").unwrap();
        let s3 = CString::new("diff").unwrap();

        let kw1 = unsafe { dlisp_make_keyword(s1.as_ptr() as *mut i8) };
        let kw2 = unsafe { dlisp_make_keyword(s2.as_ptr() as *mut i8) };
        let kw3 = unsafe { dlisp_make_keyword(s3.as_ptr() as *mut i8) };

        // Same name => equal
        let eq_result = unsafe { dlisp_eq(kw1, kw2) };
        unsafe {
            assert_eq!((*eq_result).type_, ValueType::Bool);
            assert!((*eq_result).payload.bool_val);
        }

        // Different name => not equal
        let neq_result = unsafe { dlisp_eq(kw1, kw3) };
        unsafe {
            assert_eq!((*neq_result).type_, ValueType::Bool);
            assert!(!(*neq_result).payload.bool_val);
        }
    }

    // --- Map tests ---

    #[test]
    fn test_make_map() {
        dlisp_gc_init();
        let m = unsafe { dlisp_make_map() };
        unsafe {
            assert!(!m.is_null());
            assert_eq!((*m).type_, ValueType::Map);
        }
    }

    #[test]
    fn test_map_predicate() {
        dlisp_gc_init();
        let m = unsafe { dlisp_make_map() };
        let result = unsafe { dlisp_map_p(m) };
        unsafe {
            assert_eq!((*result).type_, ValueType::Bool);
            assert!((*result).payload.bool_val);
        }

        // Non-map should return false
        let int_val = dlisp_make_int(42);
        let result2 = unsafe { dlisp_map_p(int_val) };
        unsafe {
            assert_eq!((*result2).type_, ValueType::Bool);
            assert!(!(*result2).payload.bool_val);
        }
    }

    #[test]
    fn test_map_assoc_and_get() {
        dlisp_gc_init();
        let m = unsafe { dlisp_make_map() };

        let key = CString::new("a").unwrap();
        let kw = unsafe { dlisp_make_keyword(key.as_ptr() as *mut i8) };
        let val = dlisp_make_int(42);

        // assoc a key/value
        let m2 = unsafe { dlisp_map_assoc(m, kw, val) };
        unsafe {
            assert!(!m2.is_null());
            assert_eq!((*m2).type_, ValueType::Map);
        }

        // get the value back
        let retrieved = unsafe { dlisp_map_get(m2, kw) };
        unsafe {
            assert!(!retrieved.is_null());
            assert_eq!((*retrieved).type_, ValueType::Int);
            assert_eq!((*retrieved).payload.int_val, 42);
        }
    }

    #[test]
    fn test_map_get_nonexistent_key() {
        dlisp_gc_init();
        let m = unsafe { dlisp_make_map() };
        let key = CString::new("missing").unwrap();
        let kw = unsafe { dlisp_make_keyword(key.as_ptr() as *mut i8) };

        let result = unsafe { dlisp_map_get(m, kw) };
        unsafe {
            assert!(!result.is_null());
            assert_eq!((*result).type_, ValueType::Nil);
        }
    }

    #[test]
    fn test_map_assoc_update_existing_key() {
        dlisp_gc_init();
        let m = unsafe { dlisp_make_map() };

        let key = CString::new("x").unwrap();
        let kw = unsafe { dlisp_make_keyword(key.as_ptr() as *mut i8) };

        // First insert
        let m2 = unsafe { dlisp_map_assoc(m, kw, dlisp_make_int(10)) };
        // Update same key
        let m3 = unsafe { dlisp_map_assoc(m2, kw, dlisp_make_int(20)) };

        // Should get updated value
        let retrieved = unsafe { dlisp_map_get(m3, kw) };
        unsafe {
            assert_eq!((*retrieved).type_, ValueType::Int);
            assert_eq!((*retrieved).payload.int_val, 20);
        }

        // Length should still be 1 (not 2)
        unsafe {
            let map_data = (*m3).payload.map_val;
            assert_eq!((*map_data).len, 1);
        }
    }

    #[test]
    fn test_map_type_of() {
        dlisp_gc_init();
        let m = unsafe { dlisp_make_map() };
        let result = unsafe { dlisp_type_of(m) };
        unsafe {
            assert_eq!((*result).type_, ValueType::String);
            let c_str = std::ffi::CStr::from_ptr((*result).payload.str_val);
            assert_eq!(c_str.to_string_lossy(), "map");
        }
    }

    #[test]
    fn test_map_equality() {
        dlisp_gc_init();
        let m1 = unsafe { dlisp_make_map() };
        let m2 = unsafe { dlisp_make_map() };

        let key = CString::new("k").unwrap();
        let kw = unsafe { dlisp_make_keyword(key.as_ptr() as *mut i8) };

        let m1_with = unsafe { dlisp_map_assoc(m1, kw, dlisp_make_int(1)) };
        let m2_with = unsafe { dlisp_map_assoc(m2, kw, dlisp_make_int(1)) };

        // Same content => equal
        let eq_result = unsafe { dlisp_eq(m1_with, m2_with) };
        unsafe {
            assert_eq!((*eq_result).type_, ValueType::Bool);
            assert!((*eq_result).payload.bool_val);
        }

        // Different content => not equal
        let m3_with = unsafe { dlisp_map_assoc(m2, kw, dlisp_make_int(2)) };
        let neq_result = unsafe { dlisp_eq(m1_with, m3_with) };
        unsafe {
            assert_eq!((*neq_result).type_, ValueType::Bool);
            assert!(!(*neq_result).payload.bool_val);
        }
    }

    #[test]
    fn test_map_get_on_non_map() {
        dlisp_gc_init();
        let int_val = dlisp_make_int(42);
        let key = CString::new("k").unwrap();
        let kw = unsafe { dlisp_make_keyword(key.as_ptr() as *mut i8) };

        // get on non-map should return nil
        let result = unsafe { dlisp_map_get(int_val, kw) };
        unsafe {
            assert!(!result.is_null());
            assert_eq!((*result).type_, ValueType::Nil);
        }
    }

    #[test]
    fn test_map_multiple_keys() {
        dlisp_gc_init();
        let m = unsafe { dlisp_make_map() };

        let k1 = CString::new("one").unwrap();
        let kw1 = unsafe { dlisp_make_keyword(k1.as_ptr() as *mut i8) };
        let k2 = CString::new("two").unwrap();
        let kw2 = unsafe { dlisp_make_keyword(k2.as_ptr() as *mut i8) };
        let k3 = CString::new("three").unwrap();
        let kw3 = unsafe { dlisp_make_keyword(k3.as_ptr() as *mut i8) };

        let m = unsafe { dlisp_map_assoc(m, kw1, dlisp_make_int(1)) };
        let m = unsafe { dlisp_map_assoc(m, kw2, dlisp_make_int(2)) };
        let m = unsafe { dlisp_map_assoc(m, kw3, dlisp_make_int(3)) };

        // Verify length
        unsafe {
            let map_data = (*m).payload.map_val;
            assert_eq!((*map_data).len, 3);
        }

        // Verify all values
        unsafe {
            let v1 = dlisp_map_get(m, kw1);
            assert_eq!((*v1).payload.int_val, 1);
            let v2 = dlisp_map_get(m, kw2);
            assert_eq!((*v2).payload.int_val, 2);
            let v3 = dlisp_map_get(m, kw3);
            assert_eq!((*v3).payload.int_val, 3);
        }
    }
}
