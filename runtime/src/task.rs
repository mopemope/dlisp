use std::ffi::c_void;

use tokio::runtime::Runtime;

use crate::constructors::dlisp_make_int;
use crate::gc::{
    GC_call_with_stack_base, GC_get_stack_base, GC_register_my_thread, GC_stack_base,
    GC_unregister_my_thread, dlisp_gc_init,
};
use crate::value::{DlispValue, ValueType};

const GC_SUCCESS: i32 = 0;

/// Runs `f` on a GC-registered thread.
///
/// Compiled programs allocate from worker threads (tokio blocking pool);
/// Boehm aborts with "Collecting from unknown thread" if such a thread
/// triggers a collection without being registered. Registration is
/// per-thread and must be undone before the pooled thread exits.
pub fn with_gc_registered<T>(f: impl FnOnce() -> T) -> T {
    dlisp_gc_init();
    unsafe {
        let mut stack_base = GC_stack_base {
            mem_base: std::ptr::null_mut(),
        };
        let registered = GC_get_stack_base(&mut stack_base) == GC_SUCCESS
            && GC_register_my_thread(&stack_base) == GC_SUCCESS;
        let out = f();
        if registered {
            GC_unregister_my_thread();
        }
        out
    }
}

// Wrapper callbacks for GC_call_with_stack_base

extern "C" fn run_user_main_wrapper(_sb: *mut c_void, arg: *mut c_void) -> *mut c_void {
    unsafe {
        // arg is user_main_ptr cast to void*
        // transmute back to fn
        let user_main_ptr: extern "C" fn(*mut c_void) -> i64 = std::mem::transmute(arg);
        user_main_ptr(std::ptr::null_mut());
        // An uncaught `throw` escaping the user main returns the sentinel
        // and leaves the thrown value pending. Report it and fail the
        // process so compiled programs surface errors like the interpreter
        // does (which prints "Error in main:" and exits 1).
        if crate::errors::dlisp_thrown_pending() != 0 {
            let _thrown = crate::errors::dlisp_take_thrown();
            eprintln!("\x1b[31mError in main:\x1b[0m DLISP_THROW");
            std::process::exit(1);
        }
        std::ptr::null_mut()
    }
}

extern "C" fn run_closure_wrapper(_sb: *mut c_void, arg: *mut c_void) -> *mut c_void {
    unsafe {
        let closure_ptr = arg as *mut Closure;
        let closure = &*closure_ptr;
        (closure.func)(closure.env);
        // An uncaught `throw` inside a spawned task returns the sentinel and
        // leaves the thrown value pending on this pooled thread's
        // thread-local state. Clear it so later tasks on the same thread
        // never observe a stale pending error.
        let thrown = crate::errors::dlisp_take_thrown();
        if !thrown.is_null() {
            eprintln!("Uncaught throw in spawned task");
        }
        std::ptr::null_mut()
    }
}

/// # Safety
/// This function is unsafe because it executes an arbitrary function pointer.
/// The caller must ensure that `user_main_ptr` is a valid function pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_main(user_main_ptr: extern "C" fn(*mut c_void) -> i64) {
    // Initialize GC
    dlisp_gc_init();

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        // Run the user's main function
        // Pass NULL as env
        let handle = tokio::task::spawn_blocking(move || {
            with_gc_registered(|| unsafe {
                GC_call_with_stack_base(run_user_main_wrapper, user_main_ptr as *mut c_void);
            });
        });

        handle.await.unwrap();
    });
}

#[repr(C)]
pub struct Closure {
    pub func: extern "C" fn(*mut c_void) -> i64,
    pub env: *mut c_void,
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `closure_ptr` points to a valid closure `DlispValue`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_spawn(closure_ptr: *mut DlispValue) {
    if closure_ptr.is_null() {
        eprintln!("Type Error: spawn requires a closure");
        std::process::abort();
    }

    let closure_data = unsafe {
        if (*closure_ptr).type_ != ValueType::Closure {
            eprintln!("Type Error: spawn requires a closure");
            std::process::abort();
        }

        let closure_data = (*closure_ptr).payload.closure_val;
        if closure_data.is_null() {
            eprintln!("Runtime Error: closure data is null");
            std::process::abort();
        }

        &*closure_data
    };

    let func_ptr_val = closure_data.func_ptr as usize;
    let env_ptr_val = closure_data.env as usize;

    tokio::task::spawn_blocking(move || {
        let func: extern "C" fn(*mut c_void) -> i64 = unsafe { std::mem::transmute(func_ptr_val) };
        let env = env_ptr_val as *mut c_void;

        // Construct a temporary closure on stack to pass to wrapper
        let local_closure = Closure { func, env };
        let local_closure_ptr = &local_closure as *const _ as *mut c_void;

        with_gc_registered(|| unsafe {
            let _res = GC_call_with_stack_base(run_closure_wrapper, local_closure_ptr);
        });
    });
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `val` points to a valid `DlispValue` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_sleep(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if (*val).type_ != ValueType::Int {
            eprintln!("Type Error: sleep requires integer");
            std::process::abort();
        }
        let ms = (*val).payload.int_val as u64;
        std::thread::sleep(std::time::Duration::from_millis(ms));
        dlisp_make_int(0) // return nil
    }
}
