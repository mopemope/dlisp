#[cfg(test)]
mod tests {
    use crate::value::{DlispValue, ValueType};
    use crate::{
        dlisp_cons, dlisp_eq, dlisp_gc_init, dlisp_keyword_p, dlisp_make_cons, dlisp_make_int,
        dlisp_make_keyword, dlisp_make_map, dlisp_make_nil, dlisp_make_string, dlisp_make_vector,
        dlisp_map_assoc, dlisp_map_get, dlisp_map_p, dlisp_type_of, dlisp_vector_count,
        dlisp_vector_get, dlisp_vector_push,
    };
    use std::ffi::CString;

    unsafe extern "C" fn inc1(_env: *mut DlispValue, x: *mut DlispValue) -> *mut DlispValue {
        unsafe { crate::dlisp_add(x, crate::dlisp_make_int(1)) }
    }

    unsafe extern "C" fn passthrough(_env: *mut DlispValue, x: *mut DlispValue) -> *mut DlispValue {
        x
    }

    unsafe extern "C" fn add2(
        _env: *mut DlispValue,
        a: *mut DlispValue,
        b: *mut DlispValue,
    ) -> *mut DlispValue {
        unsafe { crate::dlisp_add(a, b) }
    }

    fn unbox_bool(p: *mut DlispValue) -> bool {
        unsafe {
            assert!(!p.is_null());
            let val = *p;
            assert_eq!(val.type_, ValueType::Bool);
            val.payload.bool_val
        }
    }

    fn unbox_int(p: *mut DlispValue) -> i64 {
        unsafe {
            assert!(!p.is_null());
            let val = *p;
            assert_eq!(val.type_, ValueType::Int);
            val.payload.int_val
        }
    }

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
    fn test_vector_count() {
        dlisp_gc_init();

        // Vector: length from stored len
        let vec = dlisp_make_vector(2);
        unsafe {
            dlisp_vector_push(vec, dlisp_make_int(1));
            dlisp_vector_push(vec, dlisp_make_int(2));
            let count = *dlisp_vector_count(vec);
            assert_eq!(count.type_, ValueType::Int);
            assert_eq!(count.payload.int_val, 2);
        }

        // List: walk cons cells (1 2 3)
        unsafe {
            let l3 = dlisp_make_cons(dlisp_make_int(3), dlisp_make_nil());
            let l2 = dlisp_make_cons(dlisp_make_int(2), l3);
            let l1 = dlisp_make_cons(dlisp_make_int(1), l2);
            let count = *dlisp_vector_count(l1);
            assert_eq!(count.type_, ValueType::Int);
            assert_eq!(count.payload.int_val, 3);
        }

        // Nil counts as 0
        unsafe {
            let count = *dlisp_vector_count(dlisp_make_nil());
            assert_eq!(count.type_, ValueType::Int);
            assert_eq!(count.payload.int_val, 0);
        }
    }

    #[test]
    fn test_higher_order_supports_vectors() {
        dlisp_gc_init();

        unsafe {
            let v = dlisp_make_vector(3);
            dlisp_vector_push(v, dlisp_make_int(1));
            dlisp_vector_push(v, dlisp_make_int(2));
            dlisp_vector_push(v, dlisp_make_int(3));

            // map over a vector returns a vector
            let inc = crate::dlisp_make_closure(std::ptr::null_mut(), inc1 as *const _);
            let mapped = crate::dlisp_map(inc, v);
            assert_eq!((*mapped).type_, ValueType::Vector);
            assert_eq!(unbox_int(dlisp_vector_get(mapped, dlisp_make_int(0))), 2);
            assert_eq!(unbox_int(dlisp_vector_get(mapped, dlisp_make_int(2))), 4);

            // filter over a vector keeps truthy elements (Int 0 is falsy)
            let vf = dlisp_make_vector(3);
            dlisp_vector_push(vf, dlisp_make_int(0));
            dlisp_vector_push(vf, dlisp_make_int(1));
            dlisp_vector_push(vf, dlisp_make_int(2));
            let passthru = crate::dlisp_make_closure(std::ptr::null_mut(), passthrough as *const _);
            let filtered = crate::dlisp_filter(passthru, vf);
            assert_eq!((*filtered).type_, ValueType::Vector);
            let first = dlisp_vector_get(filtered, dlisp_make_int(0));
            assert_eq!(unbox_int(first), 1);

            // reduce over a vector folds with the closure
            let sum = crate::dlisp_make_closure(std::ptr::null_mut(), add2 as *const _);
            let total = crate::dlisp_reduce(sum, dlisp_make_int(0), v);
            assert_eq!(unbox_int(total), 6);
        }
    }

    #[test]
    fn test_vector_to_list() {
        dlisp_gc_init();

        unsafe {
            let v = dlisp_make_vector(2);
            dlisp_vector_push(v, dlisp_make_int(1));
            dlisp_vector_push(v, dlisp_make_int(2));

            let list = crate::dlisp_vector_to_list(v);
            assert_eq!((*list).type_, ValueType::List);
            assert_eq!(unbox_int(crate::dlisp_car(list)), 1);
            let rest = crate::dlisp_cdr(list);
            assert_eq!(unbox_int(crate::dlisp_car(rest)), 2);
            assert_eq!((*crate::dlisp_cdr(rest)).type_, ValueType::Nil);

            // Lists pass through and nil stays nil
            assert_eq!(crate::dlisp_vector_to_list(list), list);
            assert_eq!(
                (*crate::dlisp_vector_to_list(dlisp_make_nil())).type_,
                ValueType::Nil
            );
        }
    }

