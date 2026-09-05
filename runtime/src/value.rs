#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Int,
    Float,
    Bool,
    Nil,
    String,
    List,
    Vector,
    Map,
    Symbol,
    Keyword,
    Closure,
    NativePtr,
    Channel,
    Atom,
    Error,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ValuePayload {
    pub int_val: i64,
    pub float_val: f64,
    pub bool_val: bool,
    pub str_val: *mut i8, // C-string
    pub list_val: *mut ListData,
    pub vector_val: *mut VectorData,
    pub map_val: *mut MapData,
    pub closure_val: *mut ClosureData,
    pub ptr_val: *mut std::ffi::c_void,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ListData {
    pub car: *mut DlispValue,
    pub cdr: *mut DlispValue,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VectorData {
    pub len: usize,
    pub cap: usize,
    pub data: *mut *mut DlispValue,
}

#[repr(C)]
pub struct ClosureData {
    pub env: *mut DlispValue,
    pub func_ptr: *const std::ffi::c_void,
}

#[repr(C)]
pub struct MapData {
    pub elements: *mut Option<(*mut DlispValue, *mut DlispValue)>,
    pub len: usize,
    pub cap: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DlispValue {
    pub type_: ValueType,
    pub payload: ValuePayload,
}

impl DlispValue {
    pub fn new_int(val: i64) -> Self {
        Self {
            type_: ValueType::Int,
            payload: ValuePayload { int_val: val },
        }
    }

    pub fn new_string(s: *mut i8) -> Self {
        Self {
            type_: ValueType::String,
            payload: ValuePayload { str_val: s },
        }
    }

    pub fn new_symbol(s: *mut i8) -> Self {
        Self {
            type_: ValueType::Symbol,
            payload: ValuePayload { str_val: s },
        }
    }

    pub fn new_keyword(s: *mut i8) -> Self {
        Self {
            type_: ValueType::Keyword,
            payload: ValuePayload { str_val: s },
        }
    }

    pub fn new_float(val: f64) -> Self {
        Self {
            type_: ValueType::Float,
            payload: ValuePayload { float_val: val },
        }
    }

    pub fn new_bool(val: bool) -> Self {
        Self {
            type_: ValueType::Bool,
            payload: ValuePayload { bool_val: val },
        }
    }

    pub fn new_nil() -> Self {
        Self {
            type_: ValueType::Nil,
            payload: ValuePayload { int_val: 0 }, // Payload doesn't matter for Nil
        }
    }

    /// Wraps a value inside an error marker (`Value::Error` on the interpreter side).
    pub fn new_error(inner: *mut DlispValue) -> Self {
        Self {
            type_: ValueType::Error,
            payload: ValuePayload {
                ptr_val: inner as *mut std::ffi::c_void,
            },
        }
    }

    pub fn new_vector(vec: *mut VectorData) -> Self {
        Self {
            type_: ValueType::Vector,
            payload: ValuePayload { vector_val: vec },
        }
    }

    /// Wraps an opaque GC-allocated state pointer (channel, atom, ...).
    pub fn new_opaque(type_: ValueType, state: *mut std::ffi::c_void) -> Self {
        Self {
            type_,
            payload: ValuePayload { ptr_val: state },
        }
    }

    pub fn opaque_ptr(&self) -> *mut std::ffi::c_void {
        unsafe { self.payload.ptr_val }
    }
}
