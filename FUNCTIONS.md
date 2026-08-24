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
- `dotimes`, `dolist`, `loop` / `recur`(任意位置の recur で束縛を再束縛して反復)

評価と適用:
- `eval`, `apply`, `quote`, `macroexpand`, `load`, `require`

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
- `>`, `<`, `=`, `>=`, `<=`, `/=` — すべて Bool を返す(interpreter / JIT / AOT で同一)

表示と文字列化:
- `print`, `println`, `print-str`, `eprintln`, `format`, `str`

ファイルと OS:
- `read-file`, `write-file`, `file-exists?`, `is-dir?`, `is-file?`, `delete-file`, `list-dir`, `sh`, `exec`

環境:
- `getenv`, `setenv`, `cwd`, `set-cwd`, `args`, `exit`, `sleep`

List:
- `list`, `not`, `car`, `first`, `cdr`, `rest`, `cons`, `append`, `reverse`, `sort`, `last`, `butlast`, `flatten`, `take`, `drop`, `zip`

Vector:
- `vector`, `nth`, `count`, `conj`

Map:
- `hash-map`, `get`, `assoc`, `keys` — compiled(JIT/AOT)対応
- `dissoc`, `vals`, `contains?`, `merge`, `select-keys` — interpreter のみ

String:
- `string-length`, `substring`, `string-append`, `string-split`, `string-replace`, `string-upper`, `string-lower`, `string-trim`, `string-trim-left`, `string-trim-right`, `string-starts-with?`, `string-ends-with?`, `string-contains?`, `string-index-of`, `string->number`, `number->string`, `char-at`

Macro utils:
- `gensym`

Type / error:
- `nil?`, `error?`, `empty?`, `list?`, `number?`, `string?`, `symbol?`, `keyword?`, `vector?`, `map?`, `type-of`, `error-value`

## Bundled stdlib

`(require "core")` で同梱標準ライブラリを一度だけ読み込む。

`(require "./lib/foo.lisp")` のように file module も一度だけ読み込める。相対 path は呼び出し元ファイルのディレクトリ、REPL では current working directory を基準に解決する。

`core` module:
- `inc`, `dec`, `identity`, `constantly`, `second`, `third`
- `zero?`, `positive?`, `negative?`, `empty-list?`
- `when-let`
- `match` — パターンマッチングマクロ。`(match expr (pattern body...)+)`。リテラル / 束縛 / `_` / `[p...]`(+ `&rest`)/ `{:k p}` / `(pat :when guard)` をサポートし、no-match は `nil`。詳細は `spec.md` 制御セクション
- `range` — `(range end)` / `(range start end)` / `(range start end step)`。非整数引数や step 0 は `nil`。内部 helper として `range-build`, `range-iter` も定義される
- コレクション: `member?`, `distinct`, `frequencies`, `group-by`, `merge-with`(2 map), `get-in`, `assoc-in`, `update-in`(unary f、パスは vector), `partition`, `interleave` — 全経路 compile 対応
- `thread-first-step`, `thread-last-step`
- Threading macros: `->`, `->>`, `as->`, `some->`
