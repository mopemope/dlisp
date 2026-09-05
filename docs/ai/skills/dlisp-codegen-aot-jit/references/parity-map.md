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
- Core values/collections: `print`, `not`, `list`, `cons`, `car`, `first`, `cdr`, `rest`, `vector`, `nth`, `count`, `conj`, `hash-map`, `assoc`, `get`, `keys`
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
  `while`, `dotimes`, `dolist`, `loop` / `recur`(recur は最内 loop の
  束縛を一時変数経由で再定義し body 先頭へ jump; interpreter の
  abort-on-recur 段階的実行に対応するため後続式は dead block へ)
- Quoting/concurrency: `quote`, `spawn`
- Errors: `try` / `throw` (sentinel propagation; see below)

## Interpreter-only by default
Builtins absent from `COMPILED_BUILTINS` are interpreter-only. The JIT gate
keeps such functions on the interpreter automatically; AOT compilation fails
with an explicit error naming the function and builtin.

Still interpreter-only examples include `sort`, `zip`, `format`,
`gensym`, `write-file`, `exec`, `select-keys`, `dissoc`, `merge`,
`vals`, and the numeric helpers `abs`, `min`, `max`, `pow`.
`error?` and `error-value` are compiled builtins.
`range` used to be an interpreter-only builtin; it now lives in the bundled
stdlib (`stdlib/src/core.lisp`) as Lisp code and compiles on every path.

Interpreter-only special forms (`eval`, `apply`,
`macroexpand`, `load`, `require`, `defmacro`, `map-indexed`, `update`,
`map-keys`, `map-vals`) are rejected by the AOT precheck via
`is_interpreter_only_special_form` with an explicit error.

## Error handling (try / throw) on compiled paths
`try`/`throw` compile on every path via sentinel propagation
(`runtime/src/errors.rs`):
- `throw` stores the thrown value in thread-local state and returns a
  reserved sentinel pointer (`dlisp_throw_sentinel`).
- User call sites (`compile_function_call`, `compile_indirect_call`) compare
  the result against the sentinel and escape: jump to the innermost `try`
  catch block, or return the sentinel out of the current function.
- `try` registers its catch block on `ctx.try_frames` while the body is
  compiled; the handler takes the pending value, wraps it in an error
  marker (`dlisp_make_error`), and binds the catch variable — matching the
  interpreter's `Value::Error` wrapper.
- The JIT boundary in `core/src/interpreter/apply.rs` converts a throw
  raised *during the call* (transition on `dlisp_thrown_pending`, not the
  absolute flag — a stale flag from a swallowed higher-order-callback throw
  must not poison unrelated calls) into
  `EvalFailure::Message("DLISP_THROW")` + `interpreter.last_error` so an
  enclosing interpreter `try` can catch it without re-running the body.
- A catch-less `try` escapes to the enclosing try's catch block when one
  exists (the frame stack is popped before the escape target is chosen) and
  otherwise returns the sentinel out of the current function, matching the
  interpreter's outward bubbling. `(try)` evaluates to nil on all paths.
- An uncaught throw escaping the AOT user main is detected by the runtime
  entry shim (`run_user_main_wrapper` in `runtime/src/task.rs`), which
  prints `Error in main: DLISP_THROW` and exits 1 like the interpreter.
- The JIT gate treats the trailing `(catch var handler...)` clause of `try`
  as structure (not a call); the catch variable is scoped to the handler
  body. `test_jit_gate_compiles_try_catch` pins this.
- Caught values print as `<error v>`; `type-of` returns `"error"`.

Limitations (compile-time safe, but documented):
- Throws thrown inside closures passed to higher-order builtins
  (`map`/`filter`/`reduce`/`some`/`every`/`find`/`for-each`) are NOT
  propagated: the runtime helpers do not check for the sentinel, so a
  thrown value can end up as a list element. Keep `throw` out of
  higher-order callbacks on compiled paths.
- An uncaught `throw` in a spawned task (OS thread) is reported as
  "Uncaught throw in spawned task" and cleared; it cannot be caught by a
  `try` in another task.
