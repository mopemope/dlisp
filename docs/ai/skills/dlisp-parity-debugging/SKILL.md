---
name: dlisp-parity-debugging
description: Use when dlisp interpreter, JIT, or AOT results diverge, or a compiled-path test may silently use interpreter fallback.
---

# dlisp Execution Path Differences

- Reduce the failure to one program and record expected values, output counts, errors/exit status, and state changes.
- Verify the path before diagnosing semantics: inspect `Value::UserFunc::jit_code`, assert `Some` for JIT, and run the AOT-generated executable. A test name containing JIT or successful codegen alone is insufficient.
- For pure interpreter comparison, clone the test function with `jit_code: None` in a fresh environment before calling it. Ensure nested callees do not retain compiled pointers; use a small fixture whose call graph is controlled.
- Compare the same inputs against explicit expected results on each path. Concurrent output comparisons must preserve duplicate counts.
- Inspect `core/src/forms/defun.rs` for gate failures, `core/src/interpreter/apply.rs` and `core/src/jit_runner.rs` for conversion/dispatch, then the affected lowering/runtime implementation.
- Read only the relevant section of [parity-map](../dlisp-codegen-aot-jit/references/parity-map.md); pin an intentional divergence with a regression test and reason.
- Start with the relevant `core/tests/jit_parity_tests.rs` case and `cli/tests/parity_golden_tests.rs`; runtime changes also need the runtime crate tests.