    #[test]
    fn test_some_every_find_for_each() {
        dlisp_gc_init();

        unsafe {
            let make_vec = |elems: &[i64]| {
                let v = dlisp_make_vector(elems.len());
                for e in elems {
                    dlisp_vector_push(v, dlisp_make_int(*e));
                }
                v
            };
            let passthru = crate::dlisp_make_closure(std::ptr::null_mut(), passthrough as *const _);

            // some: first truthy element (passthrough returns the element)
            let some_val = crate::dlisp_some(passthru, make_vec(&[0, 1, 2]));
            assert_eq!(unbox_int(some_val), 1);

            // all falsy -> nil
            assert_eq!(
                (*crate::dlisp_some(passthru, make_vec(&[0]))).type_,
                ValueType::Nil
            );

            // every: vacuously true, false on any falsy element
            assert!(unbox_bool(crate::dlisp_every(passthru, make_vec(&[1, 2]))));
            assert!(!unbox_bool(crate::dlisp_every(passthru, make_vec(&[1, 0]))));
            assert!(unbox_bool(crate::dlisp_every(passthru, make_vec(&[]))));

            // find: first truthy element itself
            let found = crate::dlisp_find(passthru, make_vec(&[0, 3, 4]));
            assert_eq!(unbox_int(found), 3);
            assert_eq!(
                (*crate::dlisp_find(passthru, make_vec(&[0]))).type_,
                ValueType::Nil
            );

            // for-each returns nil
            assert_eq!(
                (*crate::dlisp_for_each(passthru, make_vec(&[1]))).type_,
                ValueType::Nil
            );
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

    #[test]
    fn test_dlisp_eq_deep_collections() {
        dlisp_gc_init();

        unsafe {
            let make_pair_list = |a: *mut DlispValue, b: *mut DlispValue| {
                dlisp_make_cons(a, dlisp_make_cons(b, dlisp_make_nil()))
            };

            // Equal lists
            let l1 = make_pair_list(dlisp_make_int(1), dlisp_make_int(2));
            let l2 = make_pair_list(dlisp_make_int(1), dlisp_make_int(2));
            assert!(unbox_bool(dlisp_eq(l1, l2)));

            // Unequal lists (different element)
            let l3 = make_pair_list(dlisp_make_int(1), dlisp_make_int(3));
            assert!(!unbox_bool(dlisp_eq(l1, l3)));

            // Unequal lists (different length)
            let shorter = dlisp_make_cons(dlisp_make_int(1), dlisp_make_nil());
            assert!(!unbox_bool(dlisp_eq(l1, shorter)));

            // Nested list equality
            let n1 = dlisp_make_cons(
                make_pair_list(dlisp_make_int(2), dlisp_make_int(3)),
                dlisp_make_nil(),
            );
            let n2 = dlisp_make_cons(
                make_pair_list(dlisp_make_int(2), dlisp_make_int(3)),
                dlisp_make_nil(),
            );
            assert!(unbox_bool(dlisp_eq(n1, n2)));

            // Equal vectors
            let v1 = dlisp_make_vector(2);
            dlisp_vector_push(v1, dlisp_make_int(1));
            dlisp_vector_push(v1, dlisp_make_int(2));
            let v2 = dlisp_make_vector(2);
            dlisp_vector_push(v2, dlisp_make_int(1));
            dlisp_vector_push(v2, dlisp_make_int(2));
            assert!(unbox_bool(dlisp_eq(v1, v2)));

            // Unequal vectors
            let v3 = dlisp_make_vector(2);
            dlisp_vector_push(v3, dlisp_make_int(9));
            dlisp_vector_push(v3, dlisp_make_int(2));
            assert!(!unbox_bool(dlisp_eq(v1, v3)));

            // Mixed container types are not equal (interpreter parity)
            assert!(!unbox_bool(dlisp_eq(l1, v1)));
        }
    }

    #[test]
    fn test_dlisp_vector_get_on_list() {
        dlisp_gc_init();

        unsafe {
            let l3 = dlisp_make_cons(dlisp_make_int(30), dlisp_make_nil());
            let l2 = dlisp_make_cons(dlisp_make_int(20), l3);
            let l1 = dlisp_make_cons(dlisp_make_int(10), l2);

            assert_eq!(unbox_int(dlisp_vector_get(l1, dlisp_make_int(0))), 10);
            assert_eq!(unbox_int(dlisp_vector_get(l1, dlisp_make_int(2))), 30);

            // Out of range and negative indices yield nil
            let oob = *dlisp_vector_get(l1, dlisp_make_int(5));
            assert_eq!(oob.type_, ValueType::Nil);
            let neg = *dlisp_vector_get(l1, dlisp_make_int(-1));
            assert_eq!(neg.type_, ValueType::Nil);
        }
    }
}
