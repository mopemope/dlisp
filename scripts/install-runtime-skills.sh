#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-codex}"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SOURCE_DIR="$ROOT_DIR/docs/ai/skills"

case "$MODE" in
  codex)
    DEST_DIR="${CODEX_HOME:-$HOME/.codex}/skills"
    ;;
  opencode)
    DEST_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/opencode/skills"
    ;;
  claude)
    DEST_DIR="${CLAUDE_CONFIG_DIR:-${CLAUDE_HOME:-$HOME/.claude}}/skills"
    ;;
  repo-links)
    # (Re)create project-local symlinks in .claude/skills/ -> docs/ai/skills/
    # so opencode / Claude Code discover them. Canonical source stays in
    # docs/ai/skills/.
    mkdir -p "$ROOT_DIR/.claude/skills"
    for skill_dir in "$SOURCE_DIR"/*; do
      [[ -d "$skill_dir" ]] || continue
      skill_name="$(basename "$skill_dir")"
      ln -sfn "../../docs/ai/skills/$skill_name" "$ROOT_DIR/.claude/skills/$skill_name"
    done
    echo "created repo links in $ROOT_DIR/.claude/skills"
    exit 0
    ;;
  list)
    for skill_dir in "$SOURCE_DIR"/*; do
      [[ -d "$skill_dir" ]] || continue
      basename "$skill_dir"
    done | sort
    exit 0
    ;;
  *)
    echo "usage: scripts/install-runtime-skills.sh [codex|opencode|claude|repo-links|list]" >&2
    exit 1
    ;;
esac

install -d "$DEST_DIR"

for skill_dir in "$SOURCE_DIR"/*; do
  [[ -d "$skill_dir" ]] || continue
  skill_name="$(basename "$skill_dir")"
  target="$DEST_DIR/$skill_name"
  rm -rf "$target"
  cp -R "$skill_dir" "$target"
done

echo "installed skills to $DEST_DIR"
