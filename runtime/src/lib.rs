use std::ffi::c_void;
use tokio::runtime::Runtime;

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_main(user_main_ptr: extern "C" fn()) {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        // Run the user's main function
        // Note: user_main in our current compilation model is blocking initially,
        // but it can call dlisp_spawn which will use this runtime context.
        // Wait, if user_main is blocking, it might block the runtime if we are not careful.
        // But for AOT, the user's main is just a function.
        // We probably want to run it on the runtime.

        let handle = tokio::task::spawn_blocking(move || {
            user_main_ptr();
        });

        handle.await.unwrap();
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_spawn(func_ptr: extern "C" fn()) {
    // This assumes we are inside a Tokio context (which dlisp_main ensures)
    tokio::task::spawn_blocking(move || {
        func_ptr();
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_sleep(ms: u64) {
    // Verify we are in runtime?
    // Since compilation generates synchronous calls, "sleep" needs to be blocking
    // OR we need to rethink if the compiled code is async.
    // Given the current codegen is simple function calls, everything is effectively synchronous
    // unless spawned.
    // If we are in spawn_blocking (which dlisp_spawn uses), we can block thread safely.
    std::thread::sleep(std::time::Duration::from_millis(ms));
}
