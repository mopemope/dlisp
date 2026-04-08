# AI / Skill 運用メモ

このディレクトリは、このリポジトリでの AI 利用時の token 消費を減らすための運用情報をまとめる。

## 目的
- 常時読む文書を短くする。
- 詳細は必要時だけ読む。
- repo 固有知識を Skill と reference に分離する。

## 配置
- canonical Skill source: `docs/ai/skills/`
- Codex runtime skills: `~/.codex/skills/`

## 使い分け
- `AGENTS.md`: この repo で最初に守る短いルールだけを書く。
- `SKILL.md`: 別エージェントが作業を始めるための最短手順だけを書く。
- `references/`: 長い説明、モジュール一覧、チェックリストを置く。
- `scripts/`: 毎回書き直したくない決定的な補助処理を置く。

## 収録 Skill
- `dlisp-repo`: クレートやモジュールの当たりを最短で付ける。
- `dlisp-language-surface`: 特殊形式、組み込み関数、仕様文書を更新するときの source of truth を案内する。

## 導入
- sample Skill の runtime 配置には `scripts/install-runtime-skills.sh` を使う。

```bash
scripts/install-runtime-skills.sh
```

## authoring ルール
- trigger 条件は frontmatter の `description` に集約する。
- `SKILL.md` 本文には長い「when to use」を書かない。
- 詳細は `references/` に逃がす。
- shell / Rust / reference で済むなら、新しい長文ドキュメントを増やさない。
