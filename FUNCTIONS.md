# Functions and Special Forms Index

このファイルは `dlisp` の surface を素早く確認するための索引です。完全な source of truth ではありません。

## Source of truth
- special forms: `core/src/forms/registry.rs`
- builtins: `core/src/builtins/mod.rs`
- snapshot helper: `docs/ai/skills/dlisp-language-surface/scripts/extract-surface.sh`

## Special forms

定義と束縛:
- `defun`, `defvar`, `setq`, `lambda`, `let`, `let*`

制御:
- `if`, `cond`, `and`, `or`, `progn`, `do`, `when`, `unless`, `while`

反復:
- `dotimes`, `dolist`

評価と適用:
- `eval`, `apply`, `quote`, `macroexpand`, `load`

高階操作:
- `map`, `filter`, `reduce`, `some`, `every`, `find`, `for-each`, `map-indexed`, `update`, `map-keys`, `map-vals`

非同期と例外:
- `spawn`, `try`, `throw`

補足:
- `defmacro` は registry ではなく evaluator 側で特別扱いされる

## Builtins

算術:
- `+`, `-`, `*`, `/`, `%`, `mod`, `max`, `min`, `abs`, `pow`

比較:
- `>`, `<`, `=`, `>=`, `<=`, `/=`

表示と文字列化:
- `print`, `println`, `print-str`, `eprintln`, `format`, `str`

ファイルと OS:
- `read-file`, `write-file`, `file-exists?`, `is-dir?`, `is-file?`, `delete-file`, `list-dir`, `sh`, `exec`

環境:
- `getenv`, `setenv`, `cwd`, `set-cwd`, `args`, `exit`, `sleep`

List:
- `list`, `not`, `car`, `first`, `cdr`, `rest`, `cons`, `append`, `reverse`, `sort`, `last`, `butlast`, `flatten`, `range`, `take`, `drop`, `zip`

Vector:
- `vector`, `nth`, `count`, `conj`

Map:
- `hash-map`, `get`, `assoc`, `dissoc`, `keys`, `vals`, `contains?`, `merge`, `select-keys`

String:
- `string-length`, `substring`, `string-append`, `string-split`, `string-replace`, `string-upper`, `string-lower`, `string-trim`, `string-trim-left`, `string-trim-right`, `string-starts-with?`, `string-ends-with?`, `string-contains?`, `string-index-of`, `string->number`, `number->string`, `char-at`

Macro utils:
- `gensym`

Type / error:
- `nil?`, `error?`, `empty?`, `list?`, `number?`, `string?`, `symbol?`, `keyword?`, `vector?`, `map?`, `type-of`, `error-value`
