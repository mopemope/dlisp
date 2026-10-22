#!/usr/bin/env python3
"""Select conservative validation commands from changed paths, without an LLM."""
import argparse
from pathlib import Path
import shlex
import subprocess


ROOT = Path(__file__).resolve().parents[1]
DOC_CHECKS = [
    ["python3", "scripts/check_doc_refs.py"],
    ["python3", "scripts/check_agent_skills.py"],
    ["python3", "docs/ai/skills/dlisp-language-surface/scripts/extract_surface.py", "check"],
]
TOOL_TESTS = ["python3", "-m", "unittest", "discover", "-s", "scripts/tests", "-q"]
FULL = [["bash", "scripts/check.sh"], ["cargo", "test", "--workspace", "--quiet"]]


def test(package, target=None):
    return ["cargo", "test", "-p", package, *(["--test", target] if target else []), "--quiet"]


def plan(paths, final=False):
    if not paths:
        return []
    commands = []
    code = False
    for path in paths:
        if path.startswith(("docs/", ".agents/skills/", ".claude/skills/")) or path in ("AGENTS.md", "README.md", "FUNCTIONS.md", "spec.md"):
            commands.extend(DOC_CHECKS)
        elif path.startswith("scripts/"):
            commands.extend([TOOL_TESTS, *DOC_CHECKS])
        elif path.startswith(("core/", "cli/", "runtime/", "stdlib/", "example-lisp/")):
            code = True
            if Path(path).parent == Path("core/tests") and path.endswith(".rs"):
                commands.append(test("dlisp-core", Path(path).stem))
            elif Path(path).parent == Path("cli/tests") and path.endswith(".rs"):
                commands.append(test("dlisp", Path(path).stem))
            elif path in ("core/src/parser.rs", "core/src/ast.rs"):
                commands.extend([test("dlisp-core", "parser_comments_tests"), test("dlisp-core", "backquote_tests")])
            elif path.startswith("runtime/"):
                commands.append(test("dlisp_runtime"))
            elif path.startswith("stdlib/"):
                commands.extend([test("dlisp-core", "stdlib_tests"), test("dlisp-core", "require_tests"), test("dlisp-core", "loop_recur_tests")])
            elif path.startswith("core/"):
                commands.append(test("dlisp-core", "interpreter_tests"))
            else:
                commands.append(test("dlisp", "integration_tests"))
            commands.extend([test("dlisp-core", "jit_parity_tests"), test("dlisp", "parity_golden_tests")])
        else:
            # Build files, CI and unknown areas may affect the whole workspace.
            commands.extend(FULL)
            code = True
    if final and code:
        commands.extend(FULL)
    return [list(command) for command in dict.fromkeys(tuple(command) for command in commands)]


def changed_paths(base):
    paths = []
    for args in (["diff", "--name-only", "-z", base, "--"], ["ls-files", "--others", "--exclude-standard", "-z"]):
        result = subprocess.run(["git", *args], cwd=ROOT, check=True, capture_output=True)
        paths.extend(result.stdout.decode().rstrip("\0").split("\0") if result.stdout else [])
    return sorted(set(paths))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--plan", action="store_true", help="print commands only (default)")
    mode.add_argument("--focused", action="store_true", help="run selected checks")
    mode.add_argument("--final", action="store_true", help="run selected checks plus the full code gate")
    parser.add_argument("--base", default="HEAD", help="git revision to compare; includes staged and unstaged changes")
    parser.add_argument("paths", nargs="*", help="explicit paths instead of git diff")
    args = parser.parse_args()
    commands = plan(args.paths or changed_paths(args.base), final=args.final)
    if not commands:
        print("no changed files")
    for command in commands:
        print(shlex.join(command), flush=True)
        if args.focused or args.final:
            result = subprocess.run(command, cwd=ROOT)
            if result.returncode:
                return result.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
