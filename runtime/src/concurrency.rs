//! Channels and atoms for concurrent dlisp programs.
//!
//! All state lives in GC-allocated blocks so the queued/referenced
//! `DlispValue` pointers are traced by the conservative collector on every
//! path (interpreter tasks and OS-thread spawned tasks alike).
//!
//! Handles handed to the interpreter are raw addresses, and interpreter
//! environments are invisible to Boehm. Every channel/atom wrapper is
//! therefore also pinned in a process-lifetime registry (itself GC
//! allocated) so a collection triggered elsewhere can never reclaim an
//! object the interpreter still references. Sync primitives are created in
//! bounded numbers by real programs; pinning them is deliberate.
//!
//! Semantics are identical between interpreted and compiled code:
//! - `(send ch v)` returns `true` if delivered, `false` when closed.
//! - `(recv ch)` blocks until a value is available; returns `nil` once the
//!   channel is closed and drained.
//! - `(try-recv ch)` never blocks: the next value or `nil`.
//! - `(close ch)` is idempotent.

use std::sync::{Condvar, Mutex, OnceLock};

use crate::constructors::{dlisp_make_bool, dlisp_make_nil};
use crate::gc::dlisp_gc_malloc;
use crate::value::{DlispValue, ValueType};

/// Process-lifetime registry of channel/atom wrappers.
///
/// The item array is GC allocated, so every stored pointer is traced for as
/// long as the registry array itself is reachable from the static root.
struct HandleRegistry {
    items: *mut *mut DlispValue,
    len: usize,
    cap: usize,
}

unsafe impl Send for HandleRegistry {}

fn registry() -> &'static Mutex<HandleRegistry> {
    static REGISTRY: OnceLock<Mutex<HandleRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        Mutex::new(HandleRegistry {
            items: std::ptr::null_mut(),
            len: 0,
            cap: 0,
        })
    })
}

/// Pins a freshly created channel/atom wrapper so the collector keeps it
/// alive even when only non-scanned memory (interpreter environments, tokio
/// task state) holds its address.
/// Pins a freshly created channel/atom wrapper so the collector keeps it
/// alive even when only non-scanned memory (interpreter environments, tokio
/// task state) holds its address.
fn pin_handle(wrapper: *mut DlispValue) {
    debug_assert!(!wrapper.is_null());
    let mut reg = registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    unsafe {
        if reg.len == reg.cap {
            let new_cap = if reg.cap == 0 { 16 } else { reg.cap * 2 };
            let new_items = dlisp_gc_malloc(new_cap * std::mem::size_of::<*mut DlispValue>())
                as *mut *mut DlispValue;
            if !reg.items.is_null() {
                std::ptr::copy_nonoverlapping(reg.items, new_items, reg.len);
            }
            reg.items = new_items;
            reg.cap = new_cap;
        }
        *reg.items.add(reg.len) = wrapper;
        reg.len += 1;
    }
}

/// Linked queue node, GC-allocated so `value` stays traced while parked.
#[repr(C)]
struct ChanNode {
    value: *mut DlispValue,
    next: *mut ChanNode,
}

struct ChanState {
    head: *mut ChanNode,
    tail: *mut ChanNode,
    closed: bool,
}

/// GC-allocated channel state. The mutex/condvar are only ever touched by
/// Rust FFI code; compiled programs just pass the opaque wrapper around.
#[repr(C)]
pub struct ChanData {
    state: Mutex<ChanState>,
    not_empty: Condvar,
}

#[repr(C)]
pub struct AtomData {
    value: Mutex<*mut DlispValue>,
}

fn chan_data(chan: *mut DlispValue) -> Option<&'static ChanData> {
    if chan.is_null() {
        return None;
    }
    let val = unsafe { &*chan };
    if val.type_ != ValueType::Channel {
        return None;
    }
    let data = val.opaque_ptr() as *const ChanData;
    if data.is_null() {
        return None;
    }
    Some(unsafe { &*data })
}

fn atom_data(atom: *mut DlispValue) -> Option<&'static AtomData> {
    if atom.is_null() {
        return None;
    }
    let val = unsafe { &*atom };
    if val.type_ != ValueType::Atom {
        return None;
    }
    let data = val.opaque_ptr() as *const AtomData;
    if data.is_null() {
        return None;
    }
    Some(unsafe { &*data })
}

