# Source of Truth

## Canonical code locations
- special form names: `core/src/forms/registry.rs`
- builtin names: `core/src/builtins/mod.rs`
- default environment setup: `core/src/interpreter.rs`
- AOT compile entry: `core/src/compiler.rs`, `cli/src/compile.rs`
- runtime exports: `runtime/src/lib.rs`

## Canonical behavior checks
- evaluator semantics: `core/tests/interpreter_tests.rs`
- JIT/codegen coverage: `core/tests/jit_phase3_tests.rs`, `core/tests/compiler_*`
- CLI/examples: `cli/tests/integration_tests.rs`
- language features by topic: `core/tests/*.rs`

## Update rules
- コードとテストを先に確認し、その後で prose docs を直す。
- 文書がコードと食い違う場合は、コードとテストを正とする。
- `FUNCTIONS.md` は surface index。完全仕様書として扱わない。
- `spec.md` は評価モデルと意味論を置く。全 API 一覧は置かない。
- `README.md` は導入、実行、CLI 利用に絞る。

## Search hints
- special forms を一覧したいとき: `sed -n 's/.*reg.register(\"\\([^\"]*\\)\".*/\\1/p' core/src/forms/registry.rs`
- builtins を一覧したいとき: `sed -n 's/.*env.set(\"\\([^\"]*\\)\".*/\\1/p' core/src/builtins/mod.rs`
