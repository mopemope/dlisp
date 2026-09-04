# Agent Guide

このリポジトリでは、token 消費を抑えるために「最小探索・最小検証」を徹底すること。

## 基本方針
- チャットは日本語で行う。
- Python 実行は使用可。単純な探索は `rg` / shell を優先し、繰り返す検証や正確な抽出は Python 補助スクリプトを使ってよい。
- まず `rg --files` / `rg -n` で当たりを付け、必要なファイルだけ読む。
- `README.md` 全文を最初から読まない。ユーザー向け挙動、公開文書、導入手順を触るときだけ必要箇所を開く。
- 言語仕様や関数一覧を直すときも、最初に prose docs を読むのではなく `core/src/forms/registry.rs` と `core/src/builtins/mod.rs` を見る。
- 変更後は関係する最小コマンドで検証し、ワークスペース全体テストは最後に限定する。

## 探索順
1. `Cargo.toml` で workspace 境界を確認する。
2. `rg -n "<symbol>|<feature>" cli core runtime stdlib` で実装位置を絞る。
3. repo 固有の案内が必要なら `docs/ai/skills/` 以下を読む。
4. 文書がコードと食い違う場合は、コードとテストを優先する。

## タスク別ルーティング
- parser / syntax: `core/src/parser.rs`, `core/src/ast.rs` -> `core/tests/*parser*`, syntax 関連テスト。必要なら `docs/ai/skills/dlisp-parser-syntax/`。
- evaluator / special forms: `core/src/interpreter.rs`, `core/src/forms/registry.rs`, `core/src/forms/` -> 対象 form の `core/tests/*`。必要なら `docs/ai/skills/dlisp-evaluator-forms/`。
- builtins / language surface: `core/src/builtins/mod.rs`, 対象 `core/src/builtins/*.rs` -> 対象 builtin の単体/統合テスト。必要なら `docs/ai/skills/dlisp-builtins-surface/`。
- JIT / AOT / codegen: `core/src/codegen/`, `core/src/jit.rs`, `core/src/compiler.rs`, `cli/src/compile.rs` -> JIT/codegen/CLI compile テスト。必要なら `docs/ai/skills/dlisp-codegen-aot-jit/`。
- runtime / FFI / GC: `runtime/src/value.rs`, `runtime/src/lib.rs`, `runtime/src/{gc,constructors,lists,maps,collections,arith,cmp,strings,predicates,print,task,io,sys,os,vectors,higher_order}.rs` -> `cargo test -p dlisp_runtime --quiet` と必要な compile 経路。必要なら `docs/ai/skills/dlisp-runtime-ffi/`。
- stdlib (`stdlib/src/core.lisp`): Lisp 実装の標準ライブラリ -> `core/tests/stdlib_tests.rs`, `require_tests`, `loop_recur_tests`。制約(定義順、`nth` は vector のみ、interpreter-only form 禁止、nil 中立値)は `docs/ai/skills/dlisp-codegen-aot-jit/references/parity-map.md` の stdlib 節を参照。
- docs / public surface: 先に code と test を確認し、`FUNCTIONS.md` / `spec.md` / `README.md` の責務に合わせて編集する。必要なら `docs/ai/skills/dlisp-language-surface/`。

## 主要クレート
- `cli`: CLI、REPL、ファイル実行、AOT compile エントリポイント。起点は `cli/src/main.rs`。
- `core`: evaluator、special forms、builtins、parser、JIT/AOT codegen。
- `runtime`: AOT/JIT 実行時の FFI、GC、OS/IO 補助。
- `stdlib`: `(require "core")` で読む Lisp 実装の標準ライブラリ(`stdlib/src/core.lisp`)。interpreter-only builtin の移管先。

## 生成物とチェック
- `docs/ai/generated/`(`surface.md`, `symbol-index.md`)は生成物。編集しない。言語 surface を変えたら `scripts/check.sh --regenerate` で再生成する。
- symbol 一覧や定義位置を知りたいときは registry を開く前に `docs/ai/generated/surface.md` / `symbol-index.md` を見る。
- commit 前の高速ゲートは `scripts/check.sh`(fmt / clippy / surface check / codegen parity / 生成物鮮度)。テストは含まない。

## 主要入口
- 評価とデフォルト環境: `core/src/interpreter.rs`
- 特殊形式の登録: `core/src/forms/registry.rs`
- 組み込み関数の登録: `core/src/builtins/mod.rs`
- AOT compile: `core/src/compiler.rs`, `cli/src/compile.rs`
- REPL: `cli/src/repl.rs`
- runtime / GC / FFI: `runtime/src/lib.rs`(re-export hub), `runtime/build.rs`

## 実装ルール
- 二重経路の原則: interpreter の特殊 form・評価挙動を変えたら、JIT/AOT codegen 側(`core/src/codegen/`, `core/src/compiler.rs`)の対応有無を必ず確認する。意図しない interpreter fallback を残さない。
- 言語 surface の変更は `registry.rs` / `builtins/mod.rs` が source of truth。`FUNCTIONS.md` / `spec.md` を更新するときは登録名と突き合わせて drift がないか確認する。
- runtime の FFI 関数は機能別モジュール(`runtime/src/{gc,constructors,arith,cmp,...}.rs`)に追加し、`lib.rs` の re-export で crate-root パスを維持する。シンボル名は変更しない。
- 挙動確認は `cargo run -- example-lisp/<対象>.lisp` など最小例で先に行い、その後にテストを書く。

## 検証の最小単位
- インタプリタ評価系: `cargo test -p dlisp-core --test interpreter_tests --quiet`
- JIT / codegen 系: `cargo test -p dlisp-core --test jit_phase3_tests --quiet`
- CLI / example / AOT 系: `cargo test -p dlisp --test integration_tests --quiet`
- runtime crate 単体: `cargo test -p dlisp_runtime --quiet`
- 複数クレートを跨いだ変更だけ: `cargo test --workspace --quiet`
- 1 件だけ絞り込む: `cargo test -p dlisp-core --test <target> -- <filter>`(例: `-- test_match_guards`)

## Skill 運用
- canonical source は `docs/ai/skills/` に置く。
- repo 内の `.claude/skills/` は `docs/ai/skills/` への symlink(opencode / Claude Code が自動検出)。symlink が展開されていたら `scripts/install-runtime-skills.sh repo-links` で再作成する。
- Codex runtime 配置先は `~/.codex/skills/`。opencode / Claude Code の global 配置は `scripts/install-runtime-skills.sh [opencode|claude]` を使う。
- repo 調査とコード変更では `docs/ai/skills/dlisp-repo/` を使う。
- 言語仕様と関数一覧の更新では `docs/ai/skills/dlisp-language-surface/` を使う。
- 領域が明確な実装では、上記タスク別 skill を優先して読む。

## テスト要件
- コード変更後は interpreter path と JIT / compile path の両方を確認する。
- 最終確認では `cargo test --workspace --quiet` を通す。
