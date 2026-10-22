#!/usr/bin/env python3
"""Run repeatable Codex routing/repair evaluations in disposable git snapshots."""
import argparse
import io
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import tarfile
import tempfile
import time


ROOT = Path(__file__).resolve().parents[1]
CASES = ROOT / "docs/ai/evals/cases.json"


def trace_metrics(events):
    totals = {key: None for key in ("input_tokens", "cached_input_tokens", "output_tokens")}
    skills = set()
    commands = []
    completed = False
    failed = False
    messages = []
    for event in events:
        if event.get("type") == "turn.completed":
            completed = True
            for key in totals:
                value = event.get("usage", {}).get(key)
                if isinstance(value, int):
                    totals[key] = (totals[key] or 0) + value
        failed |= event.get("type") in ("turn.failed", "error")
        item = event.get("item", {})
        if event.get("type") == "item.completed" and item.get("type") == "command_execution":
            command = item.get("command", "")
            commands.append(item)
            if item.get("exit_code") == 0:
                # Evidence of an explicit read, not just a skill mentioned in the final answer.
                if re.search(r"\b(cat|sed|head|tail|read_text)\b", command):
                    skills.update(re.findall(r"(dlisp-[a-z-]+)/SKILL\.md", command))
        if event.get("type") == "item.completed" and item.get("type") == "agent_message":
            messages.append(item.get("text", ""))
    return {
        **totals, "skills_read": sorted(skills), "commands": len(commands),
        "failed_commands": sum(item.get("exit_code", 0) != 0 for item in commands),
        "turn_failed": failed, "completed": completed, "final_text": "\n".join(messages),
    }


def git(root, *args):
    return subprocess.run(["git", *args], cwd=root, check=True, capture_output=True).stdout


def run_process(command, work, stdout, stderr, timeout):
    """Bound the lifetime of both the CLI and any tool/test children."""
    environment = os.environ.copy()
    for key, value in {"CARGO_PROFILE_DEV_DEBUG": "0", "CARGO_PROFILE_TEST_DEBUG": "0",
                       "CARGO_INCREMENTAL": "0", "CARGO_BUILD_JOBS": "2"}.items():
        environment.setdefault(key, value)
    try:
        process = subprocess.Popen(command, cwd=work, stdin=subprocess.DEVNULL,
                                   stdout=stdout, stderr=stderr, env=environment, start_new_session=True)
    except OSError as error:
        # Keep infrastructure failures reviewable in the same result format.
        print(f"failed to start {command[0]}: {error}", file=stdout, flush=True)
        return 127, False
    try:
        return process.wait(timeout=timeout), False
    except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait()
        if isinstance(error, KeyboardInterrupt):
            raise
        return process.returncode, True


def snapshot(ref, destination):
    # A committed revision makes baseline/candidate comparisons reproducible.
    archive = git(ROOT, "archive", "--format=tar", ref)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        tar.extractall(destination, filter="data")
    git(destination, "init", "-q")
    git(destination, "add", "-A")
    tree = git(destination, "write-tree").decode().strip()
    baseline = git(destination, "-c", "user.name=Agent Evaluation", "-c", "user.email=agent-eval@localhost",
                   "commit-tree", tree, "-m", f"Evaluation baseline for {ref}").decode().strip()
    git(destination, "-c", "core.hooksPath=/dev/null", "update-ref", "HEAD", baseline)


def changed_files(work, baseline):
    tracked = git(work, "diff", baseline, "--name-only", "-z").decode().split("\0")
    untracked = git(work, "ls-files", "--others", "--exclude-standard", "-z").decode().split("\0")
    return sorted(set(tracked + untracked) - {""})


