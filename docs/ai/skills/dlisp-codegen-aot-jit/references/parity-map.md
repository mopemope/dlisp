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
- Higher-order/runtime-backed: `map`, `filter`, `reduce`, `some`, `every`, `find`, `for-each`
- IO/sys/os subset: `read-file`, `file-exists?`, `is-dir?`, `is-file?`, `list-dir`, `delete-file`, `getenv`, `setenv`, `cwd`, `set-cwd`, `args`, `exit`, `sh`, `sleep`

Compiled special forms include:
- Binding/functions: `let`, `let*`, `setq`, `defvar`, `lambda`
- Control: `if`, `progn`, `do`, `when`, `unless`, `and`, `or`, `cond`,
  `while`, `dotimes`, `dolist`
- Quoting/concurrency: `quote`, `spawn`

## Interpreter-only by default
Builtins absent from `COMPILED_BUILTINS` are interpreter-only. The JIT gate
keeps such functions on the interpreter automatically; AOT compilation fails
with an explicit error naming the function and builtin.

Still interpreter-only examples include `sort`, `zip`, `range`, `format`,
`gensym`, `write-file`, `exec`, `select-keys`, `dissoc`, `merge`, `keys`,
`vals`, and the numeric helpers `abs`, `min`, `max`, `pow`.

Interpreter-only special forms (`try`, `throw`, `eval`, `apply`,
`macroexpand`, `load`, `require`, `defmacro`, `map-indexed`, `update`,
`map-keys`, `map-vals`) are rejected by the AOT precheck via
`is_interpreter_only_special_form` with an explicit error.

## Known minor divergences: error reporting
Compiled code cannot raise catchable errors, so a few builtins differ from
the interpreter in their failure mode (value-level results match):
- `(nth coll -1)` / out-of-range: interpreter errors on negative index;
  compiled code returns nil.
- `(car x)` / `(cdr x)` on non-lists: interpreter errors; compiled code
  returns nil.
- `(dotimes (k n))` with a negative `n`: interpreter errors; compiled code
  runs zero iterations. A non-integer count aborts via the comparison FFI
  (interpreter raises a catchable error).
- `(dolist (x coll))` with a non-collection: interpreter errors; compiled
  code runs zero iterations.
- `(some/every/find/for-each f coll)` with a non-collection: interpreter
  errors; compiled code returns the neutral value (nil, or true for every).
- Comparison return types: interpreter `>`/`<`/`=` builtins return Integer
  1/0 while compiled comparisons return Bool. Truthiness is identical.

Avoid relying on these errors inside functions that get JIT/AOT compiled.

## Parity fixes (verified)
These previously diverged and now match the interpreter; regression tests
live in `core/tests/jit_parity_tests.rs`,
`cli/tests/integration_tests.rs` (vector/collection and loop parity), and
`runtime/src/verify_tests.rs`:
- `dlisp_vector_count`: walks lists and treats nil as 0
- `dlisp_map` / `dlisp_filter` / `dlisp_reduce`: support vectors (shape
  preserved) and canonical truthiness (Int 0 falsy)
- `dlisp_some` / `dlisp_every` / `dlisp_find` / `dlisp_for_each`: compiled
  lowerings matching the interpreter special forms
- `dlisp_eq`: deep equality on lists and vectors
- `dlisp_vector_get`: supports list indices
- `dlisp_vector_to_list`: vector→list normalization used by `dolist`
- Vector literal lowering no longer reads a result from the void-returning
  `dlisp_vector_push`
- `while` / `dotimes` / `dolist`: Cranelift loop lowering in
  `core/src/codegen/forms/control.rs`

## JIT gate fixes (verified)
`find_uncompiled_call` (`core/src/forms/defun.rs`) previously rejected
functions that are perfectly compilable, silently keeping them on the
interpreter. Fixed false positives, pinned by
`test_jit_gate_accepts_let_loops_and_higher_order_forms`:
- `let` / `let*` binding names were treated as unresolved calls; binding
  heads are now locals and only init expressions are scanned as calls
- `lambda` parameter names were treated as unresolved calls inside the
  lambda body
- Env-unbound names in `COMPILED_BUILTINS` (the higher-order forms live in
  the interpreter registry, not the environment) were treated as
  unresolved; they now lower via codegen like any other compiled builtin

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