fn wrap(type_: ValueType, state: *mut std::ffi::c_void) -> *mut DlispValue {
    unsafe {
        let ptr = dlisp_gc_malloc(std::mem::size_of::<DlispValue>()) as *mut DlispValue;
        *ptr = DlispValue::new_opaque(type_, state);
        ptr
    }
}

/// # Safety
/// Allocates a fresh unbounded channel. Never fails.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_chan_new() -> *mut DlispValue {
    unsafe {
        let data = dlisp_gc_malloc(std::mem::size_of::<ChanData>()) as *mut ChanData;
        *data = ChanData {
            state: Mutex::new(ChanState {
                head: std::ptr::null_mut(),
                tail: std::ptr::null_mut(),
                closed: false,
            }),
            not_empty: Condvar::new(),
        };
        let wrapper = wrap(ValueType::Channel, data as *mut std::ffi::c_void);
        pin_handle(wrapper);
        wrapper
    }
}

/// # Safety
/// `chan` must be a channel `DlispValue` (or null, which yields `false`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_chan_send(
    chan: *mut DlispValue,
    val: *mut DlispValue,
) -> *mut DlispValue {
    let Some(data) = chan_data(chan) else {
        return dlisp_make_bool(false);
    };
    let mut state = match data.state.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if state.closed {
        return dlisp_make_bool(false);
    }
    unsafe {
        let node = dlisp_gc_malloc(std::mem::size_of::<ChanNode>()) as *mut ChanNode;
        *node = ChanNode {
            value: val,
            next: std::ptr::null_mut(),
        };
        if state.tail.is_null() {
            state.head = node;
        } else {
            (*state.tail).next = node;
        }
        state.tail = node;
    }
    drop(state);
    data.not_empty.notify_one();
    dlisp_make_bool(true)
}

/// Non-blocking receive. Returns the next value, or nil when the queue is
/// empty (whether open or closed).
///
/// # Safety
/// `chan` must be a channel `DlispValue` (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_chan_try_recv(chan: *mut DlispValue) -> *mut DlispValue {
    let Some(data) = chan_data(chan) else {
        return dlisp_make_nil();
    };
    let mut state = match data.state.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let value = pop_locked(&mut state);
    if value.is_null() {
        dlisp_make_nil()
    } else {
        value
    }
}
/// Blocking receive. Returns nil only when the channel is closed and drained.
///
/// # Safety
/// `chan` must be a channel `DlispValue` (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_chan_recv(chan: *mut DlispValue) -> *mut DlispValue {
    let mut closed: i64 = 0;
    unsafe { chan_poll(chan, -1, &mut closed) }
}

/// Shared receive core. Negative `timeout_ms` waits indefinitely, zero never
/// blocks, positive values bound each wait. The value and the closed flag
/// are observed under a single lock acquisition so no sent value can be
/// dropped in favour of a later `closed` observation.
unsafe fn chan_poll(
    chan: *mut DlispValue,
    timeout_ms: i64,
    out_closed: *mut i64,
) -> *mut DlispValue {
    let Some(data) = chan_data(chan) else {
        if !out_closed.is_null() {
            unsafe { *out_closed = 0 };
        }
        return dlisp_make_nil();
    };
    let deadline = (timeout_ms > 0)
        .then(|| std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms as u64));
    let mut state = match data.state.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    loop {
        // Value presence and closed flag are read under one lock hold: no
        // concurrent send can be dropped in favour of a later close.
        let value = pop_locked(&mut state);
        let closed = state.closed;
        if !value.is_null() || closed {
            drop(state);
            if !out_closed.is_null() {
                unsafe { *out_closed = closed as i64 };
            }
            return if value.is_null() {
                dlisp_make_nil()
            } else {
                value
            };
        }
        // Open and empty: wait for progress or deadline expiry.
        state = match deadline {
            None => match data.not_empty.wait(state) {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            },
            Some(deadline) => {
                let now = std::time::Instant::now();
                if now >= deadline {
                    drop(state);
                    if !out_closed.is_null() {
                        unsafe { *out_closed = 0 };
                    }
                    return dlisp_make_nil();
                }
                let remaining = deadline.saturating_duration_since(now);
                let (guard, _) = match data.not_empty.wait_timeout(state, remaining) {
                    Ok(result) => result,
                    Err(poisoned) => poisoned.into_inner(),
                };
                guard
            }
        };
    }
}