def evaluate(case, args, iteration, revision):
    output = args.output_dir / f"{case['id']}-{iteration}"
    output.mkdir(parents=True, exist_ok=False)
    with tempfile.TemporaryDirectory(prefix="dlisp-agent-eval-") as tmp:
        work = Path(tmp)
        snapshot(revision, work)
        baseline = git(work, "write-tree").decode().strip()
        for mutation in case.get("mutations", []):
            path = work / mutation["path"]
            source = path.read_text()
            if source.count(mutation["before"]) != 1:
                raise ValueError(f"mutation no longer applies uniquely: {path}")
            path.write_text(source.replace(mutation["before"], mutation["after"]))
        # Save trusted runtime tests outside the agent workspace for repair graders.
        trusted = {path.relative_to(work): path.read_bytes() for path in (work / "runtime/src").glob("*tests*.rs")}
        command = [args.codex_bin, "exec", "--json", "--model", args.model,
                   "--config", f'model_reasoning_effort="{args.effort}"',
                   "--sandbox", "workspace-write" if case["kind"] == "repair" else "read-only",
                   case["prompt"]]
        (output / "request.json").write_text(json.dumps({"revision": revision, "case": case, "command": command}, ensure_ascii=False, indent=2))
        started = time.monotonic()
        with (output / "trace.jsonl").open("w") as trace, (output / "stderr.log").open("w") as stderr:
            returncode, timed_out = run_process(command, work, trace, stderr, args.timeout)
        elapsed = time.monotonic() - started
        events = []
        malformed = False
        for line in (output / "trace.jsonl").read_text().splitlines():
            try:
                event = json.loads(line)
                if not isinstance(event, dict):
                    raise ValueError("event must be an object")
                events.append(event)
            except ValueError:
                malformed = True
        metrics = trace_metrics(events)
        changed = changed_files(work, baseline)
        (output / "changes.patch").write_bytes(git(work, "diff", baseline, "--binary"))
        checks = {}
        if case["kind"] == "routing":
            checks["no_edits"] = not changed
            checks["routing_evidence"] = set(case["skills"]).issubset(metrics["skills_read"])
            checks["references"] = all(path in metrics["final_text"] for path in case["files"])
        else:
            checks["scope"] = set(changed).issubset(case["allowed_paths"])
            for relative, content in trusted.items():
                (work / relative).write_bytes(content)
            grader = case["grader"]
            if grader == "abi":
                grade_commands = [
                    ["python3", str(ROOT / "scripts/check_runtime_abi.py"), "--root", str(work)],
                    ["cargo", "check", "-p", "dlisp-core", "--quiet"],
                ]
            else:
                grade_commands = [["cargo", "test", "-p", "dlisp_runtime", "--quiet"]]
            with (output / "grader.log").open("w") as log:
                checks["behavior"] = True
                for grade_command in grade_commands:
                    grade_code, grade_timeout = run_process(grade_command, work, log, subprocess.STDOUT, args.timeout)
                    if grade_code != 0 or grade_timeout:
                        checks["behavior"] = False
                        break
        passed = (returncode == 0 and metrics["completed"] and not metrics["turn_failed"]
                  and not timed_out and not malformed and all(checks.values()))
        result = {"case": case["id"], "kind": case["kind"], "iteration": iteration,
                  "revision": revision, "model": args.model, "effort": args.effort,
                  "seconds": round(elapsed, 3), "returncode": returncode, "timed_out": timed_out,
                  "malformed_trace": malformed, **metrics, "changed_files": changed, "checks": checks, "passed": passed}
        (output / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2))
        return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--case", action="append", help="case id; repeat to select several (default: all)")
    parser.add_argument("--ref", default="HEAD", help="committed revision to evaluate; uncommitted edits are excluded")
    parser.add_argument("--model", help="explicit model required for real runs")
    parser.add_argument("--effort", choices=("low", "medium", "high", "xhigh"), default="medium")
    parser.add_argument("--repeat", type=int, default=1)
    parser.add_argument("--timeout", type=int, default=900)
    parser.add_argument("--output-dir", type=Path)
    parser.add_argument("--codex-bin", default="codex", help="Codex executable, or a fixture runner for tooling tests")
    args = parser.parse_args()
    cases = json.loads(CASES.read_text())
    if args.list:
        for case in cases:
            print(f"{case['id']}\t{case['kind']}\t{case['prompt']}")
        return 0
    if not args.model or not args.output_dir or args.repeat < 1 or args.timeout < 1:
        parser.error("--model, --output-dir, positive --repeat and --timeout are required")
    selected = [case for case in cases if not args.case or case["id"] in args.case]
    if args.case and set(args.case) - {case["id"] for case in cases}:
        parser.error("unknown case id")
    args.output_dir = args.output_dir.resolve()
    args.codex_bin = str(Path(args.codex_bin).resolve()) if "/" in args.codex_bin else args.codex_bin
    revision = git(ROOT, "rev-parse", "--verify", f"{args.ref}^{{commit}}").decode().strip()
    results = []
    for case in selected:
        for iteration in range(1, args.repeat + 1):
            result = evaluate(case, args, iteration, revision)
            results.append(result)
            print(f"{case['id']} #{iteration}: {'PASS' if result['passed'] else 'FAIL'}", flush=True)
    passed = sum(row["passed"] for row in results)
    usage = {}
    for key in ("input_tokens", "cached_input_tokens", "output_tokens"):
        values = [row[key] for row in results]
        usage[key] = sum(values) if all(value is not None for value in values) else None
    total = (usage["input_tokens"] + usage["output_tokens"]
             if usage["input_tokens"] is not None and usage["output_tokens"] is not None else None)
    summary = {"revision": revision, "runs": len(results), "passed": passed,
               "usage": usage, "tokens_per_success": total / passed if total is not None and passed else None,
               "by_kind": {kind: {"runs": sum(row["kind"] == kind for row in results),
                                   "passed": sum(row["kind"] == kind and row["passed"] for row in results)}
                           for kind in ("routing", "repair")}, "results": results}
    (args.output_dir / "summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2))
    return int(summary["passed"] != summary["runs"])


if __name__ == "__main__":
    raise SystemExit(main())
