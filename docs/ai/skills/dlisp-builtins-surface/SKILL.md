---
name: dlisp-builtins-surface
description: Use when adding or changing dlisp builtins, builtin registration, language surface docs, FUNCTIONS.md entries, or builtin parity with JIT/AOT/runtime.
---

# dlisp Builtins Surface

- Start with `core/src/builtins/mod.rs` and the target `core/src/builtins/*.rs` module.
- Read [references/change-checklist.md](references/change-checklist.md) before adding or renaming a builtin.
- Run `docs/ai/skills/dlisp-language-surface/scripts/extract-surface.sh check` after doc updates.
- If the builtin must work in compiled code, also use `dlisp-codegen-aot-jit` and `dlisp-runtime-ffi`.
