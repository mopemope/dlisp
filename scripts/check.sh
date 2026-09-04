#!/usr/bin/env bash
# Fast static gate for humans and AI agents. Run before committing.
# Tests are NOT run here; pick a scoped cargo test command per AGENTS.md.
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

if [[ "${1:-}" == "--regenerate" ]]; then
  python3 docs/ai/skills/dlisp-language-surface/scripts/extract_surface.py snapshot \
    > docs/ai/generated/surface.md
  python3 scripts/gen_symbol_index.py --write
  exit 0
fi

status=0
step() { echo; echo "== $*"; }

step "cargo fmt --check"
cargo fmt --check || status=1

step "cargo clippy (-D warnings, workspace lints)"
cargo clippy --workspace --all-targets --quiet -- -D warnings || status=1

SURFACE_SCRIPT="docs/ai/skills/dlisp-language-surface/scripts/extract_surface.py"

step "surface docs vs registries"
python3 "$SURFACE_SCRIPT" check || status=1

step "codegen parity (COMPILED_BUILTINS / lowering / FFI)"
python3 scripts/check_codegen_parity.py || status=1

step "doc refs (AGENTS.md / docs/ai paths and test targets)"
python3 scripts/check_doc_refs.py || status=1

step "generated docs freshness"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
python3 "$SURFACE_SCRIPT" snapshot > "$tmp/surface.md"
python3 scripts/gen_symbol_index.py > "$tmp/symbol-index.md"
for name in surface.md symbol-index.md; do
  if [[ ! -f "docs/ai/generated/$name" ]]; then
    echo "missing docs/ai/generated/$name (run: $0 --regenerate)"
    status=1
  elif ! diff -u "docs/ai/generated/$name" "$tmp/$name"; then
    echo "docs/ai/generated/$name is stale (run: $0 --regenerate)"
    status=1
  fi
done

echo
if [[ "$status" -eq 0 ]]; then
  echo "all checks passed"
else
  echo "checks FAILED"
fi
exit "$status"
