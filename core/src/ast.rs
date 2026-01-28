use std::fmt;

#[derive(Debug, Clone, PartialEq)]
#[allow(unpredictable_function_pointer_comparisons)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Symbol(String),
    String(String),
    // For simplicty, native functions are handled by name or special variant.
    // We'll use a wrapper struct or just string for now to avoid complexity in this step if possible?
    // No, we need to call them.
    // Let's use a function type alias.
    NativeFunc(fn(&[Value]) -> Result<Value, String>),
    UserFunc {
        args: Vec<String>,
        body: Vec<Value>,
        jit_code: Option<usize>,
    },
    List(Vec<Value>),
    Nil,
}

// Function pointers implement PartialEq, but let's confirm.
// If not, we might need manual PartialEq.
// Rust fn pointers DO implement PartialEq.

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::String(s) => write!(f, "\"{}\"", s), // Simple escaping for now
            Value::List(l) => {
                write!(f, "(")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Value::Nil => write!(f, "nil"),
            Value::NativeFunc(_) => write!(f, "<native-func>"),
            Value::UserFunc { .. } => write!(f, "<user-func>"),
        }
    }
}
