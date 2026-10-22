---
name: dlisp-cli
description: Use when changing dlisp CLI arguments, file execution, REPL behavior, diagnostics, or process exit status.
---

# dlisp CLI

- Route argument/file execution to `cli/src/main.rs`, REPL to `cli/src/repl.rs`, logging to `cli/src/config.rs`, AOT commands to `cli/src/compile.rs`.
- File read, parse, top-level evaluation, and user `main` errors are distinct paths. Pin exit status, stdout/stderr, and whether later expressions execute; keep interactive recovery separate from file execution.
- Start with the affected test in `cli/tests/integration_tests.rs`. Public compile behavior also needs `parity_golden_tests`; lowering changes need `dlisp-codegen-aot-jit`.
- Consult the relevant README section only when user-facing CLI usage changes.
