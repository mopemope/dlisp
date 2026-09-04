# Parser / Syntax Workflow

## Source of truth
- Parser: `core/src/parser.rs`
- AST/value shape: `core/src/ast.rs`
- Parser-local tests: inline tests in `core/src/parser.rs`
- Comment tests: `core/tests/parser_comments_tests.rs`
- Behavior tests using parsed values: `core/tests/backquote_tests.rs`, `keyword_tests.rs`, `vector_tests.rs`, `map_tests.rs`

## Change checklist
- Confirm the desired `Value` representation before changing evaluator behavior.
- Keep syntax sugar lowered to existing forms when possible, e.g. quote/backquote forms.
- Add parser coverage for valid syntax, malformed input, and whitespace/comment edge cases.
- If parser output changes evaluator semantics, run the narrow evaluator test that covers the affected form/value.

## Validation
- Parser/comment syntax: `cargo test -p dlisp-core --test parser_comments_tests --quiet`
- Literal behavior: run the matching `core/tests/*_tests.rs` target.
