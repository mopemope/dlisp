---
name: dlisp-builtins-surface
description: Use when implementing, registering, or changing dlisp builtins and their compiled runtime support.
---

# dlisp Builtins Surface

- Start with `core/src/builtins/mod.rs` and the target `core/src/builtins/*.rs` module.
- Read [references/change-checklist.md](references/change-checklist.md) before adding or renaming a builtin.
- Run `docs/ai/skills/dlisp-language-surface/scripts/extract-surface.sh check` after doc updates.
- If the builtin must work in compiled code, also use `dlisp-codegen-aot-jit` and `dlisp-runtime-ffi`.
- If public names or documented behavior change, also use `dlisp-language-surface`.
