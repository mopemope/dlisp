# Agent Guide

このリポジトリでは、token 消費を抑えるために「最小探索・最小検証」を徹底すること。

## 基本方針
- チャットは日本語で行う。
- Python 実行は禁止。補助スクリプトは shell を使う。
- まず `rg --files` / `rg -n` で当たりを付け、必要なファイルだけ読む。
- `README.md` 全文を最初から読まない。ユーザー向け挙動、公開文書、導入手順を触るときだけ必要箇所を開く。
- 言語仕様や関数一覧を直すときも、最初に prose docs を読むのではなく `core/src/forms/registry.rs` と `core/src/builtins/mod.rs` を見る。
- 変更後は関係する最小コマンドで検証し、ワークスペース全体テストは最後に限定する。

## 探索順
1. `Cargo.toml` で workspace 境界を確認する。
2. `rg -n "<symbol>|<feature>" cli core runtime stdlib` で実装位置を絞る。
3. repo 固有の案内が必要なら `docs/ai/skills/` 以下を読む。
4. 文書がコードと食い違う場合は、コードとテストを優先する。

## 主要クレート
- `cli`: CLI、REPL、ファイル実行、AOT compile エントリポイント。起点は `cli/src/main.rs`。
- `core`: evaluator、special forms、builtins、parser、JIT/AOT codegen。
- `runtime`: AOT/JIT 実行時の FFI、GC、OS/IO 補助。
- `stdlib`: 現状は最小のプレースホルダ crate。

## 主要入口
- 評価とデフォルト環境: `core/src/interpreter.rs`
- 特殊形式の登録: `core/src/forms/registry.rs`
- 組み込み関数の登録: `core/src/builtins/mod.rs`
- AOT compile: `core/src/compiler.rs`, `cli/src/compile.rs`
- REPL: `cli/src/repl.rs`
- runtime / GC / FFI: `runtime/src/lib.rs`, `runtime/build.rs`

## 検証の最小単位
- インタプリタ評価系: `cargo test -p dlisp-core --test interpreter_tests --quiet`
- JIT / codegen 系: `cargo test -p dlisp-core --test jit_phase3_tests --quiet`
- CLI / example / AOT 系: `cargo test -p dlisp --test integration_tests --quiet`
- runtime crate 単体: `cargo test -p dlisp_runtime --quiet`
- 複数クレートを跨いだ変更だけ: `cargo test --workspace --quiet`

## Skill 運用
- canonical source は `docs/ai/skills/` に置く。
- Codex runtime 配置先は `~/.codex/skills/`。
- 導入や更新は `scripts/install-runtime-skills.sh` を使う。
- repo 調査とコード変更では `docs/ai/skills/dlisp-repo/` を使う。
- 言語仕様と関数一覧の更新では `docs/ai/skills/dlisp-language-surface/` を使う。

## テスト要件
- コード変更後は interpreter path と JIT / compile path の両方を確認する。
- 最終確認では `cargo test --workspace --quiet` を通す。
