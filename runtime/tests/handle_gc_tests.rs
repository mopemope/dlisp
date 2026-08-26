//! GC-pressure regression tests for concurrency handles.
//!
//! Kept in a dedicated test binary: the stress test below triggers real
//! Boehm stop-the-world collections, which cannot run concurrently with the
//! other lib-unit tests in this sandbox (unregistered harness threads make
//! suspension abort). A single-test target gives the collector a quiet
//! process.

use dlisp_runtime::value::ValueType;
use dlisp_runtime::{dlisp_atom_deref, dlisp_atom_new, dlisp_gc_init};

/// Smoke test: thousands of atoms plus channel churn under repeated real
/// collections, verifying the handle path stays intact end to end.
///
/// Note this cannot by itself detect a missing pinning registry on builds
/// where the allocator is conservatively scanned (heap residue keeps
/// wrappers alive); the registry remains required for correctness on
/// platforms without such masking.
#[test]
fn handles_survive_gc_pressure() {
    unsafe {
        dlisp_gc_init();
        const N: usize = 4000;
        let atoms: Vec<*mut dlisp_runtime::value::DlispValue> = (0..N as i64)
            .map(|i| dlisp_atom_new(dlisp_runtime::dlisp_make_int(i)))
            .collect();
        // Churn enough GC memory to force multiple collections while the
        // only strong references live inside the registry.
        for _ in 0..200_000u32 {
            let a = dlisp_runtime::dlisp_make_int(1);
            let b = dlisp_runtime::dlisp_cons(a, dlisp_runtime::dlisp_make_nil());
            std::hint::black_box(b);
        }
        // Clobber the stack region where wrapper addresses lingered,
        // including the deeper frames the allocation path used.
        fn burn(depth: usize) -> u64 {
            if depth == 0 {
                return 0;
            }
            let scratch = [0xdead_beef_u64; 32];
            std::hint::black_box(&scratch);
            burn(depth - 1).wrapping_add(1)
        }
        for _ in 0..8 {
            std::hint::black_box(burn(4_000));
        }
        let mut bad = 0usize;
        for (i, atom) in atoms.iter().enumerate() {
            let cur = dlisp_atom_deref(*atom);
            if (*cur).type_ != ValueType::Int || (*cur).payload.int_val != i as i64 {
                bad += 1;
            }
        }
        println!("BAD_COUNT={}", bad);
        assert_eq!(bad, 0);
    }
}
