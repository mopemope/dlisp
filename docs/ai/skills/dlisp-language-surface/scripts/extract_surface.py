#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parents[5]


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        raise SystemExit(f"failed to locate {path}") from None


def extract_special_forms(root: Path) -> set[str]:
    registry = read_text(root / "core/src/forms/registry.rs")
    names = set(re.findall(r'\breg\.register\(\s*"([^"]+)"', registry))

    interpreter = read_text(root / "core/src/interpreter.rs")
    if re.search(r'\bs\s*==\s*"defmacro"', interpreter):
        names.add("defmacro")

    return names


def extract_builtins(root: Path) -> set[str]:
    builtins = read_text(root / "core/src/builtins/mod.rs")
    return set(
        re.findall(
            r'\benv\.set\(\s*"([^"]+)"\.to_string\(\)',
            builtins,
            flags=re.DOTALL,
        )
    )


def markdown_section(text: str, heading: str) -> str:
    start_match = re.search(rf"^## {re.escape(heading)}\s*$", text, flags=re.MULTILINE)
    if not start_match:
        return ""

    start = start_match.end()
    next_match = re.search(r"^##\s+", text[start:], flags=re.MULTILINE)
    end = start + next_match.start() if next_match else len(text)
    return text[start:end]


def extract_doc_names(root: Path, heading: str) -> set[str]:
    functions_md = read_text(root / "FUNCTIONS.md")
    section = markdown_section(functions_md, heading)
    return set(re.findall(r"`([^`\n]+)`", section))


def print_snapshot(root: Path) -> None:
    print("# dlisp surface snapshot")
    print()
    print("## Special forms")
    for name in sorted(extract_special_forms(root)):
        print(f"- `{name}`")
    print()
    print("## Builtins")
    for name in sorted(extract_builtins(root)):
        print(f"- `{name}`")


def report_delta(title: str, names: set[str]) -> None:
    if not names:
        return
    print(title)
    for name in sorted(names):
        print(f"- `{name}`")


def check_surface(root: Path) -> int:
    actual_forms = extract_special_forms(root)
    actual_builtins = extract_builtins(root)
    doc_forms = extract_doc_names(root, "Special forms")
    doc_builtins = extract_doc_names(root, "Builtins")

    form_missing = actual_forms - doc_forms
    form_stale = doc_forms - actual_forms
    builtin_missing = actual_builtins - doc_builtins
    builtin_stale = doc_builtins - actual_builtins

    if not any([form_missing, form_stale, builtin_missing, builtin_stale]):
        print("surface check passed")
        return 0

    report_delta("Special forms missing from FUNCTIONS.md:", form_missing)
    report_delta("Special forms stale in FUNCTIONS.md:", form_stale)
    report_delta("Builtins missing from FUNCTIONS.md:", builtin_missing)
    report_delta("Builtins stale in FUNCTIONS.md:", builtin_stale)
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Extract or verify dlisp language surface.")
    parser.add_argument(
        "mode",
        nargs="?",
        choices=("snapshot", "check"),
        default="snapshot",
        help="snapshot prints extracted forms/builtins; check compares FUNCTIONS.md",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="compare FUNCTIONS.md with extracted surface",
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=ROOT_DIR,
        help="repository root (defaults to the script-relative dlisp root)",
    )
    args = parser.parse_args()

    root = args.root.resolve()
    if args.check or args.mode == "check":
        return check_surface(root)

    print_snapshot(root)
    return 0


if __name__ == "__main__":
    sys.exit(main())
