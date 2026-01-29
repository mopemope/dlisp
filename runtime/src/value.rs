use std::ffi::c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Int,
    Float,
    Bool,
    Nil,
    String,
    List,
    Symbol,
    Closure,
    NativePtr,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ValuePayload {
    pub int_val: i64,
    pub float_val: f64,
    pub bool_val: bool,
    pub str_val: *mut i8, // C-string
    pub list_val: *mut ListData,
    pub ptr_val: *mut c_void,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ListData {
    pub car: *mut DlispValue,
    pub cdr: *mut DlispValue,
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
}
