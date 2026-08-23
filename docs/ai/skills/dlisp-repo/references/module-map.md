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
- runtime export: `runtime/src/lib.rs`(crate-root `dlisp_*` は各モジュールからの re-export)

## Task routing
- parser / syntax: `core/src/parser.rs`, `core/src/ast.rs`, `core/tests/parser_comments_tests.rs`
- evaluator / forms: `core/src/interpreter.rs`, `core/src/interpreter/apply.rs`, `core/src/forms/`
- builtins: `core/src/builtins/mod.rs`, `core/src/builtins/*.rs`
- JIT / AOT: `core/src/codegen/`, `core/src/jit.rs`, `core/src/jit_runner.rs`, `core/src/compiler.rs`, `cli/src/compile.rs`
- runtime / FFI: `runtime/src/lib.rs`(re-export hub), `value.rs`, `gc.rs`, `constructors.rs`,
  `{lists,maps,collections,arith,cmp,strings,predicates,print,task}.rs`, `{io,sys,os,vectors,higher_order}.rs`
- CLI / REPL: `cli/src/main.rs`, `cli/src/repl.rs`, `cli/src/config.rs`

## Search hints
- special form の実装位置: `rg -n "reg.register\\(|struct .*Form" core/src/forms core/src/forms/registry.rs`
- builtin の実装位置: `rg -n "env.set\\(" core/src/builtins/mod.rs`
- evaluator の経路: `rg -n "eval_special_form|apply\\(" core/src/interpreter.rs core/src/interpreter`
- compile 経路: `rg -n "compile\\(|compile_file|ObjectModule" cli/src core/src`
- runtime export: `rg -n "extern \"C\" fn dlisp_" runtime/src`
- user-visible examples: `rg -n "" example-lisp cli/tests core/tests`
