# Agent Guide

このリポジトリでは、正確性を保ちながら探索・検証・指示の重複を減らす。チャットは日本語で行う。

## 探索と判断
- `rg --files` / `rg -n` で対象を絞り、必要なファイルだけ読む。README 全文や全 Skill を事前に読まない。
- クレート境界が不明なときだけ `Cargo.toml`、担当領域が不明なときだけ `dlisp-repo` の module-map を読む。
- シンボル名・定義位置の検索には `docs/ai/generated/surface.md` / `symbol-index.md` を利用する。挙動は実装とテストで確認する。
- 特殊形式の登録は `core/src/forms/registry.rs`、builtin の登録は `core/src/builtins/mod.rs` が source of truth。文書が食い違えばコードとテストを優先し、古い文書も直す。
- 単純な探索は shell、繰り返す検証や正確な抽出は Python を使ってよい。

## タスクの入口
対象が明確なら該当 Skill から始める。複数領域の Skill は実際にその領域を変更・調査するときだけ追加する。

| 対象 | Skill | 主な入口 |
|---|---|---|
| 担当箇所が不明 | `dlisp-repo` | `Cargo.toml`、module-map |
| parser / syntax | `dlisp-parser-syntax` | `core/src/parser.rs`、`core/src/ast.rs` |
| evaluator / special forms | `dlisp-evaluator-forms` | `core/src/interpreter.rs`、`core/src/forms/` |
| builtin 実装・登録 | `dlisp-builtins-surface` | `core/src/builtins/` |
| JIT / AOT / codegen | `dlisp-codegen-aot-jit` | `core/src/codegen/`、`core/src/compiler.rs` |
| runtime / FFI / GC | `dlisp-runtime-ffi` | `runtime/src/value.rs`、対象 runtime module |
| Lisp 標準ライブラリ | `dlisp-stdlib` | `stdlib/src/core.lisp` |
| CLI / REPL / 終了コード | `dlisp-cli` | `cli/src/main.rs`、`cli/src/repl.rs` |
| 実行経路間の不一致 | `dlisp-parity-debugging` | `core/tests/jit_parity_tests.rs` |
| 公開仕様・関数一覧 | `dlisp-language-surface` | 登録元と対象の公開文書 |

## 実装の不変条件
- evaluator・特殊形式・言語挙動を変えたら interpreter / JIT / AOT の影響を確認する。意図しない fallback を残さず、JIT を期待するテストでは `jit_code.is_some()` を確認する。
- runtime FFI は機能別モジュールに実装し、`runtime/src/lib.rs` の re-export で crate-root パスを維持する。シンボル・引数・戻り値を codegen と揃える。
- 不具合修正は最小再現を確認し、修正前に失敗する回帰テストを追加する。既存の経路差は parity-map と対応テストを確認して扱う。
- 公開名・仕様を変えたら `FUNCTIONS.md` / `spec.md` の担当箇所も更新する。導入・CLI 利用方法は `README.md` が担当する。

## 検証と完了条件
- 作業中は対象の最小テストから始める。候補は `bash scripts/agent-check.sh --plan`、詳細は `docs/ai/skills/dlisp-repo/references/test-scope.md`。
- 言語・runtime・CLI のコード変更後は interpreter と compiled path を確認する。コンパイル成功だけで値・副作用の正しさを判断しない。
- 言語・runtime・CLI のコード変更の最終確認は `bash scripts/check.sh` と `cargo test --workspace --quiet`。静的ゲート自体には Rust テストを含まない。
- 文書・Skill のみなら `bash scripts/agent-check.sh --focused` で文書・配置チェックを行う。補助スクリプトの変更は `python3 -m unittest discover -s scripts/tests -q` も実行する。
- 実行した検証と未検証事項を区別して報告する。無関係な既存エラーがあれば根拠と影響を残す。

## 生成物と Skills
- `docs/ai/generated/` は直接編集せず `bash scripts/check.sh --regenerate` で再生成する。
- Skill の canonical source は `docs/ai/skills/`。`.agents/skills/` と `.claude/skills/` はそこへの相対 symlink とし、`bash scripts/install-runtime-skills.sh repo-links` で再作成する。
- repo 固有 Skill は checkout-local 配置を優先する。古い global copy は自動削除せず、重複・同期漏れは `python3 scripts/check_agent_skills.py --installed <配置先>` で調べる。
- Skill は狭い適用条件と判断基準を本文に、長い詳細を references に置く。追加・変更時は `docs/ai/evals/README.md` の評価ケースも確認する。
