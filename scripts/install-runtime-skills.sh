#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-codex}"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SOURCE_DIR="$ROOT_DIR/docs/ai/skills"

case "$MODE" in
  codex)
    DEST_DIR="${CODEX_HOME:-$HOME/.codex}/skills"
    ;;
  list)
    find "$SOURCE_DIR" -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | sort
    exit 0
    ;;
  *)
    echo "usage: scripts/install-runtime-skills.sh [codex|list]" >&2
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
