# dlisp

Cranelift を使った JIT/AOT 対応の Lisp interpreter です。REPL 実行、スクリプト実行、ネイティブ実行ファイル生成を同じ workspace で扱います。

## Workspace
- `cli`: `dlisp` コマンド本体、REPL、ファイル実行、AOT compile サブコマンド
- `core`: parser、evaluator、special forms、builtins、JIT/AOT codegen
- `runtime`: AOT/JIT 実行時のランタイムと Boehm GC 連携
- `stdlib`: 最小の補助 crate

## Requirements
- Rust stable
- `cc`
- Boehm GC 開発パッケージ
- `pkg-config`

Ubuntu / Debian:

```bash
sudo apt-get install libgc-dev pkg-config build-essential
```

## Build / Run

Build:

```bash
cargo build --workspace
```

Run REPL:

```bash
cargo run --bin dlisp
```

Run a script:

```bash
cargo run --bin dlisp -- path/to/script.lisp
```

Split code across files with `require`:

```lisp
(require "./lib/math.lisp")
```

Compile to a native executable:

```bash
cargo run --bin dlisp -- compile path/to/script.lisp -o my_app
./my_app
```

Run all tests:

```bash
cargo test --workspace --quiet
```

## CLI

```text
dlisp [FILE] [COMMAND]

Commands:
  compile  Compile a script to a native executable
```

Compile options:

```text
-o, --output <OUTPUT>
-O, --optimize
    --release
```

## Runtime Notes
- REPL history: `~/.local/state/dlisp/history.txt`
- Debug log: `~/.local/state/dlisp/debug.log`
- `RUST_LOG=debug cargo run --bin dlisp` で tracing を有効化できる
- `runtime/build.rs` で `gc` を link する
- interpreter 実行では `defun` の一部が JIT 化される。固定引数、`&rest`、自己再帰、相互再帰、`spawn` を含む関数が対象で、`spawn f arg...` は zero-arg thunk に lower して実行する

## Quick Example

```lisp
user> (+ 1 2 (* 3 4))
=> 15
user> (defun square (x) (* x x))
=> <user-func:square>
user> (square 5)
=> 25
```

## More Docs
- 言語の意味論: `spec.md`
- 特殊形式と組み込み関数の索引: `FUNCTIONS.md`
- AI / Skill 運用: `docs/ai/README.md`
- repo 内での最短探索ルール: `AGENTS.md`
