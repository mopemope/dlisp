use std::ffi::{CStr, c_char, c_void};
use std::io::Write;
use tokio::runtime::Runtime;

pub mod value;
use value::{DlispValue, ListData, ValueType};

// Boehm GC bindings (Manual FFI)
#[link(name = "gc")]
unsafe extern "C" {
    pub fn GC_init();
    pub fn GC_malloc(size: usize) -> *mut c_void;
    pub fn GC_call_with_stack_base(
        func: extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void,
        arg: *mut c_void,
    ) -> *mut c_void;
    pub fn GC_allow_register_threads();
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

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_make_float(val: f64) -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_float(val);
        ptr
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_make_bool(val: bool) -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_bool(val);
        ptr
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dlisp_make_nil() -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_nil();
        ptr
    }
}

pub mod vectors;

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `list` points to a valid `DlispValue` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_car(list: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if list.is_null() {
            return dlisp_make_nil();
        }
        let val = *list;
        if val.type_ != ValueType::List {
            if val.type_ == ValueType::Nil {
                return dlisp_make_nil();
            }
            return dlisp_make_nil();
        }
        let list_data = *val.payload.list_val;
        list_data.car
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `list` points to a valid `DlispValue` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_cdr(list: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if list.is_null() {
            return dlisp_make_nil();
        }
        let val = *list;
        if val.type_ != ValueType::List {
            if val.type_ == ValueType::Nil {
                return dlisp_make_nil();
            }
            return dlisp_make_nil();
        }
        let list_data = *val.payload.list_val;
        list_data.cdr
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
        let _ = std::io::stdout().flush();
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
            ValueType::Float => {
                print!("{}", (*val).payload.float_val);
            }
            ValueType::Bool => {
                print!("{}", (*val).payload.bool_val);
            }
            ValueType::Nil => {
                print!("nil");
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
            ValueType::Vector => {
                print!("[");
                let vec_data = (*val).payload.vector_val;
                if !vec_data.is_null() {
                    let len = (*vec_data).len;
                    let data = (*vec_data).data;
                    for i in 0..len {
                        if i > 0 {
                            print!(" ");
                        }
                        let elem = *data.add(i);
                        dlisp_print_value(elem);
                    }
                }
                print!("]");
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
        match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => {
                dlisp_make_int((*a).payload.int_val + (*b).payload.int_val)
            }
            (ValueType::Float, ValueType::Float) => {
                dlisp_make_float((*a).payload.float_val + (*b).payload.float_val)
            }
            (ValueType::Int, ValueType::Float) => {
                dlisp_make_float((*a).payload.int_val as f64 + (*b).payload.float_val)
            }
            (ValueType::Float, ValueType::Int) => {
                dlisp_make_float((*a).payload.float_val + (*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: + requires numbers");
                std::process::abort();
            }
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_sub(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => {
                dlisp_make_int((*a).payload.int_val - (*b).payload.int_val)
            }
            (ValueType::Float, ValueType::Float) => {
                dlisp_make_float((*a).payload.float_val - (*b).payload.float_val)
            }
            (ValueType::Int, ValueType::Float) => {
                dlisp_make_float((*a).payload.int_val as f64 - (*b).payload.float_val)
            }
            (ValueType::Float, ValueType::Int) => {
                dlisp_make_float((*a).payload.float_val - (*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: - requires numbers");
                std::process::abort();
            }
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_mul(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => {
                dlisp_make_int((*a).payload.int_val * (*b).payload.int_val)
            }
            (ValueType::Float, ValueType::Float) => {
                dlisp_make_float((*a).payload.float_val * (*b).payload.float_val)
            }
            (ValueType::Int, ValueType::Float) => {
                dlisp_make_float((*a).payload.int_val as f64 * (*b).payload.float_val)
            }
            (ValueType::Float, ValueType::Int) => {
                dlisp_make_float((*a).payload.float_val * (*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: * requires numbers");
                std::process::abort();
            }
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_gt(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let result = match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => (*a).payload.int_val > (*b).payload.int_val,
            (ValueType::Float, ValueType::Float) => (*a).payload.float_val > (*b).payload.float_val,
            (ValueType::Int, ValueType::Float) => {
                ((*a).payload.int_val as f64) > (*b).payload.float_val
            }
            (ValueType::Float, ValueType::Int) => {
                (*a).payload.float_val > ((*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: > requires numbers");
                std::process::abort();
            }
        };

        if result {
            dlisp_make_bool(true)
        } else {
            dlisp_make_bool(false)
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_lt(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let result = match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => (*a).payload.int_val < (*b).payload.int_val,
            (ValueType::Float, ValueType::Float) => (*a).payload.float_val < (*b).payload.float_val,
            (ValueType::Int, ValueType::Float) => {
                ((*a).payload.int_val as f64) < (*b).payload.float_val
            }
            (ValueType::Float, ValueType::Int) => {
                (*a).payload.float_val < ((*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: < requires numbers");
                std::process::abort();
            }
        };

        if result {
            dlisp_make_bool(true)
        } else {
            dlisp_make_bool(false)
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_eq(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let result = match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => (*a).payload.int_val == (*b).payload.int_val,
            (ValueType::Float, ValueType::Float) => {
                ((*a).payload.float_val - (*b).payload.float_val).abs() < f64::EPSILON
            }
            (ValueType::Int, ValueType::Float) => {
                ((*a).payload.int_val as f64 - (*b).payload.float_val).abs() < f64::EPSILON
            }
            (ValueType::Float, ValueType::Int) => {
                ((*a).payload.float_val - (*b).payload.int_val as f64).abs() < f64::EPSILON
            }
            (ValueType::Symbol, ValueType::Symbol) => {
                let s1 = CStr::from_ptr((*a).payload.str_val);
                let s2 = CStr::from_ptr((*b).payload.str_val);
                s1 == s2
            }
            (ValueType::String, ValueType::String) => {
                let s1 = CStr::from_ptr((*a).payload.str_val);
                let s2 = CStr::from_ptr((*b).payload.str_val);
                s1 == s2
            }
            (ValueType::Nil, ValueType::Nil) => true,
            _ => false,
        };

        if result {
            dlisp_make_bool(true)
        } else {
            dlisp_make_bool(false)
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
        if (*val).type_ == ValueType::Nil {
            return 0;
        }
        if (*val).type_ == ValueType::Bool {
            return if (*val).payload.bool_val { 1 } else { 0 };
        }
        if (*val).type_ == ValueType::Int {
            // Historical/Compatibility: 0 is falsey?
            // Strict Lisp: only nil is false (and maybe false).
            // Let's keep 0 as falsey for now if we relied on it, but ideally we move to Bool/Nil.
            // For backwards compat with Phase 2 implementation where 0 was false.
            if (*val).payload.int_val == 0 {
                return 0;
            }
        }
        1
    }
}

// --- Main & Async ---

// Wrapper callbacks for GC_call_with_stack_base

extern "C" fn run_user_main_wrapper(_sb: *mut c_void, arg: *mut c_void) -> *mut c_void {
    unsafe {
        // arg is user_main_ptr cast to void*
        // transmute back to fn
        let user_main_ptr: extern "C" fn(*mut c_void) -> i64 = std::mem::transmute(arg);
        user_main_ptr(std::ptr::null_mut());
        std::ptr::null_mut()
    }
}

extern "C" fn run_closure_wrapper(_sb: *mut c_void, arg: *mut c_void) -> *mut c_void {
    unsafe {
        let closure_ptr = arg as *mut Closure;
        let closure = &*closure_ptr;
        (closure.func)(closure.env);
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
        let handle = tokio::task::spawn_blocking(move || unsafe {
            GC_call_with_stack_base(run_user_main_wrapper, user_main_ptr as *mut c_void);
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
    let closure = unsafe { &*closure_ptr };
    let func = closure.func;
    let env = closure.env;
    let func_ptr_val = func as usize;
    let env_ptr_val = env as usize;

    tokio::task::spawn_blocking(move || {
        let func: extern "C" fn(*mut c_void) -> i64 = unsafe { std::mem::transmute(func_ptr_val) };
        let env = env_ptr_val as *mut c_void;

        // Construct a temporary closure on stack to pass to wrapper
        let local_closure = Closure { func, env };
        let local_closure_ptr = &local_closure as *const _ as *mut c_void;

        unsafe {
            let _res = GC_call_with_stack_base(run_closure_wrapper, local_closure_ptr);
        }
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

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `val` points to a valid `DlispValue` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_read_file(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if (*val).type_ != ValueType::String {
            eprintln!("Type Error: read-file requires string");
            std::process::abort();
        }
        let c_str = CStr::from_ptr((*val).payload.str_val);
        let path = c_str.to_string_lossy().to_string();

        let result = tokio::runtime::Handle::current()
            .block_on(async move { tokio::fs::read_to_string(path).await });

        match result {
            Ok(content) => {
                let bytes = content.as_bytes();
                let len = bytes.len();
                let ptr = dlisp_gc_malloc(len + 1) as *mut u8;
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
                *ptr.add(len) = 0;
                dlisp_make_string(ptr as *mut i8)
            }
            Err(_) => {
                // Return nil on error for now
                dlisp_make_nil()
            }
        }
    }
}

mod verify_tests;
