use crate::ast::Value;
use dlisp_runtime::value::{DlispValue, ValueType};
use futures::future::LocalBoxFuture;

use crate::jit_runner::{runtime_to_value, value_to_runtime};

type BuiltinResult = Result<Value, String>;

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Integer(_) => "integer",
        Value::Float(_) => "float",
        Value::Bool(_) => "boolean",
        Value::Symbol(_) => "symbol",
        Value::Keyword(_) => "keyword",
        Value::String(_) => "string",
        Value::List(_) => "list",
        Value::Vector(_) => "vector",
        Value::Map(_) => "map",
        Value::Nil => "nil",
        Value::Channel(_) => "channel",
        Value::Atom(_) => "atom",
        _ => "value",
    }
}

/// Converts an interpreter `Value` into a runtime `DlispValue` pointer.
unsafe fn to_runtime(value: &Value) -> Result<*mut DlispValue, String> {
    unsafe { value_to_runtime(value) }.ok_or_else(|| {
        format!(
            "cannot pass a {} through the concurrency builtins",
            type_name(value)
        )
    })
}

unsafe fn from_runtime(ptr: *mut DlispValue) -> Result<Value, String> {
    unsafe { runtime_to_value(ptr) }.ok_or_else(|| "unsupported runtime value".to_string())
}

fn handle_ptr(addr: u64) -> *mut DlispValue {
    addr as *mut DlispValue
}

fn require_channel(arg: &Value, op: &str) -> Result<u64, String> {
    match arg {
        Value::Channel(addr) => Ok(*addr),
        other => Err(format!(
            "{op} requires a channel as argument 1, got {}",
            type_name(other)
        )),
    }
}

fn require_atom(arg: &Value, op: &str) -> Result<u64, String> {
    match arg {
        Value::Atom(addr) => Ok(*addr),
        other => Err(format!(
            "{op} requires an atom as argument 1, got {}",
            type_name(other)
        )),
    }
}

pub fn chan(args: &[Value]) -> LocalBoxFuture<'static, BuiltinResult> {
    let args = args.to_vec();
    Box::pin(async move {
        if !args.is_empty() {
            return Err("chan requires no arguments".to_string());
        }
        unsafe { from_runtime(dlisp_runtime::dlisp_chan_new()) }
    })
}

pub fn send(args: &[Value]) -> LocalBoxFuture<'static, BuiltinResult> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("send requires exactly 2 arguments (channel value)".to_string());
        }
        let addr = require_channel(&args[0], "send")?;
        let val = unsafe { to_runtime(&args[1])? };
        let delivered = unsafe { *dlisp_runtime::dlisp_chan_send(handle_ptr(addr), val) };
        if delivered.type_ != ValueType::Bool {
            return Err("send: malformed runtime result".to_string());
        }
        Ok(Value::Bool(unsafe { delivered.payload.bool_val }))
    })
}

pub fn try_recv(args: &[Value]) -> LocalBoxFuture<'static, BuiltinResult> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("try-recv requires exactly 1 argument (channel)".to_string());
        }
        let addr = require_channel(&args[0], "try-recv")?;
        let got = unsafe { dlisp_runtime::dlisp_chan_try_recv(handle_ptr(addr)) };
        unsafe { from_runtime(got) }
    })
}

pub fn recv(args: &[Value]) -> LocalBoxFuture<'static, BuiltinResult> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("recv requires exactly 1 argument (channel)".to_string());
        }
        let addr = require_channel(&args[0], "recv")?;
        // Every interpreter task runs on one thread, so recv cannot block
        // the executor on the condvar. `dlisp_chan_poll` observes value and
        // closed state under a single lock (no lost-wakeup race with OS
        // threads) and parks up to 1ms per call; the yield in between lets
        // cooperative tasks run. OS-thread producers wake the condvar
        // immediately.
        let mut closed: i64 = 0;
        loop {
            let got = unsafe { dlisp_runtime::dlisp_chan_poll(handle_ptr(addr), 1, &mut closed) };
            match unsafe { from_runtime(got) }? {
                Value::Nil if closed != 0 => return Ok(Value::Nil),
                Value::Nil => tokio::task::yield_now().await,
                value => return Ok(value),
            }
        }
    })
}

pub fn close(args: &[Value]) -> LocalBoxFuture<'static, BuiltinResult> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("close requires exactly 1 argument (channel)".to_string());
        }
        let addr = require_channel(&args[0], "close")?;
        unsafe { dlisp_runtime::dlisp_chan_close(handle_ptr(addr)) };
        Ok(Value::Nil)
    })
}

pub fn channel_p(args: &[Value]) -> LocalBoxFuture<'static, BuiltinResult> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("channel? requires exactly 1 argument".to_string());
        }
        Ok(Value::Bool(matches!(args[0], Value::Channel(_))))
    })
}

pub fn atom_new(args: &[Value]) -> LocalBoxFuture<'static, BuiltinResult> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("atom requires exactly 1 argument".to_string());
        }
        let val = unsafe { to_runtime(&args[0])? };
        unsafe { from_runtime(dlisp_runtime::dlisp_atom_new(val)) }
    })
}

pub fn deref(args: &[Value]) -> LocalBoxFuture<'static, BuiltinResult> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("deref requires exactly 1 argument (atom)".to_string());
        }
        let addr = require_atom(&args[0], "deref")?;
        let val = unsafe { dlisp_runtime::dlisp_atom_deref(handle_ptr(addr)) };
        unsafe { from_runtime(val) }
    })
}

pub fn reset(args: &[Value]) -> LocalBoxFuture<'static, BuiltinResult> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 2 {
            return Err("reset! requires exactly 2 arguments (atom value)".to_string());
        }
        let addr = require_atom(&args[0], "reset!")?;
        let val = unsafe { to_runtime(&args[1])? };
        let stored = unsafe { dlisp_runtime::dlisp_atom_reset(handle_ptr(addr), val) };
        unsafe { from_runtime(stored) }
    })
}

pub fn atom_p(args: &[Value]) -> LocalBoxFuture<'static, BuiltinResult> {
    let args = args.to_vec();
    Box::pin(async move {
        if args.len() != 1 {
            return Err("atom? requires exactly 1 argument".to_string());
        }
        Ok(Value::Bool(matches!(args[0], Value::Atom(_))))
    })
}
