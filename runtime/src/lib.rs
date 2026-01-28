use std::ffi::c_void;
use tokio::runtime::Runtime;

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_main(user_main_ptr: extern "C" fn(*mut c_void) -> i64) {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        // Run the user's main function
        // Pass NULL as env
        let handle = tokio::task::spawn_blocking(move || {
            user_main_ptr(std::ptr::null_mut());
        });

        let _ = handle.await.unwrap();
    });
}

#[repr(C)]
pub struct Closure {
    pub func: extern "C" fn(*mut c_void) -> i64, // Assume 0-arity for spawn for now + env
    pub env: *mut c_void,
}

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_spawn(closure_ptr: *mut Closure) {
    // Safety: We assume closure_ptr is valid.
    // We cast to usize to pass across thread boundary safely.
    let ptr_val = closure_ptr as usize;

    tokio::task::spawn_blocking(move || {
        let closure_ptr = ptr_val as *mut Closure;
        let closure = unsafe { &*closure_ptr };
        let func = closure.func;
        let env = closure.env;

        // Call it with env
        func(env);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_sleep(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}
