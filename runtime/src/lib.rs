use std::ffi::{CStr, c_char, c_void};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use tokio::runtime::Runtime;

pub mod value;
use value::{DlispValue, ListData, MapData, ValuePayload, ValueType};

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
pub unsafe extern "C" fn dlisp_make_keyword(s: *mut c_char) -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_keyword(s);
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
            ValueType::Keyword => {
                let c_str = CStr::from_ptr((*val).payload.str_val);
                print!(":{}", c_str.to_string_lossy());
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
            ValueType::Map => {
                print!("{{");
                let map_data = (*val).payload.map_val;
                if !map_data.is_null() {
                    let cap = (*map_data).cap;
                    let elements = (*map_data).elements;
                    let mut first = true;
                    for i in 0..cap {
                        let opt_ptr = elements.add(i);
                        if let Some((elem_k, elem_v)) = *opt_ptr {
                            if !first {
                                print!(" ");
                            }
                            dlisp_print_value(elem_k);
                            print!(" ");
                            dlisp_print_value(elem_v);
                            first = false;
                        }
                    }
                }
                print!("}}");
            }
        }
    }
}

/// # Map Operations

/// # Safety
/// This function is unsafe because it uses raw pointers directly.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_make_map() -> *mut DlispValue {
    unsafe {
        let cap = 8;
        let map_val = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        let map_data = dlisp_gc_malloc(std::mem::size_of::<MapData>()) as *mut MapData;

        let elements_size = cap * std::mem::size_of::<Option<(*mut DlispValue, *mut DlispValue)>>();
        let elements =
            dlisp_gc_malloc(elements_size) as *mut Option<(*mut DlispValue, *mut DlispValue)>;

        // Initialize to None
        for i in 0..cap {
            std::ptr::write(elements.add(i), None);
        }

        (*map_data).len = 0;
        (*map_data).cap = cap;
        (*map_data).elements = elements;

        *map_val = DlispValue {
            type_: ValueType::Map,
            payload: ValuePayload { map_val: map_data },
        };
        map_val
    }
}

