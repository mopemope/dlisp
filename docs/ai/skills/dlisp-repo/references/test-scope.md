# Test Scope

最初から workspace 全体を回さない。変更範囲に合う最小コマンドから始める。

## 変更領域 → 最小テスト対応表

| 変更対象 | テストファイル | コマンド |
|---|---|---|
| evaluator / 基本special forms | `core/tests/interpreter_tests.rs` | `cargo test -p dlisp-core --test interpreter_tests --quiet` |
| special forms 追加・変更 | `core/tests/special_forms.rs` | `cargo test -p dlisp-core --test special_forms --quiet` |
| binding / scope | `defvar_tests`, `setq_tests`, `let_star_tests`, `destructure_tests` | 各 `--test <名前>` |
| macro / backquote | `macros.rs`, `macro_utils_tests.rs`, `backquote_tests.rs` | 各 `--test <名前>` |
| 基本 builtin surface | `phase1_tests`, `phase2_tests`, `review_tests` | 各 `--test <名前>` |
| stdlib (`require "core"`) | `core/tests/stdlib_tests.rs`, `loop_recur_tests.rs` | 各 `--test <名前>` |
| map / vector / format / io / sys / os builtin | `map_tests`, `vector_tests`, `format_tests`, `io_tests`, `sys_tests`, `os_tests` | 対象の1つ |
| JIT / codegen surface | `core/tests/jit_phase3_tests.rs`(他に `jit_phase2_tests`) | `cargo test -p dlisp-core --test jit_phase3_tests --quiet` |
| AOT / compiler | `compiler_tests`, `compiler_integration_tests` | 各 `--test <名前>` |
| interpreter ↔ AOT parity | `cli/tests/parity_golden_tests.rs` | `cargo test -p dlisp --test parity_golden_tests --quiet` |
| parser / syntax | `core/tests/parser_comments_tests.rs`, `backquote_tests` | `cargo test -p dlisp-core --test parser_comments_tests --quiet` |
| runtime crate 単体 | `runtime/src/verify_tests.rs`(lib内) | `cargo test -p dlisp_runtime --quiet` |
| CLI / examples / AOT 導線 | `cli/tests/integration_tests.rs` | `cargo test -p dlisp --test integration_tests --quiet` |
| 跨 crate 変更・最終確認 | 全部 | `cargo test --workspace --quiet` |

## 選択ルール
- 変更ファイルからの候補は `bash scripts/agent-check.sh --plan`。`--focused` は候補を実行、`--final` はコード変更に全体ゲートも追加する。`--base <revision>` で比較基準を指定でき、staged / unstaged / untracked を含む。
- この選択は保守的な補助であり、意味上の依存は変更者が追加する。不明な領域は全体検証へ寄せる。
- interpreter 経路の変更 → 対応する `--test` 1本から始め、通ったら JIT 系を確認。
- JIT/AOT 経路に影響しそうな言語surface変更 → `jit_phase3_tests` と `integration_tests` を追加。
- runtime FFI の変更 → `-p dlisp_runtime` の後に `integration_tests`(リンク確認)。
- コード変更の最終確認は workspace 全体。文書・Skill のみは文書とリンク検証、補助スクリプト変更は Python 回帰テストも実行する。AGENTS.md を完了条件の唯一の定義とする。
