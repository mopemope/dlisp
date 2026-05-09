---
name: dlisp-codegen-aot-jit
description: Use when changing dlisp JIT, AOT compiler, Cranelift codegen, compiled builtin support, function lowering, runtime ABI declarations, compile command behavior, or interpreter/JIT/AOT parity.
---

# dlisp Codegen AOT JIT

- Start with `core/src/codegen/`, `core/src/jit.rs`, `core/src/jit_runner.rs`, and `core/src/compiler.rs`.
- Read [references/workflow.md](references/workflow.md) before editing lowering.
- Read [references/parity-map.md](references/parity-map.md) when builtin/form support differs between interpreter and compiled paths.
- If runtime symbols or value layout change, also use `dlisp-runtime-ffi`.