- Runtime FFI type errors still abort (`eprintln!` + `process::abort`) —
  see the error-reporting divergences above; converting those aborts into
  catchable throws is future work.

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
- Concurrent `spawn` output: compiled tasks run on OS threads, so stdout
  writes can interleave between the text and its newline. The interpreter's
  cooperative scheduler keeps each `(print ...)` atomic. Line content is
  identical; ordering/interleaving is not guaranteed in either path.
  Pinned by the order-insensitive comparison for `spawn.lisp` in
  `cli/tests/parity_golden_tests.rs`.

Comparison builtins (`>`, `<`, `=`, `>=`, `<=`, `/=`) return `Bool` on all
paths (unified; previously the interpreter returned Integer 1/0 for
`>`, `<`, `=`). Byte-exact interpreter-vs-AOT parity for all bundled
examples except spawn interleaving is enforced by
`cli/tests/parity_golden_tests.rs`.

Avoid relying on error divergences inside functions that get JIT/AOT
compiled.

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

## Concurrency builtins (chan / send / recv / try-recv / close, atom family)
Value-level semantics are identical on every path by construction: the
interpreter builtins and the compiled lowerings call the same runtime FFI
(`runtime/src/concurrency.rs`), whose state lives in GC-allocated blocks so
queued values stay traced. `send` returns a bool instead of raising, so no
catchable-error divergence exists for these builtins.

Handles handed to the interpreter are raw wrapper addresses, which Boehm
cannot see inside interpreter environments; every channel/atom wrapper is
therefore pinned for process lifetime in a registry allocated from GC
memory (`pin_handle` in `concurrency.rs`). Creating sync primitives in
unbounded loops would grow that registry accordingly.

The one intentional difference is *waiting*: interpreted `recv` calls
`dlisp_chan_poll` with a 1ms timeout plus `yield_now`, because blocking the
single-threaded executor indefinitely would stall cooperative tasks.
Compiled `recv` uses the blocking `dlisp_chan_recv`. Both observe value and
closed state under a single lock acquisition (`dlisp_chan_poll`), so no
sent value can be lost to a later close observation. Keep spawn/recv pairs
in the same function anyway: an interpreted `(spawn ...)` (LocalSet task)
combined with a `recv` inside a separately compiled function adds up to 1ms
scheduling latency per step and is best avoided (see
`example-lisp/concurrency.lisp`, byte-exact golden).

## Adding a compiled builtin (checklist)
1. runtime FFI export in `runtime/src/<feature>.rs` (+ `lib.rs` re-export)
2. symbol registration in `core/src/jit.rs`
3. signature declaration + field in `core/src/codegen/builtins.rs`
4. lowering arm in `core/src/codegen/forms/builtins.rs`
5. name added to `COMPILED_BUILTINS` (`core/src/codegen/mod.rs`)
6. dummy symbol in `core/tests/jit_phase3_tests.rs` + compile test
7. behavior check via REPL defun (auto-JIT) and AOT compile

`scripts/check_codegen_parity.py` statically verifies steps 1-5 stay in sync.

## Defining stdlib functions instead of builtins
Interpreter-only surface can move to `stdlib/src/core.lisp` as Lisp code;
`(require "core")` forms are compiled into AOT output by
`collect_required_module` (`core/src/compiler.rs`). Constraints learned from
the `range` migration:
- Define callees before callers: each defun eagerly JIT-compiles at
  definition time, and forward references leave the caller unresolvable
  ("can't resolve symbol ..." panic when a later function is JIT-compiled).
- `nth` is vector-only; use `first`/`second`/`third` (car/cdr) for lists.
- Avoid interpreter-only special forms (`try`, `throw`, `apply`, ...) inside
  stdlib defuns; they make the definition uncompilable.
- Degenerate inputs should yield neutral values (e.g. nil) since compiled
  code cannot raise catchable errors.

## Runtime export check
- Runtime exported symbols: `rg -n "extern \"C\" fn dlisp_" runtime/src`
- Codegen declarations: `rg -n "declare_function\\(\"dlisp_" core/src/codegen`
- Lowering dispatch: match arms in `core/src/codegen/forms/builtins.rs`

## Known limitation: cross-function recur
Interpreter `recur` escapes dynamically, so a helper called from inside a
`loop` can recur the caller's loop. Compiled code resolves `recur` against
the compile-time loop stack; a `recur` inside a lambda/defun with no
lexically enclosing `loop` fails codegen ("recur outside loop"), which makes
the JIT gate keep such functions on the interpreter and makes AOT reject the
function. Keep `recur` lexically inside its `loop` for compiled paths.
