# Parser / Syntax Workflow

## Source of truth
- Parser: `core/src/parser.rs`
- AST/value shape: `core/src/ast.rs`
- Parser-local tests: inline tests in `core/src/parser.rs`, `core/src/parser_comments_test.rs`
- Behavior tests using parsed values: `core/tests/backquote_tests.rs`, `keyword_tests.rs`, `vector_tests.rs`, `map_tests.rs`

## Change checklist
- Confirm the desired `Value` representation before changing evaluator behavior.
- Keep syntax sugar lowered to existing forms when possible, e.g. quote/backquote forms.
- Add parser coverage for valid syntax, malformed input, and whitespace/comment edge cases.
- If parser output changes evaluator semantics, run the narrow evaluator test that covers the affected form/value.

## Validation
- Parser-only: `cargo test -p dlisp-core parser --quiet`
- Comment syntax: `cargo test -p dlisp-core parser_comments_test --quiet`
- Literal behavior: run the matching `core/tests/*_tests.rs` target.
