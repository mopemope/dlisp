---
name: dlisp-repo
description: Use when dlisp task ownership or validation scope is unclear; choose a specialist directly when the area is known.
---

# dlisp Repo

- Start with `rg --files` or `rg -n`; do not open broad docs first.
- If the task area is clear, use the matching specialist skill before reading broad files:
  `dlisp-parser-syntax`, `dlisp-evaluator-forms`, `dlisp-builtins-surface`,
  `dlisp-codegen-aot-jit`, `dlisp-runtime-ffi`, `dlisp-stdlib`, `dlisp-cli`,
  or `dlisp-parity-debugging`.
- Read [references/module-map.md](references/module-map.md) when ownership is unclear.
- Read [references/test-scope.md](references/test-scope.md) before choosing cargo commands.
- Open `README.md` only for user-facing behavior, CLI examples, or install docs.
- If docs disagree with code, trust `core/src/forms/registry.rs`, `core/src/builtins/mod.rs`, and tests.
