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
| map / vector / format / io / sys / os builtin | `map_tests`, `vector_tests`, `format_tests`, `io_tests`, `sys_tests`, `os_tests` | 対象の1つ |
| JIT / codegen surface | `core/tests/jit_phase3_tests.rs`(他に `jit_phase2_tests`) | `cargo test -p dlisp-core --test jit_phase3_tests --quiet` |
| AOT / compiler | `compiler_tests`, `compiler_integration_tests` | 各 `--test <名前>` |
| parser / syntax | `core/tests/parser_comments_tests.rs`, `backquote_tests` | `cargo test -p dlisp-core --test parser_comments_tests --quiet` |
| runtime crate 単体 | `runtime/src/verify_tests.rs`(lib内) | `cargo test -p dlisp_runtime --quiet` |
| CLI / examples / AOT 導線 | `cli/tests/integration_tests.rs` | `cargo test -p dlisp --test integration_tests --quiet` |
| 跨 crate 変更・最終確認 | 全部 | `cargo test --workspace --quiet` |

## 選択ルール
- interpreter 経路の変更 → 対応する `--test` 1本から始め、通ったら JIT 系を確認。
- JIT/AOT 経路に影響しそうな言語surface変更 → `jit_phase3_tests` と `integration_tests` を追加。
- runtime FFI の変更 → `-p dlisp_runtime` の後に `integration_tests`(リンク確認)。
- 最終確認のみ workspace 全体。AGENTS.md の最終ルールに従う。
