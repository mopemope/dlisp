# AI / Skill 運用メモ

このディレクトリは、このリポジトリでの AI 利用時の token 消費を減らすための運用情報をまとめる。

## 目的
- 常時読む文書を短くする。
- 詳細は必要時だけ読む。
- repo 固有知識を Skill と reference に分離する。

## 配置
- canonical Skill source: `docs/ai/skills/`
- opencode / Claude Code: `.claude/skills/` に `docs/ai/skills/` への symlink をコミット。repo checkout 時点で自動検出される(opencode は `.claude/skills/` も探索する)
- Codex: `.agents/skills/` の相対 symlink から checkout と同じ版を自動検出する。
- 従来の Codex global 配置は `scripts/install-runtime-skills.sh codex`。repo-local 配置と同名で重複させない。古いコピーの内容比較は `python3 scripts/check_agent_skills.py --installed <配置先>`。global ファイルは自動削除しない。
- opencode / Claude Code の global 配置: `scripts/install-runtime-skills.sh [opencode|claude]`
- repo 内 symlink の(再)作成: `scripts/install-runtime-skills.sh repo-links`
- `repo-links` は `.agents/skills/` と `.claude/skills/` の両方を更新する。実ディレクトリや通常ファイルがある場合は上書きせず停止する。Windows などで symlink が実体化していたら内容を確認・退避してからリンクを作り直す。
- 旧 Codex global copy の整理: `bash scripts/install-runtime-skills.sh retire-codex --plan` で対象を確認し、`--plan` を外して実行する。repo-local link を検証したうえで、この repo と同名の Skill だけを Codex home の skill-backups 以下へ退避する。内容は削除しない。復元は各退避ディレクトリを元の skills 配下へ戻す（同名の既存配置がある場合は先に内容を確認する）。

## ドキュメント整合性
- `AGENTS.md` と `docs/ai/**/*.md` 内のファイルパス・`--test` ターゲット・`-p` クレート名は `scripts/check_doc_refs.py` が機械検査する(`scripts/check.sh` に組み込み済み)。doc を編集したら壊れた参照を残さないこと。
- `scripts/check_agent_skills.py` は Skill の名前・description と checkout-local link を検査する。`scripts/check_runtime_abi.py` は runtime extern と Cranelift import の引数・戻り値の ABI 幅を照合する。所有権や struct layout は対象外。

## 使い分け
- `AGENTS.md`: この repo で最初に守る短いルールだけを書く。
- `SKILL.md`: 別エージェントが作業を始めるための最短手順だけを書く。
- `references/`: 長い説明、モジュール一覧、チェックリストを置く。
- `scripts/`: 毎回書き直したくない決定的な補助処理を置く。

## 収録 Skill
- `dlisp-repo`: クレートやモジュールの当たりを最短で付ける。
- `dlisp-language-surface`: 特殊形式、組み込み関数、仕様文書を更新するときの source of truth を案内する。
- `dlisp-parser-syntax`: parser、AST、reader syntax の変更。
- `dlisp-evaluator-forms`: evaluator、special forms、environment の変更。
- `dlisp-builtins-surface`: builtin 実装、登録、surface docs の変更。
- `dlisp-codegen-aot-jit`: JIT/AOT/codegen と runtime ABI 接続の変更。
- `dlisp-runtime-ffi`: runtime value ABI、GC、FFI export、OS/IO runtime の変更。
- `dlisp-stdlib`: Lisp 標準ライブラリの実装と require。
- `dlisp-cli`: CLI、REPL、診断と終了コード。
- `dlisp-parity-debugging`: 実行経路差と意図しない JIT fallback の調査。

## 導入
- Skill の runtime 配置には `scripts/install-runtime-skills.sh` を使う。

```bash
bash scripts/install-runtime-skills.sh repo-links
```

## authoring ルール
- trigger 条件は frontmatter の `description` に集約する。
- `SKILL.md` 本文には長い「when to use」を書かない。
- 詳細は `references/` に逃がす。
- shell / Python / Rust / reference で済むなら、新しい長文ドキュメントを増やさない。

## 検証と評価
- 対象検証の候補: `bash scripts/agent-check.sh --plan`。最終要件は AGENTS.md に集約する。
- 補助ツールのテスト: `python3 -m unittest discover -s scripts/tests -q`。モデル呼び出しは行わない。
- エージェントの比較評価: [evals/README.md](evals/README.md)。routing と実装修正の成績を区別し、成功率と使用量を同じ条件で比較する。