unsafe fn dlisp_hash_value(val: *mut DlispValue) -> u64 {
    let mut hasher = DefaultHasher::new();
    if val.is_null() {
        return 0;
    }
    unsafe {
        match (*val).type_ {
            ValueType::Int => {
                (*val).payload.int_val.hash(&mut hasher);
            }
            ValueType::Bool => {
                (*val).payload.bool_val.hash(&mut hasher);
            }
            ValueType::String | ValueType::Symbol | ValueType::Keyword => {
                let c_str = CStr::from_ptr((*val).payload.str_val);
                c_str.to_bytes().hash(&mut hasher);
            }
            _ => {
                // For now, simpler hash based on pointer address for other types
                (val as usize).hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

unsafe fn dlisp_map_insert(map_data: *mut MapData, key: *mut DlispValue, val: *mut DlispValue) {
    unsafe {
        let cap = (*map_data).cap;
        let hash = dlisp_hash_value(key) as usize;
        let mut idx = hash % cap;

        // Linear probing
        loop {
            let opt_ptr = (*map_data).elements.add(idx);
            if (*opt_ptr).is_none() {
                std::ptr::write(opt_ptr, Some((key, val)));
                (*map_data).len += 1;
                return;
            } else if let Some((existing_key, _)) = *opt_ptr {
                let eq_val = dlisp_eq(existing_key, key);
                if !eq_val.is_null()
                    && (*eq_val).type_ == ValueType::Bool
                    && (*eq_val).payload.bool_val
                {
                    // Update existing
                    std::ptr::write(opt_ptr, Some((key, val)));
                    return;
                }
            }
            idx = (idx + 1) % cap;
        }
    }
}

/// # Safety
/// This function is unsafe because it uses raw pointers directly.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_map_assoc(
    map_val: *mut DlispValue,
    key: *mut DlispValue,
    val: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if map_val.is_null() || (*map_val).type_ != ValueType::Map {
            eprintln!("Type Error: assoc requires a map as first argument");
            std::process::abort();
        }

        let old_map_data = (*map_val).payload.map_val;
        let old_cap = (*old_map_data).cap;
        let old_len = (*old_map_data).len;

        // Check if this key already exists to avoid over-counting
        let existing = dlisp_map_get(map_val, key);
        let is_update = !existing.is_null() && (*existing).type_ != ValueType::Nil;
        let new_len = if is_update { old_len } else { old_len + 1 };

        let new_cap = if new_len * 2 > old_cap {
            old_cap * 2
        } else {
            old_cap
        };

        let result_map_val = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        let result_map_data = dlisp_gc_malloc(std::mem::size_of::<MapData>()) as *mut MapData;
        let elements_size =
            new_cap * std::mem::size_of::<Option<(*mut DlispValue, *mut DlispValue)>>();
        let elements =
            dlisp_gc_malloc(elements_size) as *mut Option<(*mut DlispValue, *mut DlispValue)>;

        for i in 0..new_cap {
            std::ptr::write(elements.add(i), None);
        }

        (*result_map_data).len = 0;
        (*result_map_data).cap = new_cap;
        (*result_map_data).elements = elements;

        // Copy existing elements
        for i in 0..old_cap {
            let opt_ptr = (*old_map_data).elements.add(i);
            if let Some((k, v)) = *opt_ptr {
                dlisp_map_insert(result_map_data, k, v);
            }
        }

        // Insert new element
        dlisp_map_insert(result_map_data, key, val);

        *result_map_val = DlispValue {
            type_: ValueType::Map,
            payload: ValuePayload {
                map_val: result_map_data,
            },
        };
        result_map_val
    }
}

/// # Safety
/// This function is unsafe because it uses raw pointers directly.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_map_get(
    map_val: *mut DlispValue,
    key: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if map_val.is_null() || (*map_val).type_ != ValueType::Map {
            // Like core/src/builtins/map.rs `get`, return nil if not map
            return dlisp_make_nil();
        }
        let map_data = (*map_val).payload.map_val;
        if map_data.is_null() || (*map_data).len == 0 {
            return dlisp_make_nil();
        }

        let cap = (*map_data).cap;
        let hash = dlisp_hash_value(key) as usize;
        let mut idx = hash % cap;
        let start_idx = idx;

        loop {
            let opt_ptr = (*map_data).elements.add(idx);
            if (*opt_ptr).is_none() {
                return dlisp_make_nil();
            } else if let Some((existing_key, existing_val)) = *opt_ptr {
                let eq_val = dlisp_eq(existing_key, key);
                if !eq_val.is_null()
                    && (*eq_val).type_ == ValueType::Bool
                    && (*eq_val).payload.bool_val
                {
                    return existing_val;
                }
            }
            idx = (idx + 1) % cap;
            if idx == start_idx {
                return dlisp_make_nil();
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
            (ValueType::Map, ValueType::Map) => {
                let m1 = (*a).payload.map_val;
                let m2 = (*b).payload.map_val;
                if m1 == m2 {
                    true
                } else if m1.is_null() || m2.is_null() || (*m1).len != (*m2).len {
                    false
                } else {
                    let len1 = (*m1).len;
                    let mut matches = 0;
                    for i in 0..(*m1).cap {
                        if let Some((k1, v1)) = *(*m1).elements.add(i) {
                            let v2 = dlisp_map_get(b, k1);
                            if v2.is_null() || (*v2).type_ == ValueType::Nil {
                                break;
                            }
                            let eq_val = dlisp_eq(v1, v2);
                            if eq_val.is_null()
                                || (*eq_val).type_ != ValueType::Bool
                                || !(*eq_val).payload.bool_val
                            {
                                break;
                            }
                            matches += 1;
                        }
                    }
                    matches == len1
                }
            }
            (ValueType::Symbol, ValueType::Symbol) => {
                let s1 = CStr::from_ptr((*a).payload.str_val);
                let s2 = CStr::from_ptr((*b).payload.str_val);
                s1 == s2
            }
            (ValueType::Keyword, ValueType::Keyword) => {
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
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_gte(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let result = match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => (*a).payload.int_val >= (*b).payload.int_val,
            (ValueType::Float, ValueType::Float) => {
                (*a).payload.float_val >= (*b).payload.float_val
            }
            (ValueType::Int, ValueType::Float) => {
                ((*a).payload.int_val as f64) >= (*b).payload.float_val
            }
            (ValueType::Float, ValueType::Int) => {
                (*a).payload.float_val >= ((*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: >= requires numbers");
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
pub unsafe extern "C" fn dlisp_lte(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let result = match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => (*a).payload.int_val <= (*b).payload.int_val,
            (ValueType::Float, ValueType::Float) => {
                (*a).payload.float_val <= (*b).payload.float_val
            }
            (ValueType::Int, ValueType::Float) => {
                ((*a).payload.int_val as f64) <= (*b).payload.float_val
            }
            (ValueType::Float, ValueType::Int) => {
                (*a).payload.float_val <= ((*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: <= requires numbers");
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
pub unsafe extern "C" fn dlisp_neq(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let eq = dlisp_eq(a, b);
        let is_eq = dlisp_is_truthy(eq) != 0;
        dlisp_make_bool(!is_eq)
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_div(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => {
                if (*b).payload.int_val == 0 {
                    eprintln!("Runtime Error: Division by zero");
                    std::process::abort();
                }
                dlisp_make_int((*a).payload.int_val / (*b).payload.int_val)
            }
            (ValueType::Float, ValueType::Float) => {
                dlisp_make_float((*a).payload.float_val / (*b).payload.float_val)
            }
            (ValueType::Int, ValueType::Float) => {
                dlisp_make_float((*a).payload.int_val as f64 / (*b).payload.float_val)
            }
            (ValueType::Float, ValueType::Int) => {
                dlisp_make_float((*a).payload.float_val / (*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: / requires numbers");
                std::process::abort();
            }
        }
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that `a` and `b` point to valid `DlispValue` structs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_mod(a: *mut DlispValue, b: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        match ((*a).type_, (*b).type_) {
            (ValueType::Int, ValueType::Int) => {
                if (*b).payload.int_val == 0 {
                    eprintln!("Runtime Error: Modulo by zero");
                    std::process::abort();
                }
                dlisp_make_int((*a).payload.int_val % (*b).payload.int_val)
            }
            (ValueType::Float, ValueType::Float) => {
                dlisp_make_float((*a).payload.float_val % (*b).payload.float_val)
            }
            (ValueType::Int, ValueType::Float) => {
                dlisp_make_float((*a).payload.int_val as f64 % (*b).payload.float_val)
            }
            (ValueType::Float, ValueType::Int) => {
                dlisp_make_float((*a).payload.float_val % (*b).payload.int_val as f64)
            }
            _ => {
                eprintln!("Type Error: % requires numbers");
                std::process::abort();
            }
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

// --- String Operations ---

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_str(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if val.is_null() {
            let c_str = std::ffi::CString::new("").unwrap();
            return dlisp_make_string(c_str.into_raw());
        }

        let s = match (*val).type_ {
            ValueType::Int => (*val).payload.int_val.to_string(),
            ValueType::Float => (*val).payload.float_val.to_string(),
            ValueType::Bool => (*val).payload.bool_val.to_string(),
            ValueType::Nil => "nil".to_string(),
            ValueType::String => {
                let c_str = CStr::from_ptr((*val).payload.str_val);
                c_str.to_string_lossy().to_string()
            }
            ValueType::Symbol => {
                let c_str = CStr::from_ptr((*val).payload.str_val);
                c_str.to_string_lossy().to_string()
            }
            ValueType::Keyword => {
                let c_str = CStr::from_ptr((*val).payload.str_val);
                format!(":{}", c_str.to_string_lossy())
            }
            ValueType::List => {
                // Simplified representation for now
                "list".to_string()
            }
            ValueType::Vector => "vector".to_string(),
            ValueType::Map => "map".to_string(),
            _ => "unknown".to_string(),
        };
        let c_str = std::ffi::CString::new(s).unwrap();
        dlisp_make_string(c_str.into_raw())
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_length(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        if (*val).type_ != ValueType::String {
            eprintln!("Type Error: string-length requires string");
            std::process::abort();
        }
        let c_str = CStr::from_ptr((*val).payload.str_val);
        let s = c_str.to_string_lossy();
        let len = s.chars().count() as i64;
        dlisp_make_int(len)
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_substring(
    s_val: *mut DlispValue,
    start_val: *mut DlispValue,
    end_val: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if (*s_val).type_ != ValueType::String
            || (*start_val).type_ != ValueType::Int
            || (*end_val).type_ != ValueType::Int
        {
            eprintln!("Type Error: substring requires string, int, int");
            std::process::abort();
        }
        let c_str = CStr::from_ptr((*s_val).payload.str_val);
        let s = c_str.to_string_lossy();
        let start = (*start_val).payload.int_val as usize;
        let end = (*end_val).payload.int_val as usize;

        let chars: Vec<char> = s.chars().collect();
        if start > chars.len() || end > chars.len() || start > end {
            let empty = std::ffi::CString::new("").unwrap();
            return dlisp_make_string(empty.into_raw());
        }

        let sub: String = chars[start..end].iter().collect();
        let c_sub = std::ffi::CString::new(sub).unwrap();
        dlisp_make_string(c_sub.into_raw())
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_append(
    a: *mut DlispValue,
    b: *mut DlispValue,
) -> *mut DlispValue {
    unsafe {
        if (*a).type_ != ValueType::String || (*b).type_ != ValueType::String {
            eprintln!("Type Error: string-append requires strings");
            std::process::abort();
        }
        let s1 = CStr::from_ptr((*a).payload.str_val).to_string_lossy();
        let s2 = CStr::from_ptr((*b).payload.str_val).to_string_lossy();
        let combined = format!("{}{}", s1, s2);
        let c_combined = std::ffi::CString::new(combined).unwrap();
        dlisp_make_string(c_combined.into_raw())
    }
}

// --- Type Predicates ---

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_nil_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { dlisp_make_bool(!val.is_null() && (*val).type_ == ValueType::Nil) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_list_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        dlisp_make_bool(
            !val.is_null() && ((*val).type_ == ValueType::List || (*val).type_ == ValueType::Nil),
        )
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_number_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        dlisp_make_bool(
            !val.is_null() && ((*val).type_ == ValueType::Int || (*val).type_ == ValueType::Float),
        )
    }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_string_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { dlisp_make_bool(!val.is_null() && (*val).type_ == ValueType::String) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_symbol_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { dlisp_make_bool(!val.is_null() && (*val).type_ == ValueType::Symbol) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_keyword_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { dlisp_make_bool(!val.is_null() && (*val).type_ == ValueType::Keyword) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_vector_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { dlisp_make_bool(!val.is_null() && (*val).type_ == ValueType::Vector) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_map_p(val: *mut DlispValue) -> *mut DlispValue {
    unsafe { dlisp_make_bool(!val.is_null() && (*val).type_ == ValueType::Map) }
}

/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_type_of(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let name = if val.is_null() {
            "nil"
        } else {
            match (*val).type_ {
                ValueType::Int => "integer",
                ValueType::Float => "float",
                ValueType::Bool => "boolean",
                ValueType::Nil => "nil",
                ValueType::String => "string",
                ValueType::Symbol => "symbol",
                ValueType::Keyword => "keyword",
                ValueType::List => "cons",
                ValueType::Vector => "vector",
                ValueType::Map => "map",
                ValueType::Closure => "closure",
                ValueType::NativePtr => "native_ptr",
            }
        };
        let c_str = std::ffi::CString::new(name).unwrap();
        dlisp_make_string(c_str.into_raw())
    }
}

pub mod io;
pub mod os;
pub mod sys;

mod verify_tests;
