# Runtime / FFI Workflow

## Source of truth
- Runtime value ABI: `runtime/src/value.rs`
- GC bindings and allocation: `runtime/src/gc.rs`
- Value constructors / cons: `runtime/src/constructors.rs`
- Feature exports: `runtime/src/{lists,maps,collections,arith,cmp,strings,predicates,print,task}.rs`
- Crate-root re-exports: `runtime/src/lib.rs`(外部・テストは `crate::dlisp_*` パスを維持)
- Runtime modules: `runtime/src/io.rs`, `sys.rs`, `os.rs`, `vectors.rs`, `higher_order.rs`
- Codegen declarations: `core/src/codegen/builtins.rs`
- JIT conversion: `core/src/jit_runner.rs`

## Change checklist
- Keep `#[repr(C)]` layout changes explicit and update all pointer offset assumptions in codegen/JIT runner.
- For new exported runtime functions, add matching declaration in `core/src/codegen/builtins.rs` before calling it from lowering.
- Return `*mut DlispValue` consistently unless the existing ABI for that operation uses a scalar.
- Initialize or allocate through existing GC helpers; avoid stack pointers escaping through FFI.
- Cover null, nil, wrong-type, and ownership-sensitive inputs when the nearby runtime code handles them.

## Validation
- Runtime only: `cargo test -p dlisp_runtime --quiet`
- JIT conversion touched: `cargo test -p dlisp-core --test jit_phase3_tests --quiet`
- AOT symbol touched: `cargo test -p dlisp-core --test compiler_tests --quiet` and CLI compile integration when public behavior changes.
