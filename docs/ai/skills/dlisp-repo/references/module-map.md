# Module Map

## Crates
- `cli`: CLI、REPL、ファイル実行、AOT compile の入口
- `core`: evaluator、special forms、builtins、parser、JIT/AOT codegen
- `runtime`: Boehm GC、FFI export、OS/IO/runtime helper
- `stdlib`: 最小 crate。現状の変更頻度は低い

## Common entry points
- CLI 起動: `cli/src/main.rs`
- REPL: `cli/src/repl.rs`
- AOT compile: `cli/src/compile.rs`, `core/src/compiler.rs`
- evaluator: `core/src/interpreter.rs`
- special forms registry: `core/src/forms/registry.rs`
- builtins registry: `core/src/builtins/mod.rs`
- parser: `core/src/parser.rs`
- JIT 実装: `core/src/jit.rs`, `core/src/jit_runner.rs`
- runtime export: `runtime/src/lib.rs`

## Search hints
- special form の実装位置: `rg -n "reg.register\\(|struct .*Form" core/src/forms core/src/forms/registry.rs`
- builtin の実装位置: `rg -n "env.set\\(" core/src/builtins/mod.rs`
- evaluator の経路: `rg -n "eval_special_form|apply\\(" core/src/interpreter.rs core/src/interpreter`
- compile 経路: `rg -n "compile\\(|compile_file|ObjectModule" cli/src core/src`
- user-visible examples: `rg -n "" example-lisp cli/tests core/tests`
