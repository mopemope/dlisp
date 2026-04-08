# Test Scope

- `cargo test -p dlisp-core --test interpreter_tests --quiet`: evaluator と基本 special forms
- `cargo test -p dlisp-core --test jit_phase3_tests --quiet`: JIT/codegen surface
- `cargo test -p dlisp --test integration_tests --quiet`: CLI、examples、AOT 導線
- `cargo test -p dlisp_runtime --quiet`: runtime crate の単体変更
- `cargo test --workspace --quiet`: 跨 crate 変更または最終確認

最初から workspace 全体を回さない。変更範囲に合う最小コマンドから始める。
