---
name: dlisp-stdlib
description: Use when adding or changing Lisp-defined standard library functions in stdlib/src/core.lisp or their require behavior.
---

# dlisp Standard Library

- Read the target definition in `stdlib/src/core.lisp` and its tests in `core/tests/stdlib_tests.rs`; inspect `core/src/forms/require.rs` only for loading issues.
- Check definition order and compiled dependencies in the stdlib section of [parity-map](../dlisp-codegen-aot-jit/references/parity-map.md). Verify current code/tests before applying a remembered limitation.
- Preserve list/vector shape, nil behavior, and existing error contracts. Interpreter success does not prove compiled support.
- Start with the affected `stdlib_tests` case; loading changes also need `require_tests`, loops need `loop_recur_tests`.
- For compiled functions, assert JIT availability and verify AOT behavior via `cli/tests/parity_golden_tests.rs`; use `dlisp-codegen-aot-jit` if lowering changes are needed.
- Use `dlisp-language-surface` when public names or documented semantics change.
