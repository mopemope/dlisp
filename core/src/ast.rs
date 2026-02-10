use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Debug, Clone)]
#[allow(clippy::mutable_key_type)]
#[allow(unpredictable_function_pointer_comparisons)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Symbol(String),
    String(String),
    NativeFunc(fn(&[Value]) -> futures::future::LocalBoxFuture<'static, Result<Value, String>>),
    UserFunc {
        args: Vec<String>,
        body: Vec<Value>,
        jit_code: Option<usize>,
        env: Option<Rc<RefCell<crate::environment::Environment>>>,
    },
    Macro {
        args: Vec<String>,
        body: Vec<Value>,
    },
    List(Vec<Value>),
    Vector(Vec<Value>),
    Map(HashMap<Value, Value>),
    Nil,
}

impl Value {
    fn discriminant(&self) -> u8 {
        match self {
            Value::Nil => 0,
            Value::Bool(_) => 1,
            Value::Integer(_) => 2,
            Value::Float(_) => 3,
            Value::Symbol(_) => 4,
            Value::String(_) => 5,
            Value::List(_) => 6,
            Value::Vector(_) => 7,
            Value::Map(_) => 8,
            Value::NativeFunc(_) => 9,
            Value::UserFunc { .. } => 10,
            Value::Macro { .. } => 11,
        }
    }

    /// Canonical truthiness check for DLisp values.
    /// Nil, Bool(false), and Integer(0) are falsy; everything else is truthy.
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false) | Value::Integer(0))
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a.to_bits() == b.to_bits(),
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Vector(a), Value::Vector(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            #[allow(clippy::fn_to_numeric_cast)]
            (Value::NativeFunc(a), Value::NativeFunc(b)) => (*a as usize) == (*b as usize),
            (
                Value::UserFunc {
                    args: a_args,
                    body: a_body,
                    jit_code: a_jit,
                    env: a_env,
                },
                Value::UserFunc {
                    args: b_args,
                    body: b_body,
                    jit_code: b_jit,
                    env: b_env,
                },
            ) => {
                a_args == b_args
                    && a_body == b_body
                    && a_jit == b_jit
                    && match (a_env, b_env) {
                        (Some(a), Some(b)) => Rc::ptr_eq(a, b), // Pointer equality for env
                        (None, None) => true,
                        _ => false,
                    }
            }
            (
                Value::Macro {
                    args: a_args,
                    body: a_body,
                },
                Value::Macro {
                    args: b_args,
                    body: b_body,
                },
            ) => a_args == b_args && a_body == b_body,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.discriminant().hash(state);
        match self {
            Value::Nil => {}
            Value::Bool(b) => b.hash(state),
            Value::Integer(i) => i.hash(state),
            Value::Float(f) => f.to_bits().hash(state),
            Value::Symbol(s) => s.hash(state),
            Value::String(s) => s.hash(state),
            Value::List(l) => l.hash(state),
            Value::Vector(v) => v.hash(state),
            Value::Map(m) => {
                // HashMap is not Hash, need custom strategy.
                // Sort entries by key (Value implements Ord) and hash.
                let mut entries: Vec<_> = m.iter().collect();
                entries.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
                for (k, v) in entries {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Value::NativeFunc(f) => (*f as usize).hash(state),
            Value::UserFunc {
                args,
                body,
                jit_code,
                env,
            } => {
                args.hash(state);
                body.hash(state);
                jit_code.hash(state);
                if let Some(env) = env {
                    (Rc::as_ptr(env) as usize).hash(state);
                } else {
                    0usize.hash(state);
                }
            }
            Value::Macro { args, body } => {
                args.hash(state);
                body.hash(state);
            }
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.discriminant() != other.discriminant() {
            return self.discriminant().cmp(&other.discriminant());
        }
        match (self, other) {
            (Value::Nil, Value::Nil) => Ordering::Equal,
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
            // Handle float NaN ordering
            (Value::Float(a), Value::Float(b)) => a.total_cmp(b),
            (Value::Symbol(a), Value::Symbol(b)) => a.cmp(b),
            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::List(a), Value::List(b)) => a.cmp(b),
            (Value::Vector(a), Value::Vector(b)) => a.cmp(b),
            (Value::Map(a), Value::Map(b)) => {
                // To compare maps, we can sort them and compare entries
                let mut entries_a: Vec<_> = a.iter().collect();
                let mut entries_b: Vec<_> = b.iter().collect();
                entries_a.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
                entries_b.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
                // Lexicographical comparison of sorted entries
                for ((k1, v1), (k2, v2)) in entries_a.iter().zip(entries_b.iter()) {
                    match k1.cmp(k2) {
                        Ordering::Equal => match v1.cmp(v2) {
                            Ordering::Equal => continue,
                            ord => return ord,
                        },
                        ord => return ord,
                    }
                }
                entries_a.len().cmp(&entries_b.len())
            }
            (Value::NativeFunc(a), Value::NativeFunc(b)) => (*a as usize).cmp(&(*b as usize)),
            (
                Value::UserFunc {
                    args: a_a,
                    body: a_b,
                    ..
                },
                Value::UserFunc {
                    args: b_a,
                    body: b_b,
                    ..
                },
            ) => a_a.cmp(b_a).then_with(|| a_b.cmp(b_b)),
            (
                Value::Macro {
                    args: a_a,
                    body: a_b,
                },
                Value::Macro {
                    args: b_a,
                    body: b_b,
                },
            ) => a_a.cmp(b_a).then_with(|| a_b.cmp(b_b)),
            _ => Ordering::Equal, // Should be covered by discriminant check
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::String(s) => write!(f, "{}", s),
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
            Value::Vector(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{} {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Nil => write!(f, "nil"),
            Value::NativeFunc(_) => write!(f, "<native-func>"),
            Value::UserFunc { .. } => write!(f, "<user-func>"),
            Value::Macro { .. } => write!(f, "<macro>"),
        }
    }
}
