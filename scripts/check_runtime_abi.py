#!/usr/bin/env python3
"""Check runtime extern signatures against explicit Cranelift import signatures.

This recognizes the declaration style in codegen/builtins.rs, not arbitrary Rust.
Unknown linked types/declarations fail closed. It checks ABI widths and arity,
not pointee types, ownership, struct layouts or calling-convention attributes.
"""
import argparse
from pathlib import Path
import re


ROOT = Path(__file__).resolve().parents[1]


def without_comments(source):
    return re.sub(r"//[^\n]*|/\*.*?\*/", "", source, flags=re.S)


def split_params(text):
    start = depth = 0
    result = []
    for index, char in enumerate(text):
        if char in "(<[":
            depth += 1
        elif char in ")>]" and not (char == ">" and index and text[index - 1] == "-"):
            depth -= 1
        elif char == "," and depth == 0:
            result.append(text[start:index].strip())
            start = index + 1
    result.append(text[start:].strip())
    return [item for item in result if item]


def abi_type(text):
    text = text.strip()
    if text.startswith(("*", "&")) or 'extern "C" fn' in text:
        return "ptr"
    if text in ("usize", "isize"):
        return "ptr"
    mapping = {"bool": "I8", "i8": "I8", "u8": "I8", "i16": "I16", "u16": "I16",
               "i32": "I32", "u32": "I32", "i64": "I64", "u64": "I64", "f32": "F32", "f64": "F64"}
    if text not in mapping:
        raise ValueError(f"unsupported runtime ABI type: {text}")
    return mapping[text]


def runtime_signatures(source):
    source = without_comments(source)
    signatures = {}
    for match in re.finditer(r'pub\s+(?:unsafe\s+)?extern\s+"C"\s+fn\s+(dlisp_\w+)\s*\(', source):
        start = match.end()
        end = start
        depth = 1
        while depth:
            if source[end] == "(":
                depth += 1
            elif source[end] == ")":
                depth -= 1
            end += 1
        params = split_params(source[start:end - 1])
        returns = source[end:source.index("{", end)].strip()
        signatures[match[1]] = (
            [abi_type(param.split(":", 1)[1]) for param in params],
            [] if not returns or returns == "-> ()" else [abi_type(returns.removeprefix("->").strip())],
        )
    return signatures


def codegen_signatures(source):
    source = without_comments(source)
    signatures = {}
    declared = {}
    recognized_accesses = 0
    pattern = re.compile(
        r"let mut (\w+) = module\.make_signature\(\);"
        r"|(\w+)\.(params|returns)\.push\(AbiParam::new\((int|types::\w+)\)\);"
        r'|\.declare_function\(\s*"(dlisp_\w+)"\s*,\s*Linkage::Import\s*,\s*&(\w+)\s*\)'
    )
    for match in pattern.finditer(source):
        name, used, side, kind, symbol, signature = match.groups()
        if name:
            signatures[name] = ([], [])
        elif used:
            recognized_accesses += 1
            if used not in signatures:
                raise ValueError(f"unknown signature variable: {used}")
            signatures[used][0 if side == "params" else 1].append("ptr" if kind == "int" else kind.removeprefix("types::"))
        else:
            if signature not in signatures:
                raise ValueError(f"unknown declaration signature: {signature}")
            declared[symbol] = tuple(list(part) for part in signatures[signature])
    linked = set(re.findall(r'declare_function\(\s*"(dlisp_\w+)"', source))
    if not linked:
        raise ValueError("no runtime imports found; refusing an empty ABI check")
    if linked != declared.keys():
        raise ValueError(f"unrecognized import declarations: {sorted(linked - declared.keys())}")
    # Fail if signature construction grows beyond the supported push style.
    accesses = re.findall(r"\b\w+\.(?:params|returns)\b", source)
    if len(accesses) != recognized_accesses:
        raise ValueError("unsupported signature operation; not all parameter/return accesses were parsed")
    return declared


def compare(declarations, runtime):
    failures = []
    for name, signature in sorted(declarations.items()):
        if name not in runtime:
            failures.append(f"{name}: runtime export missing")
        elif signature != runtime[name]:
            failures.append(f"{name}: codegen {signature}, runtime {runtime[name]}")
    return failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    args = parser.parse_args()
    try:
        declarations = codegen_signatures((args.root / "core/src/codegen/builtins.rs").read_text())
        runtime = {}
        for path in sorted((args.root / "runtime/src").glob("*.rs")):
            if "tests" not in path.stem:
                runtime.update(runtime_signatures(path.read_text()))
        failures = compare(declarations, runtime)
    except (ValueError, IndexError, OSError) as error:
        print(f"runtime ABI check failed: {error}")
        return 1
    for failure in failures:
        print(failure)
    if not failures:
        print(f"runtime ABI check passed ({len(declarations)} imports)")
    return bool(failures)


if __name__ == "__main__":
    raise SystemExit(main())
