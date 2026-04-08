---
name: dlisp-language-surface
description: Use when updating dlisp language docs, builtins, special forms, examples, or user-facing surface summaries. Helps use code and tests as source of truth and keep docs concise.
---

# dlisp Language Surface

- Start with `core/src/forms/registry.rs` and `core/src/builtins/mod.rs`.
- Run [scripts/extract-surface.sh](scripts/extract-surface.sh) before editing `FUNCTIONS.md` or `spec.md`.
- Read [references/source-of-truth.md](references/source-of-truth.md) when behavior is unclear.
- Read [references/doc-ownership.md](references/doc-ownership.md) before adding or moving prose.
- Keep docs short; avoid copying the same function list into multiple files.
