# Interpreter / JIT / AOT Parity Map

Use this as a routing aid, not as source of truth. Verify current code before editing.

## Source of truth for compiled builtins
`core/src/codegen/mod.rs` の `COMPILED_BUILTINS` が唯一の dispatch 一覧。
JIT gating (`forms/defun.rs`) と AOT 事前チェック (`compiler.rs`) はこれを参照する。

## Interpreter surface
- Special forms are registered in `core/src/forms/registry.rs`; `defmacro` is handled in `core/src/interpreter.rs`.
- Builtins are registered in `core/src/builtins/mod.rs`.
- Interpreter support does not imply JIT/AOT support.

## Compiled builtin dispatch
Compiled builtin names are listed in `COMPILED_BUILTINS` (`core/src/codegen/mod.rs`) and lowered in `core/src/codegen/forms/builtins.rs`.

Currently represented compiled builtins include:
- Arithmetic/comparison: `+`, `-`, `*`, `/`, `%`, `mod`, `>`, `<`, `=`, `>=`, `<=`, `/=`
- Core values/collections: `print`, `not`, `list`, `cons`, `car`, `first`, `cdr`, `rest`, `vector`, `nth`, `count`, `conj`, `hash-map`, `assoc`, `get`
- List helpers: `append`, `reverse`, `last`, `butlast`, `flatten`, `take`, `drop`, `empty?`
- Strings/types: `str`, `string-length`, `substring`, `string-append`,
  `string-split`, `string-replace`, `string-upper`, `string-lower`,
  `string-trim`, `string-trim-left`, `string-trim-right`,
  `string-starts-with?`, `string-ends-with?`, `string-contains?`,
  `string-index-of`, `string->number`, `number->string`, `char-at`,
  `nil?`, `list?`, `number?`, `string?`, `symbol?`, `keyword?`, `vector?`, `map?`, `type-of`
- Higher-order/runtime-backed: `map`, `filter`, `reduce`
- IO/sys/os subset: `read-file`, `file-exists?`, `is-dir?`, `is-file?`, `list-dir`, `delete-file`, `getenv`, `setenv`, `cwd`, `set-cwd`, `args`, `exit`, `sh`, `sleep`

Compiled special forms include:
- Binding/functions: `let`, `let*`, `setq`, `defvar`, `lambda`
- Control: `if`, `progn`, `do`, `when`, `unless`, `and`, `or`, `cond`
- Quoting/concurrency: `quote`, `spawn`

## Interpreter-only by default
Builtins absent from `COMPILED_BUILTINS` are interpreter-only. The JIT gate
keeps such functions on the interpreter automatically; AOT compilation fails
with an explicit error naming the function and builtin.

Still interpreter-only examples include `sort`, `zip`, `range`, `format`,
`gensym`, `write-file`, `exec`, `select-keys`, `dissoc`, `merge`, `keys`,
`vals`, and the numeric helpers `abs`, `min`, `max`, `pow`.

## Known limitation: builtin name shadowing
In compiled code (JIT/AOT), names in `COMPILED_BUILTINS` always lower to the
runtime FFI and ignore user redefinitions (`(defun last ...)` etc.). Pure
interpreter execution resolves user bindings first, so programs that shadow
these names may behave differently depending on which functions get compiled.
This is a pre-existing dispatch design; avoid shadowing compiled builtin names.

## Adding a compiled builtin (checklist)
1. runtime FFI export in `runtime/src/<feature>.rs` (+ `lib.rs` re-export)
2. symbol registration in `core/src/jit.rs`
3. signature declaration + field in `core/src/codegen/builtins.rs`
4. lowering arm in `core/src/codegen/forms/builtins.rs`
5. name added to `COMPILED_BUILTINS` (`core/src/codegen/mod.rs`)
6. dummy symbol in `core/tests/jit_phase3_tests.rs` + compile test
7. behavior check via REPL defun (auto-JIT) and AOT compile

## Runtime export check
- Runtime exported symbols: `rg -n "extern \"C\" fn dlisp_" runtime/src`
- Codegen declarations: `rg -n "declare_function\\(\"dlisp_" core/src/codegen`
- Lowering dispatch: match arms in `core/src/codegen/forms/builtins.rs`
