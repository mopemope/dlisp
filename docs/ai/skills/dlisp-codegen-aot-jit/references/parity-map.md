# Interpreter / JIT / AOT Parity Map

Use this as a routing aid, not as source of truth. Verify current code before editing.

## Interpreter surface
- Special forms are registered in `core/src/forms/registry.rs`; `defmacro` is handled in `core/src/interpreter.rs`.
- Builtins are registered in `core/src/builtins/mod.rs`.
- Interpreter support does not imply JIT/AOT support.

## Compiled builtin dispatch
Compiled builtin names are listed in `core/src/codegen/context.rs` and lowered in `core/src/codegen/forms/builtins.rs`.

Compiled form lowering is selected in `core/src/codegen/context.rs` and implemented under `core/src/codegen/forms/`.

Currently represented compiled builtins include:
- Arithmetic/comparison: `+`, `-`, `*`, `/`, `%`, `mod`, `>`, `<`, `=`, `>=`, `<=`, `/=`
- Core values/collections: `print`, `not`, `list`, `cons`, `car`, `first`, `cdr`, `rest`, `vector`, `nth`, `count`, `conj`, `hash-map`, `assoc`, `get`
- Strings/types: `str`, `string-length`, `substring`, `string-append`, `nil?`, `list?`, `number?`, `string?`, `symbol?`, `keyword?`, `map?`, `vector?`, `type-of`
- Higher-order/runtime-backed: `map`, `filter`, `reduce`
- IO/sys/os subset: `read-file`, `file-exists?`, `is-dir?`, `is-file?`, `list-dir`, `delete-file`, `getenv`, `setenv`, `cwd`, `set-cwd`, `args`, `exit`, `sh`, `sleep`

Compiled special forms include:
- Binding/functions: `let`, `let*`, `setq`, `defvar`, `lambda`
- Control: `if`, `progn`, `do`, `when`, `unless`, `and`, `or`, `cond`
- Quoting/concurrency: `quote`, `spawn`

## Interpreter-only by default
Treat builtins absent from compiled dispatch as interpreter-only until codegen, runtime export, and tests prove otherwise. Common examples include extended list/map/string helpers such as `append`, `reverse`, `sort`, `keys`, `vals`, `select-keys`, `string-split`, and `string-replace`.

## Runtime export check
- Runtime exported symbols: `rg -n "extern \"C\" fn dlisp_" runtime/src`
- Codegen declarations: `rg -n "declare_function\\(\"dlisp_" core/src/codegen`
- Lowering dispatch: `rg -n "\"<name>\"" core/src/codegen/context.rs core/src/codegen/forms/builtins.rs`
