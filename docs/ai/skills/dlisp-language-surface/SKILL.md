---
name: dlisp-language-surface
description: Use when editing dlisp public language documentation, function lists, or documented syntax and semantics.
---

# dlisp Language Surface

- Locate names with the generated surface/index, then verify registration in `core/src/forms/registry.rs` and `core/src/builtins/mod.rs` and behavior in tests.
- Run [scripts/extract-surface.sh](scripts/extract-surface.sh) before editing `FUNCTIONS.md` or `spec.md`.
- Run `scripts/extract-surface.sh check` after editing `FUNCTIONS.md` (`--check` also works).
- Read [references/source-of-truth.md](references/source-of-truth.md) when behavior is unclear.
- Read [references/doc-ownership.md](references/doc-ownership.md) before adding or moving prose.
- Keep docs short; avoid copying the same function list into multiple files.