/// Atomic poll: pops the next value if present and reports the closed flag
/// under one lock acquisition. `timeout_ms` < 0 blocks until a value or
/// close, 0 returns immediately, > 0 waits at most that long.
///
/// # Safety
/// `chan` must be a channel `DlispValue` (or null); `out_closed` must point
/// to writable memory (or be null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_chan_poll(
    chan: *mut DlispValue,
    timeout_ms: i64,
    out_closed: *mut i64,
) -> *mut DlispValue {
    unsafe { chan_poll(chan, timeout_ms, out_closed) }
}

fn pop_locked(state: &mut ChanState) -> *mut DlispValue {
    if state.head.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let node = state.head;
        let value = (*node).value;
        state.head = (*node).next;
        if state.head.is_null() {
            state.tail = std::ptr::null_mut();
        }
        // Nodes are GC memory: dropping the reference lets the collector
        // reclaim them once unreachable.
        value
    }
}

/// # Safety
/// `chan` must be a channel `DlispValue` (or null). Idempotent.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_chan_close(chan: *mut DlispValue) -> *mut DlispValue {
    let Some(data) = chan_data(chan) else {
        return dlisp_make_nil();
    };
    let mut state = match data.state.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    state.closed = true;
    drop(state);
    data.not_empty.notify_all();
    dlisp_make_nil()
}

/// # Safety
/// `val` may be any `DlispValue`; it becomes the atom's initial contents.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_atom_new(val: *mut DlispValue) -> *mut DlispValue {
    unsafe {
        let data = dlisp_gc_malloc(std::mem::size_of::<AtomData>()) as *mut AtomData;
        *data = AtomData {
            value: Mutex::new(val),
        };
        let wrapper = wrap(ValueType::Atom, data as *mut std::ffi::c_void);
        pin_handle(wrapper);
        wrapper
    }
}

/// # Safety
/// `atom` must be an atom `DlispValue` (or null, which yields nil).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_atom_deref(atom: *mut DlispValue) -> *mut DlispValue {
    let Some(data) = atom_data(atom) else {
        return dlisp_make_nil();
    };
    let value = match data.value.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *value
}

/// # Safety
/// `atom` must be an atom `DlispValue` (or null, which yields nil).
/// Returns the stored value (`val`) on success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_atom_reset(
    atom: *mut DlispValue,
    val: *mut DlispValue,
) -> *mut DlispValue {
    let Some(data) = atom_data(atom) else {
        return dlisp_make_nil();
    };
    let mut slot = match data.value.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *slot = val;
    val
}

/// # Safety
/// `x` may be any `DlispValue`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_channel_p(x: *mut DlispValue) -> *mut DlispValue {
    dlisp_make_bool(!x.is_null() && unsafe { (*x).type_ } == ValueType::Channel)
}

/// # Safety
/// `x` may be any `DlispValue`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlisp_atom_p(x: *mut DlispValue) -> *mut DlispValue {
    dlisp_make_bool(!x.is_null() && unsafe { (*x).type_ } == ValueType::Atom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constructors::dlisp_make_int;

    #[test]
    fn channel_roundtrip() {
        unsafe {
            crate::gc::dlisp_gc_init();
            let ch = dlisp_chan_new();
            assert_eq!((*ch).type_, ValueType::Channel);
            let v = dlisp_make_int(42);
            let sent = dlisp_chan_send(ch, v);
            assert!((*sent).payload.bool_val, "send should deliver");
            let got = dlisp_chan_try_recv(ch);
            assert_eq!((*got).type_, ValueType::Int);
            assert_eq!((*got).payload.int_val, 42);
            let empty = dlisp_chan_try_recv(ch);
            assert_eq!((*empty).type_, ValueType::Nil);
        }
    }

    #[test]
    fn atom_roundtrip() {
        unsafe {
            crate::gc::dlisp_gc_init();
            let a = dlisp_atom_new(dlisp_make_int(7));
            assert_eq!((*a).type_, ValueType::Atom);
            let cur = dlisp_atom_deref(a);
            assert_eq!((*cur).payload.int_val, 7);
            let stored = dlisp_atom_reset(a, dlisp_make_int(9));
            assert_eq!((*stored).payload.int_val, 9);
            assert_eq!((*dlisp_atom_deref(a)).payload.int_val, 9);
        }
    }
}
