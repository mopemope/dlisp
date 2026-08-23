# Builtin Change Checklist

## Source of truth
- Registration: `core/src/builtins/mod.rs`
- Implementations: `core/src/builtins/*.rs`
- User-facing index: `FUNCTIONS.md`
- Compiler support: `core/src/codegen/context.rs`, `core/src/codegen/forms/builtins.rs`
- Runtime support for compiled code: `runtime/src/*.rs`

## Interpreter path
- Implement the builtin in the smallest matching module.
- Register the public name in `install()`.
- Keep arity and type errors consistent with nearby functions.
- Add tests in the focused `core/tests/*` target or module-local tests.

## Compiled path
- Do not assume interpreter registration makes JIT/AOT work.
- If compiled support is required, add runtime export, codegen declaration, codegen dispatch, and compile/integration tests.
- If compiled support is intentionally absent, document it in the codegen parity reference.

## Validation
- Focused builtin tests first, e.g. `map_tests`, `vector_tests`, `format_tests`, `io_tests`, `sys_tests`, `os_tests`.
- Public surface + codegen sync: `scripts/check.sh` (fmt / clippy / surface check / codegen parity / generated docs freshness). Regenerate docs with `scripts/check.sh --regenerate`.
- Compiled path only when touched: `cargo test -p dlisp-core --test compiler_tests --quiet` or `cargo test -p dlisp --test integration_tests --quiet`
- Interpreter ↔ AOT behavior parity for examples: `cargo test -p dlisp --test parity_golden_tests --quiet`
