# AI / Skill 運用メモ

このディレクトリは、このリポジトリでの AI 利用時の token 消費を減らすための運用情報をまとめる。

## 目的
- 常時読む文書を短くする。
- 詳細は必要時だけ読む。
- repo 固有知識を Skill と reference に分離する。

## 配置
- canonical Skill source: `docs/ai/skills/`
- opencode / Claude Code: `.claude/skills/` に `docs/ai/skills/` への symlink をコミット。repo checkout 時点で自動検出される(opencode は `.claude/skills/` も探索する)
- Codex runtime skills: `~/.codex/skills/`(`scripts/install-runtime-skills.sh` で展開)
- opencode / Claude Code の global 配置: `scripts/install-runtime-skills.sh [opencode|claude]`
- repo 内 symlink の(再)作成: `scripts/install-runtime-skills.sh repo-links`
- Windows などで symlink が実体化・欠落した場合は `scripts/install-runtime-skills.sh repo-links` で再展開する

## ドキュメント整合性
- `AGENTS.md` と `docs/ai/**/*.md` 内のファイルパス・`--test` ターゲット・`-p` クレート名は `scripts/check_doc_refs.py` が機械検査する(`scripts/check.sh` に組み込み済み)。doc を編集したら壊れた参照を残さないこと。

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

## 導入
- Skill の runtime 配置には `scripts/install-runtime-skills.sh` を使う。

```bash
scripts/install-runtime-skills.sh
```

## authoring ルール
- trigger 条件は frontmatter の `description` に集約する。
- `SKILL.md` 本文には長い「when to use」を書かない。
- 詳細は `references/` に逃がす。
- shell / Python / Rust / reference で済むなら、新しい長文ドキュメントを増やさない。
