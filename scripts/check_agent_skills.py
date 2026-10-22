#!/usr/bin/env python3
"""Validate canonical skill metadata and checkout-local discovery links."""
import argparse
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]


def check(root, installed=None):
    failures = []
    names = set()
    source = root / "docs/ai/skills"
    for folder in sorted(source.iterdir()):
        if not folder.is_dir():
            continue
        entry = folder / "SKILL.md"
        if not entry.is_file():
            failures.append(f"missing {entry}")
            continue
        text = entry.read_text()
        header = re.match(r"\A---\n(.*?)\n---\n", text, re.S)
        name = re.search(r"^name: ([a-z0-9-]+)$", header[1], re.M) if header else None
        description = re.search(r"^description: (.+)$", header[1], re.M) if header else None
        if not name or name[1] != folder.name or not description:
            failures.append(f"invalid name/description: {entry}")
        elif name[1] in names:
            failures.append(f"duplicate skill: {name[1]}")
        else:
            names.add(name[1])
        for target in re.findall(r"\]\(([^)]+)\)", text):
            if "://" not in target and not target.startswith("#"):
                if not (folder / target.split("#", 1)[0]).exists():
                    failures.append(f"broken skill reference: {entry}: {target}")
        for host in (".agents", ".claude"):
            link = root / host / "skills" / folder.name
            if not link.is_symlink() or link.resolve() != folder.resolve():
                failures.append(f"missing/stale skill link: {link}")
        if installed is not None:
            for original in folder.rglob("*"):
                if original.is_file():
                    copy = installed / folder.name / original.relative_to(folder)
                    if not copy.is_file() or copy.read_bytes() != original.read_bytes():
                        failures.append(f"outdated global copy: {copy}")
    for host in (".agents", ".claude"):
        directory = root / host / "skills"
        if directory.is_dir():
            for link in directory.iterdir():
                if link.name not in names:
                    failures.append(f"unmanaged skill entry: {link}")
    return failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--installed", type=Path, help="also compare a legacy global install against canonical files")
    args = parser.parse_args()
    failures = check(ROOT, args.installed)
    for failure in failures:
        print(failure, file=sys.stderr)
    if not failures:
        print("agent skills check passed")
    return bool(failures)


if __name__ == "__main__":
    raise SystemExit(main())
