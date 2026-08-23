use std::ffi::{c_int, c_void};

// Boehm GC bindings (Manual FFI)
#[link(name = "gc")]
unsafe extern "C" {
    pub fn GC_init();
    pub fn GC_malloc(size: usize) -> *mut c_void;
    pub fn GC_call_with_stack_base(
        func: extern "C" fn(*mut c_void, arg: *mut c_void) -> *mut c_void,
        arg: *mut c_void,
    ) -> *mut c_void;
    pub fn GC_allow_register_threads();
    pub fn GC_get_stack_base(stack_base: *mut GC_stack_base) -> c_int;
    pub fn GC_register_my_thread(stack_base: *const GC_stack_base) -> c_int;
    pub fn GC_unregister_my_thread() -> c_int;
}

/// Minimal mirror of Boehm's `struct GC_stack_base`; `mem_base` is its first
/// member on every supported platform.
#[repr(C)]
pub struct GC_stack_base {
    pub mem_base: *mut c_void,
}

static GC_INIT_ONCE: std::sync::Once = std::sync::Once::new();

/// Initialize the Boehm garbage collector.
/// Must be called once at program start before any allocations.
#[unsafe(no_mangle)]
pub extern "C" fn dlisp_gc_init() {
    GC_INIT_ONCE.call_once(|| unsafe {
        GC_init();
        GC_allow_register_threads();
    });
}

/// Allocate memory using Boehm GC.
/// The allocated memory will be automatically garbage collected when unreachable.
#[unsafe(no_mangle)]
pub extern "C" fn dlisp_gc_malloc(size: usize) -> *mut c_void {
    // AOT binaries can allocate before any explicit `dlisp_gc_init` call
    // (e.g. when the first collection is triggered by a large allocation).
    // Lazy init keeps the collector on the registered main thread.
    // Idempotent: backed by `GC_INIT_ONCE`.
    dlisp_gc_init();
    unsafe { GC_malloc(size) }
}
