#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../../../.." && pwd)"
FORMS_FILE="$ROOT_DIR/core/src/forms/registry.rs"
BUILTINS_FILE="$ROOT_DIR/core/src/builtins/mod.rs"

if [[ ! -f "$FORMS_FILE" || ! -f "$BUILTINS_FILE" ]]; then
  echo "failed to locate dlisp source files" >&2
  exit 1
fi

echo "# dlisp surface snapshot"
echo
echo "## Special forms"
sed -n 's/.*reg.register("\([^"]*\)".*/- `\1`/p' "$FORMS_FILE" | sort -u
echo
echo "## Builtins"
sed -n 's/.*env.set("\([^"]*\)".*/- `\1`/p' "$BUILTINS_FILE" | sort -u
