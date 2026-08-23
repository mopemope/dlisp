#!/usr/bin/env python3
"""Cross-check the JIT/AOT codegen surface against its sources.

Verifies the four invariants documented in
docs/ai/skills/dlisp-codegen-aot-jit/references/parity-map.md:

A. every name in COMPILED_BUILTINS is a registered builtin or special form
B. every lowering-arm FFI field has a FuncId declaration and an AOT
   declare_function entry
C. every JIT-bound / AOT-imported dlisp_* symbol is exported by the runtime
D. every COMPILED_BUILTINS name has a lowering arm in codegen/forms/builtins.rs

Exit status: 0 if all invariants hold, 1 otherwise.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parents[1]

failures: list[str] = []
warnings: list[str] = []


def read(path: str) -> str:
    full = ROOT_DIR / path
    try:
        return full.read_text(encoding="utf-8")
    except FileNotFoundError:
        raise SystemExit(f"failed to locate {path}") from None


def compiled_builtins() -> set[str]:
    text = read("core/src/codegen/mod.rs")
    match = re.search(r"COMPILED_BUILTINS[^=]*=\s*&\[", text)
    if not match:
        raise SystemExit("COMPILED_BUILTINS not found")
    body = text[match.end():]
    end = body.index("]")
    return set(re.findall(r'"([^"]+)"', body[:end]))


def registered_builtins() -> set[str]:
    return set(
        re.findall(
            r'env\.set\(\s*"([^"]+)"\.to_string\(',
            read("core/src/builtins/mod.rs"),
        )
    )


def registered_forms() -> set[str]:
    forms = set(
        re.findall(r'reg\.register\(\s*"([^"]+)"', read("core/src/forms/registry.rs"))
    )
    if re.search(r'\bs\s*==\s*"defmacro"', read("core/src/interpreter.rs")):
        forms.add("defmacro")
    return forms


def lowering_arms() -> tuple[dict[str, set[str]], set[str]]:
    """Return (builtin name -> referenced dlisp_* fields, all arm heads)."""
    text = read("core/src/codegen/forms/builtins.rs")
    arms: dict[str, set[str]] = {}
    heads: set[str] = set()
    current: str | None = None
    for line in text.splitlines():
        head = re.match(r'^\s*"([^"]+)"\s*=>', line)
        if head:
            current = head.group(1)
            heads.add(current)
            arms.setdefault(current, set())
        for field in re.findall(r"funcs\.(dlisp_[a-z0-9_]+)", line):
            target = current if current is not None else ""
            arms.setdefault(target, set()).add(field)
    return arms, heads


def funcid_fields() -> set[str]:
    return set(
        re.findall(r"pub\s+(dlisp_[a-z0-9_]+)\s*:\s*FuncId", read("core/src/codegen/builtins.rs"))
    )


def aot_declared_symbols() -> set[str]:
    return set(
        re.findall(
            r'declare_function\(\s*"(dlisp_[a-z0-9_]+)"',
            read("core/src/codegen/builtins.rs"),
        )
    )


def jit_bound_symbols() -> set[str]:
    return set(
        re.findall(r'\.symbol\(\s*"([a-z0-9_]+)"', read("core/src/jit.rs"))
    )


def runtime_exports() -> tuple[set[str], dict[str, str]]:
    """Return (export names, name -> defining file). Test fixtures excluded."""
    exports: set[str] = set()
    where: dict[str, str] = {}
    pattern = re.compile(r'(?:unsafe\s+)?extern\s+"C"\s+fn\s+([a-z0-9_]+)')
    src_dir = ROOT_DIR / "runtime/src"
    for path in sorted(src_dir.glob("*.rs")):
        if path.name == "verify_tests.rs":
            continue
        for name in pattern.findall(path.read_text(encoding="utf-8")):
            exports.add(name)
            where[name] = f"runtime/src/{path.name}"
    return exports, where


def cross_reference_counts(exports: set[str]) -> dict[str, int]:
    """Count occurrences of each export name across relevant Rust sources."""
    texts: list[str] = []
    for pattern_ in ("core/src/*.rs", "core/src/*/*.rs", "runtime/src/*.rs", "cli/src/*.rs"):
        for path in sorted(ROOT_DIR.glob(pattern_)):
            if path.name == "verify_tests.rs":
                continue
            texts.append(path.read_text(encoding="utf-8"))
    return {
        name: sum(len(re.findall(rf"\b{re.escape(name)}\b", text)) for text in texts)
        for name in exports
    }


def fail(check: str, message: str) -> None:
    failures.append(f"[{check}] {message}")


def main() -> int:
    builtins = registered_builtins()
    forms = registered_forms()
    compiled = compiled_builtins()
    arms, heads = lowering_arms()
    fields = funcid_fields()
    aot_syms = aot_declared_symbols()
    jit_syms = jit_bound_symbols()
    exports, _ = runtime_exports()

    # A. COMPILED_BUILTINS must resolve to real surface names.
    for name in sorted(compiled - builtins - forms):
        fail(
            "A",
            f"`{name}` is in COMPILED_BUILTINS but registered as neither "
            "builtin nor special form",
        )

    # B. Lowering arms must have FuncId fields and AOT declarations.
    used_fields = set().union(*arms.values()) if arms else set()
    for sym in sorted(used_fields - fields):
        fail("B", f"lowering uses `{sym}` but no FuncId field exists in codegen/builtins.rs")
    for sym in sorted(used_fields & fields - aot_syms):
        fail("B", f"FuncId `{sym}` has no declare_function entry in codegen/builtins.rs")

    # C. Every linked symbol must exist in the runtime.
    for sym in sorted((jit_syms | aot_syms) - exports):
        fail("C", f"symbol `{sym}` is linked by JIT/AOT but not exported by runtime")

    # D. Every compiled builtin needs a lowering arm.
    for name in sorted(compiled - heads):
        fail("D", f"`{name}` is in COMPILED_BUILTINS but has no lowering arm")

    # Informational: exports never linked by JIT/AOT and never referenced
    # anywhere else (definition is the sole occurrence).
    ref_counts = cross_reference_counts(exports)
    unused = {
        sym
        for sym in exports - jit_syms - aot_syms
        if ref_counts.get(sym, 0) <= 1
    }
    for sym in sorted(unused):
        warnings.append(f"runtime export `{sym}` is not referenced by jit.rs or codegen/builtins.rs")

    for warning in warnings:
        print(f"warning: {warning}")
    if failures:
        print("codegen parity check FAILED:")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print(
        "codegen parity check passed "
        f"({len(compiled)} compiled builtins, {len(jit_syms)} JIT symbols, "
        f"{len(exports)} runtime exports)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
