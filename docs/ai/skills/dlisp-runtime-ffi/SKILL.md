---
name: dlisp-runtime-ffi
description: Use when changing dlisp runtime crate, FFI exports, value ABI, GC allocation, runtime maps/vectors, OS/IO/sys helpers, or symbols called by JIT/AOT compiled code.
---

# dlisp Runtime FFI

- Start with `runtime/src/value.rs` and `runtime/src/lib.rs`.
- Read [references/workflow.md](references/workflow.md) before changing value layout or exported symbols.
- Keep runtime symbol names and signatures aligned with `core/src/codegen/builtins.rs`.
- If interpreter builtins are affected, also use `dlisp-builtins-surface`.
