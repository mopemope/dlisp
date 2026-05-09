# Codegen / JIT / AOT Workflow

## Source of truth
- JIT entry: `core/src/jit.rs`, `core/src/jit_runner.rs`
- AOT entry: `core/src/compiler.rs`, `cli/src/compile.rs`
- Lowering context: `core/src/codegen/context.rs`
- Builtin declarations: `core/src/codegen/builtins.rs`
- Builtin lowering: `core/src/codegen/forms/builtins.rs`
- Form lowering: `core/src/codegen/forms/*.rs`

## Change checklist
- Confirm whether the feature needs interpreter-only, JIT, AOT, or all paths.
- Add/adjust runtime ABI declarations before lowering calls to new runtime functions.
- Keep function signatures aligned across runtime externs, `BuiltinDefinitions`, and call lowering.
- Add compile-path tests for any public behavior that should work after `dlisp compile`.

## Validation
- JIT/codegen: `cargo test -p dlisp-core --test jit_phase3_tests --quiet`
- AOT compiler: `cargo test -p dlisp-core --test compiler_tests --quiet`, `compiler_integration_tests`
- CLI compile: `cargo test -p dlisp --test integration_tests --quiet`
- Runtime ABI touched: `cargo test -p dlisp_runtime --quiet`
