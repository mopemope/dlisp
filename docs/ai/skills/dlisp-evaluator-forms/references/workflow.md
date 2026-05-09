# Evaluator / Forms Workflow

## Source of truth
- Evaluation loop and macro expansion: `core/src/interpreter.rs`
- Function application: `core/src/interpreter/apply.rs`
- Form registry: `core/src/forms/registry.rs`
- Form implementations: `core/src/forms/*.rs`
- Binding model: `core/src/environment.rs`

## Change checklist
- Decide whether the form evaluates, delays, or rewrites each argument.
- Keep macro expansion rules aligned with evaluator special cases.
- For binding changes, cover global, local, closure, shadowing, and destructuring behavior when relevant.
- For new public forms, register in `standard_registry()` and update language surface docs through `dlisp-language-surface`.
- If `defun` should JIT compile the new construct, update codegen support instead of relying on interpreter fallback accidentally.

## Validation
- General evaluator: `cargo test -p dlisp-core --test interpreter_tests --quiet`
- Special forms: `cargo test -p dlisp-core --test special_forms --quiet`
- Scope/binding: `cargo test -p dlisp-core --test defvar_tests --quiet`, `setq_tests`, `let_star_tests`, `destructure_tests`
- Macros: `cargo test -p dlisp-core --test macros --quiet`, `macro_utils_tests`, `backquote_tests`
