# 言語仕様

このドキュメントは `dlisp` の評価モデルと意味論の要約です。特殊形式と組み込み関数の索引は `FUNCTIONS.md` を参照してください。

## Source of truth
- special forms: `core/src/forms/registry.rs`
- builtins: `core/src/builtins/mod.rs`
- evaluator: `core/src/interpreter.rs`
- behavioral tests: `core/tests/*.rs`, `cli/tests/integration_tests.rs`

## 値と truthiness
- `Integer`: 64-bit 整数
- `Float`: 64-bit 浮動小数点数
- `String`
- `Symbol`
- `Keyword`
- `List`
- `Vector`
- `Map`
- `Bool`
- `Nil`
- 関数値、macro 値、error 値も内部的に扱う

条件式では `nil` と整数 `0` を偽として扱い、それ以外は真として扱う。

比較 builtins(`>`, `<`, `=`, `>=`, `<=`, `/=`)はすべて `Bool` を返す。interpreter / JIT / AOT の全経路で同一である。

## 評価モデル
- Symbol は現在の environment から解決する。
- Keyword と数値、文字列などの self-evaluating な値はそのまま返る。
- List は最初の要素が special form なら special form として評価し、そうでなければ通常の関数呼び出しとして評価する。
- Vector と Map は中の要素を再帰的に評価する。
- macro expansion は評価前に走る。
- `defmacro` は registry ではなく evaluator 側で特別扱いされる。

## 束縛と関数

### `defun`
- グローバル関数を定義する。
- 関数本体は最後の式の値を返す。
- 固定引数、`&rest`、自己再帰、後続定義で解決される相互再帰、`spawn` を含む関数は JIT compile path に乗り得る。
- `spawn f arg...` は zero-arg thunk に lower して実行する。JIT 時点で未解決の呼び出しを含む関数は interpreter path にフォールバックする。

### `lambda`
- 現在の environment をキャプチャした関数値を作る。

### `defvar`
- グローバル変数を定義する。
- 既存値がある場合は上書きしない。

### `setq`
- 既存の束縛を更新する。

### `let` / `let*`
- `let` は並列バインディング。
- `let*` は逐次バインディング。
- destructuring と `&rest` を使うケースは対応テストを参照する。

## 制御
- `if`: 2 分岐
- `cond`: 多分岐
- `and` / `or`: 短絡評価
- `progn` / `do`: 順次評価して最後の値を返す
- `when` / `unless`: 条件付きブロック
- `while`, `dotimes`, `dolist`: 反復
- `loop` / `recur`: スタック安全な反復。`(loop [name init ...] body...)` は束縛を持ち、body 中の任意位置の `(recur expr ...)` が全束縛を再束縛して次の反復へ進む。recur 引数の個数は束縛数と一致が必要。loop を囲まずに recur が評価器の外層へ到達した場合はエラー。interpreter / JIT / AOT の全経路で同一意味論
- `match`: パターンマッチング(stdlib マクロ)。`(match expr (pattern body...)+)` の形で、expr は一度だけ評価される。pattern はリテラル(`=` の構造的等値)、シンボル束縛、`_` ワイルドカード、list/vector 分解 `[p0 p1]` / `[p0 p1 &rest r]`、map 分解 `{:k p}`(全キー存在必須)、ガード付き `(pat :when guard)`。ネストしたパターンも合成できる。guard 失敗は後続の節へ fall-through し、どの節もマッチしなければ `nil` を返す。guard は型検査しないため必要に応じ `number?` 等と併用する。macro 展開結果は lower 可能な式のみで構成され、interpreter / JIT / AOT の全経路で同一意味論
- `try` / `throw`: 例外系

## 評価補助とメタプログラミング
- `quote` は値を評価せずに返す
- `'expr` は `quote` の糖衣構文
- backquote 系は macro 展開系の処理で扱う
- `macroexpand` は macro の展開結果を確認する
- `eval` は式を再評価する
- `apply` はリスト状の引数で関数適用を行う
- `load` はファイルから式を読み込み順に評価する。相対 path は現在評価中のファイルのディレクトリ、または REPL/current working directory を基準に解決する
- `require` は module を一度だけ読み込む。`(require "core")` は同梱 stdlib module を読み込み、`./`、`../`、absolute path、または `.lisp` を含む文字列は file module として読み込む
- file `require` の相対 path は、呼び出し元ファイルのディレクトリを基準に解決する。AOT compile では required file の式を呼び出し元へ include したものとして扱う

## コレクションと高階操作
- List、Vector、Map を扱う builtins がある
- `map`, `filter`, `reduce`, `some`, `every`, `find`, `for-each`, `map-indexed`, `update`, `map-keys`, `map-vals` は current evaluator の評価規則に乗る special form として登録されている
- `(require "core")` の stdlib は Clojure 風のヘルパーを提供する: `member?`, `distinct`, `frequencies`, `group-by`, `merge-with`(2 map), `get-in`, `assoc-in`, `update-in`(unary f、パスは vector), `partition`, `interleave`。これらは Lisp 実装で、interpreter / JIT / AOT 全経路で compile される

## 並行実行
- `spawn` は別タスクで関数適用を走らせる
- REPL と evaluator は async ベースで動作する

## JIT / AOT
- `defun` された関数は interpreter path と JIT path の両方を持ち得る
- JIT 実行では boxed runtime value ABI を使って interpreter の `Value` と相互変換する
- `dlisp compile` は Cranelift backend を使って object を作り、`cc` で `runtime` と link する
- 変更後は interpreter 系テストと JIT / compile 系テストの両方で確認する
