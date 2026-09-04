#!/usr/bin/env python3
"""Statically verify file paths and test targets referenced in agent docs.

Checks AGENTS.md and docs/ai/**/*.md so that agents following the docs do not
chase broken paths or run nonexistent cargo test targets. Keep allowlists
minimal; fix the docs instead of growing them.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[1]

DOC_FILES = ["AGENTS.md", *[str(p) for p in (ROOT_DIR / "docs/ai").rglob("*.md")]]
DOC_FILES = sorted(set(DOC_FILES))

# Tokens that look like paths but are not repo files.
PATH_ALLOWLIST = {
    "<name>",
    "<対象>.lisp",
}

# --test target names that are generated or renamed at build time.
TEST_ALLOWLIST = set()

BACKTICK_RE = re.compile(r"`([^`\n]+)`")
TEST_RE = re.compile(r"--test\s+([a-z0-9_]+)")


def repo_files() -> set[str]:
    files: set[str] = set()
    for base in ("cli", "core", "runtime", "stdlib", "docs", "scripts", "example-lisp", "examples"):
        base_path = ROOT_DIR / base
        if base_path.is_dir():
            files.update(str(p.relative_to(ROOT_DIR)) for p in base_path.rglob("*"))
    for extra in ("Cargo.toml", "Cargo.lock", "README.md", "AGENTS.md", "spec.md", "FUNCTIONS.md"):
        if (ROOT_DIR / extra).exists():
            files.add(extra)
    return files


def extract_doc_paths(text: str) -> set[str]:
    paths: set[str] = set()
    for token in BACKTICK_RE.findall(text):
        if token in PATH_ALLOWLIST:
            continue
        if token.startswith("--"):
            continue
        token = token.rstrip("/")
        if "/" not in token:
            continue
        # Strip trailing wildcard segments (e.g. `core/tests/*parser*`).
        if "*" in Path(token).name:
            token = str(Path(token).parent)
        if not token or "/" not in token:
            continue
        if re.match(r"^[\w./-]+$", token) and not token.startswith(("http", "~")):
            paths.add(token)
    # Drop bare relative names that appear alongside a rooted path in the
    # same sentence, e.g. "`registry.rs` / `builtins/mod.rs`" next to
    # `core/src/...` references. Only keep repo-rooted candidates.
    return {p for p in paths if not _is_likely_relative(p, paths)}


def _is_likely_relative(path: str, all_paths: set[str]) -> bool:
    if path.startswith(("cli/", "core/", "runtime/", "stdlib/", "docs/", "scripts/", "example-lisp/", "examples/")):
        return False
    # A bare `<a>/<b>.rs`-style token is kept only if some rooted doc path
    # ends with it, proving it is a shorthand reference.
    return any(other.endswith("/" + path) for other in all_paths)


def extract_doc_tests(text: str) -> set[str]:
    return set(TEST_RE.findall(text))


def test_targets() -> set[str]:
    targets: set[str] = set()
    for base in ("core/tests", "cli/tests", "runtime/tests"):
        base_path = ROOT_DIR / base
        if base_path.is_dir():
            targets.update(p.stem for p in base_path.rglob("*.rs"))
    # Lib-internal test crates: `-p dlisp_runtime` style references remain
    # unverified; only explicit --test names are checked.
    return targets


def crate_names() -> set[str]:
    names: set[str] = set()
    for manifest in ROOT_DIR.rglob("Cargo.toml"):
        try:
            text = manifest.read_text(encoding="utf-8")
        except OSError:
            continue
        match = re.search(r'^name\s*=\s*"([^"]+)"', text, flags=re.MULTILINE)
        if match:
            names.add(match.group(1))
    return names


def main() -> int:
    files = repo_files()
    targets = test_targets()
    crates = crate_names()

    broken_paths: dict[str, list[str]] = {}
    broken_tests: dict[str, list[str]] = {}
    broken_crates: dict[str, list[str]] = {}

    for doc in DOC_FILES:
        path = ROOT_DIR / doc
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")

        for ref in sorted(extract_doc_paths(text) - files):
            if (ROOT_DIR / ref).is_dir() or (ROOT_DIR / ref).is_file():
                continue
            broken_paths.setdefault(doc, []).append(ref)

        for test in sorted(extract_doc_tests(text) - targets - TEST_ALLOWLIST):
            broken_tests.setdefault(doc, []).append(test)

        for crate in sorted(
            set(re.findall(r"-p\s+(dlisp[\w-]*)", text)) - crates
        ):
            broken_crates.setdefault(doc, []).append(crate)

    status = 0
    for title, broken in (
        ("Broken file paths:", broken_paths),
        ("Broken --test targets:", broken_tests),
        ("Unknown crates in -p flags:", broken_crates),
    ):
        for doc, items in sorted(broken.items()):
            print(f"{title} {doc}")
            for item in items:
                print(f"  - {item}")
            status = 1

    if status == 0:
        print("doc refs check passed")
    return status


if __name__ == "__main__":
    sys.exit(main())
