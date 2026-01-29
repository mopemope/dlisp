use std::ffi::{CStr, c_char, c_void};
use tokio::runtime::Runtime;

pub mod value;
use value::{DlispValue, ListData, ValueType};

// Boehm GC bindings (Manual FFI)
#[link(name = "gc")]
unsafe extern "C" {
    pub fn GC_init();
    pub fn GC_malloc(size: usize) -> *mut c_void;
}

static GC_INIT_ONCE: std::sync::Once = std::sync::Once::new();

/// Initialize the Boehm garbage collector.
/// Must be called once at program start before any allocations.
#[unsafe(no_mangle)]
pub extern "C" fn dlisp_gc_init() {
    GC_INIT_ONCE.call_once(|| unsafe {
        GC_init();
    });
}

/// Allocate memory using Boehm GC.
/// The allocated memory will be automatically garbage collected when unreachable.
#[unsafe(no_mangle)]
pub extern "C" fn dlisp_gc_malloc(size: usize) -> *mut c_void {
    unsafe { GC_malloc(size) }
}

// --- Value Constructors ---

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_make_int(val: i64) -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_int(val);
        ptr
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `s` points to a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_make_string(s: *mut c_char) -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_string(s);
        ptr
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `s` points to a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_make_symbol(s: *mut c_char) -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_symbol(s);
        ptr
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `car` and `cdr` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_make_cons(
    car: *mut DlispValue,
    cdr: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        let list_data_ptr = dlisp_gc_malloc(std::mem::size_of::<ListData>()) as *mut ListData;
        *list_data_ptr = ListData { car, cdr };

        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        (*ptr).type_ = ValueType::List;
        (*ptr).payload.list_val = list_data_ptr;
        ptr
    }
}

// --- Basic Operations ---

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `val` points to a valid `DlispValue` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_print(val: *mut DlispValue) {
    unsafe {
        dlisp_print_value(val);
        println!(); // Newline for the main print
    }
}

unsafe fn dlisp_print_value(val: *mut DlispValue) {
    if val.is_null() {
        print!("nil");
        return;
    }

    // Safety: we checked for null above.
    // However, since this is a recursive function dealing with raw pointers,
    // we must be careful. We rely on the GC and correct construction.
    // Safety: we checked for null above.
    // However, since this is a recursive function dealing with raw pointers,
    // we must be careful. We rely on the GC and correct construction.
    unsafe {
        match (*val).type_ {
            ValueType::Int => {
                print!("{}", (*val).payload.int_val);
            }
            ValueType::String => {
                let c_str = CStr::from_ptr((*val).payload.str_val);
                print!("{}", c_str.to_string_lossy());
            }
            ValueType::List => {
                print!("(");
                let mut curr = val;
                let mut first = true;
                loop {
                    if curr.is_null() {
                        break;
                    }

                    if (*curr).type_ != ValueType::List {
                        // Dotted pair
                        print!(" . ");
                        dlisp_print_value(curr);
                        break;
                    }

                    let list_data = (*curr).payload.list_val;
                    if list_data.is_null() {
                        break;
                    }

                    if !first {
                        print!(" ");
                    }
                    dlisp_print_value((*list_data).car);

                    curr = (*list_data).cdr;
                    first = false;
                }
                print!(")");
            }
            ValueType::Symbol => {
                let c_str = CStr::from_ptr((*val).payload.str_val);
                print!("{}", c_str.to_string_lossy());
            }
            ValueType::Closure => {
                print!("<closure>");
            }
            ValueType::NativePtr => {
                print!("<native_ptr>");
            }
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_add(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if (*a).type_ != ValueType::Int || (*b).type_ != ValueType::Int {
            eprintln!("Type Error: + requires integers");
            std::process::abort();
        }
        dlisp_make_int((*a).payload.int_val + (*b).payload.int_val)
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_sub(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if (*a).type_ != ValueType::Int || (*b).type_ != ValueType::Int {
            eprintln!("Type Error: - requires integers");
            std::process::abort();
        }
        dlisp_make_int((*a).payload.int_val - (*b).payload.int_val)
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_mul(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if (*a).type_ != ValueType::Int || (*b).type_ != ValueType::Int {
            eprintln!("Type Error: * requires integers");
            std::process::abort();
        }
        dlisp_make_int((*a).payload.int_val * (*b).payload.int_val)
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_gt(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if (*a).type_ != ValueType::Int || (*b).type_ != ValueType::Int {
            eprintln!("Type Error: > requires integers");
            std::process::abort();
        }
        if (*a).payload.int_val > (*b).payload.int_val {
            dlisp_make_int(1)
        } else {
            dlisp_make_int(0) // nil-like
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `val` points to a valid `DlispValue` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_is_truthy(val: *mut DlispValue) -> i32 {
    // Safety: val is checked for null. Dereferencing raw pointer.
    if val.is_null() {
        return 0;
    }
    unsafe {
        if (*val).type_ == ValueType::Int && (*val).payload.int_val == 0 {
            return 0;
        }
    }
    1
}

// --- Main & Async ---

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
            user_main_ptr(std::ptr::null_mut());
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
/// The caller must ensure that `closure_ptr` points to a valid `Closure` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_spawn(closure_ptr: *mut Closure) {
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

mod verify_tests;
