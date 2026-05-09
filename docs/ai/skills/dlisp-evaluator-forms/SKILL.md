---
name: dlisp-evaluator-forms
description: Use when changing dlisp evaluator behavior, special forms, macro expansion, apply semantics, environment scoping, defun, lambda, let, setq, defvar, control flow, or higher-order forms.
---

# dlisp Evaluator Forms

- Start with `core/src/interpreter.rs`, `core/src/interpreter/apply.rs`, and `core/src/forms/registry.rs`.
- Read [references/workflow.md](references/workflow.md) before editing a form.
- Use code and tests over prose docs when semantics disagree.
- If a form should compile, also use `dlisp-codegen-aot-jit`.
