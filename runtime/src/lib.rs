//! FFI runtime for AOT-compiled and JIT-compiled dlisp programs.
//!
//! The crate-root `dlisp_*` names are re-exports from the feature modules
//! below; the exported C symbols are unchanged regardless of module layout.

pub mod arith;
pub mod cmp;
pub mod collections;
pub mod constructors;
pub mod gc;
pub mod higher_order;
pub mod io;
pub mod lists;
pub mod maps;
pub mod os;
pub mod predicates;
pub mod print;
pub mod strings;
pub mod sys;
pub mod task;
pub mod value;
pub mod vectors;

mod verify_tests;

// Re-exports preserving the historical crate-root FFI API (`dlisp_runtime::dlisp_*`).
pub use arith::{dlisp_add, dlisp_div, dlisp_mod, dlisp_mul, dlisp_sub};
pub use cmp::{dlisp_eq, dlisp_gt, dlisp_gte, dlisp_is_truthy, dlisp_lt, dlisp_lte, dlisp_neq};
pub use collections::{dlisp_conj, dlisp_get};
pub use constructors::{
    dlisp_cons, dlisp_make_bool, dlisp_make_closure, dlisp_make_cons, dlisp_make_float,
    dlisp_make_int, dlisp_make_keyword, dlisp_make_nil, dlisp_make_string, dlisp_make_symbol,
};
pub use gc::{GC_allow_register_threads, GC_call_with_stack_base, GC_init, GC_malloc};
pub use gc::{dlisp_gc_init, dlisp_gc_malloc};
pub use io::dlisp_read_file;
pub use lists::{dlisp_car, dlisp_cdr};
pub use maps::{dlisp_make_map, dlisp_map_assoc, dlisp_map_get};
pub use predicates::{
    dlisp_keyword_p, dlisp_list_p, dlisp_map_p, dlisp_nil_p, dlisp_number_p, dlisp_string_p,
    dlisp_symbol_p, dlisp_type_of, dlisp_vector_p,
};
pub use print::dlisp_print;
pub use strings::{dlisp_str, dlisp_string_append, dlisp_string_length, dlisp_substring};
pub use task::{Closure, dlisp_main, dlisp_sleep, dlisp_spawn};
