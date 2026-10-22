#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-repo-links}"
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
  retire-codex)
    # Preserve legacy copies outside the skill discovery directory. This is
    # deliberately explicit: repo-links never mutates a user's global setup.
    if [[ $# -gt 2 || ( $# -eq 2 && "$2" != "--plan" ) ]]; then
      echo "usage: $0 retire-codex [--plan]" >&2
      exit 1
    fi
    LEGACY_HOME="${CODEX_HOME:-$HOME/.codex}"
    legacy_skills=()
    for skill_dir in "$SOURCE_DIR"/*; do
      [[ -f "$skill_dir/SKILL.md" ]] || continue
      skill_name="$(basename "$skill_dir")"
      local_link="$ROOT_DIR/.agents/skills/$skill_name"
      if [[ ! -L "$local_link" || "$(readlink -f "$local_link")" != "$(readlink -f "$skill_dir")" ]]; then
        echo "create checkout-local links before retiring global skills: $local_link" >&2
        exit 1
      fi
      target="$LEGACY_HOME/skills/$skill_name"
      if [[ -e "$target" || -L "$target" ]]; then
        legacy_skills+=("$skill_name")
      fi
    done
    if [[ ${#legacy_skills[@]} -eq 0 ]]; then
      echo "no legacy Codex skills to retire"
      exit 0
    fi
    if [[ "${2:-}" == "--plan" ]]; then
      for skill_name in "${legacy_skills[@]}"; do
        echo "move $LEGACY_HOME/skills/$skill_name to $LEGACY_HOME/skill-backups/dlisp.<unique>/$skill_name"
      done
      exit 0
    fi
    mkdir -p "$LEGACY_HOME/skill-backups"
    backup_dir="$(mktemp -d "$LEGACY_HOME/skill-backups/dlisp.XXXXXXXX")"
    for skill_name in "${legacy_skills[@]}"; do
      mv -- "$LEGACY_HOME/skills/$skill_name" "$backup_dir/$skill_name"
    done
    echo "retired ${#legacy_skills[@]} legacy skills; backup: $backup_dir"
    exit 0
    ;;
  repo-links)
    # Keep each skill linked to this checkout; never replace real directories.
    for host in .agents .claude; do
      mkdir -p "$ROOT_DIR/$host/skills"
      for skill_dir in "$SOURCE_DIR"/*; do
        [[ -d "$skill_dir" ]] || continue
        skill_name="$(basename "$skill_dir")"
        target="$ROOT_DIR/$host/skills/$skill_name"
        if [[ -e "$target" && ! -L "$target" ]]; then
          echo "refusing to replace non-symlink: $target" >&2
          exit 1
        fi
        ln -sfn "../../docs/ai/skills/$skill_name" "$target"
      done
    done
    echo "created repo links in .agents/skills and .claude/skills"
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
    echo "usage: scripts/install-runtime-skills.sh [codex|opencode|claude|repo-links|list|retire-codex [--plan]]" >&2
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
