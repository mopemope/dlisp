"""Regression tests for agent tooling; no agent/API invocation required."""
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import patch


SCRIPTS = Path(__file__).resolve().parents[1]


def module(name):
    spec = importlib.util.spec_from_file_location(name, SCRIPTS / f"{name}.py")
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


class ValidationPlanTests(unittest.TestCase):
    def test_docs_do_not_require_cargo(self):
        plan = module("agent_check").plan(["docs/ai/README.md"])
        self.assertFalse(any(command[0] == "cargo" for command in plan))

    def test_runtime_includes_behavior_and_linking(self):
        plan = module("agent_check").plan(["runtime/src/maps.rs"])
        self.assertIn(["cargo", "test", "-p", "dlisp_runtime", "--quiet"], plan)
        self.assertIn(["cargo", "test", "-p", "dlisp", "--test", "parity_golden_tests", "--quiet"], plan)

    def test_unknown_code_uses_full_gate(self):
        plan = module("agent_check").plan(["new-crate/src/lib.rs"])
        self.assertIn(["cargo", "test", "--workspace", "--quiet"], plan)

    def test_parser_includes_execution_paths(self):
        plan = module("agent_check").plan(["core/src/parser.rs"])
        self.assertTrue(any("jit_parity_tests" in command for command in plan))
        self.assertTrue(any("parity_golden_tests" in command for command in plan))

    def test_no_changes_is_empty(self):
        self.assertEqual(module("agent_check").plan([]), [])


class TraceTests(unittest.TestCase):
    def test_process_uses_lightweight_build_defaults(self):
        import os
        import sys
        runner = module("run_agent_eval")
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / "log"
            for override, expected in ((None, "0"), ("1", "1")):
                with patch.dict(os.environ):
                    if override is None:
                        os.environ.pop("CARGO_PROFILE_TEST_DEBUG", None)
                    else:
                        os.environ["CARGO_PROFILE_TEST_DEBUG"] = override
                    with log.open("w") as stream:
                        code, timeout = runner.run_process(
                            [sys.executable, "-c", "import os; print(os.environ['CARGO_PROFILE_TEST_DEBUG'])"],
                            tmp, stream, stream, 5)
                self.assertEqual(code, 0)
                self.assertFalse(timeout)
                self.assertEqual(log.read_text().strip(), expected)

    def test_snapshot_has_head_for_repository_validation_commands(self):
        runner = module("run_agent_eval")
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / "source"
            source.mkdir()
            (source / "tracked.txt").write_text("baseline")
            runner.git(source, "init", "-q")
            runner.git(source, "add", "-A")
            tree = runner.git(source, "write-tree").decode().strip()
            revision = runner.git(source, "-c", "user.name=Fixture", "-c", "user.email=fixture@localhost", "commit-tree", tree, "-m", "fixture").decode().strip()
            work = root / "work"
            work.mkdir()
            with patch.object(runner, "ROOT", source):
                runner.snapshot(revision, work)
            self.assertTrue(runner.git(work, "rev-parse", "--verify", "HEAD"))
            self.assertEqual(runner.git(work, "diff", "HEAD"), b"")

    def test_seeded_cases_match_current_code(self):
        runner = module("run_agent_eval")
        cases = json.loads(runner.CASES.read_text())
        self.assertEqual(len(cases), len({case["id"] for case in cases}))
        for case in cases:
            for mutation in case.get("mutations", []):
                self.assertEqual((runner.ROOT / mutation["path"]).read_text().count(mutation["before"]), 1)
            if case["kind"] == "repair":
                self.assertIn(case["grader"], ("abi", "runtime"))
            else:
                for name in case["skills"]:
                    self.assertTrue((runner.ROOT / "docs/ai/skills" / name / "SKILL.md").is_file())

    def test_timeout_and_missing_executable_are_failures(self):
        runner = module("run_agent_eval")
        with tempfile.TemporaryDirectory() as tmp:
            with (Path(tmp) / "log").open("w") as log:
                code, timed_out = runner.run_process(["/bin/sleep", "10"], tmp, log, log, 0.01)
                self.assertNotEqual(code, 0)
                self.assertTrue(timed_out)
                code, timed_out = runner.run_process([str(Path(tmp) / "missing")], tmp, log, log, 1)
                self.assertEqual(code, 127)
                self.assertFalse(timed_out)

    def test_usage_is_not_double_counted_and_cache_is_separate(self):
        metrics = module("run_agent_eval").trace_metrics([
            {"type": "turn.completed", "usage": {"input_tokens": 100, "cached_input_tokens": 40, "output_tokens": 10}},
            {"type": "turn.completed", "usage": {"input_tokens": 80, "cached_input_tokens": 30, "output_tokens": 5}},
            {"type": "item.completed", "item": {"type": "command_execution", "command": "cat docs/ai/skills/dlisp-parser-syntax/SKILL.md", "exit_code": 0}},
        ])
        self.assertEqual(metrics["input_tokens"], 180)
        self.assertEqual(metrics["cached_input_tokens"], 70)
        self.assertEqual(metrics["output_tokens"], 15)
        self.assertEqual(metrics["failed_commands"], 0)
        self.assertIn("dlisp-parser-syntax", metrics["skills_read"])

    def test_missing_usage_is_unknown(self):
        metrics = module("run_agent_eval").trace_metrics([])
        self.assertIsNone(metrics["input_tokens"])

    def test_failed_turn_cannot_pass(self):
        metrics = module("run_agent_eval").trace_metrics([
            {"type": "turn.failed", "error": {"message": "interrupted"}}
        ])
        self.assertTrue(metrics["turn_failed"])

    def test_incomplete_trace_cannot_pass(self):
        self.assertFalse(module("run_agent_eval").trace_metrics([])["completed"])

    def test_staged_edits_cannot_evade_read_only_grader(self):
        runner = module("run_agent_eval")
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fake = root / "fake-codex"
            fake.write_text('''#!/usr/bin/env python3
import json, subprocess
from pathlib import Path
Path("tracked.txt").write_text("changed")
subprocess.run(["git", "add", "tracked.txt"], check=True)
print(json.dumps({"type":"turn.completed", "usage":{"input_tokens":10,"output_tokens":2}}))
''')
            fake.chmod(0o755)

            def snapshot(_ref, work):
                (work / "tracked.txt").write_text("original")
                runner.git(work, "init", "-q")
                runner.git(work, "add", "-A")

            args = SimpleNamespace(output_dir=root / "results", codex_bin=str(fake), model="fixture", effort="low", timeout=10)
            case = {"id": "fixture", "kind": "routing", "prompt": "inspect", "skills": [], "files": []}
            with patch.object(runner, "snapshot", snapshot):
                result = runner.evaluate(case, args, 1, "fixture-ref")
            self.assertFalse(result["passed"])
            self.assertEqual(result["changed_files"], ["tracked.txt"])
            self.assertEqual(result["input_tokens"], 10)


class InstallerTests(unittest.TestCase):
    def test_retirement_backs_up_only_repo_skills_and_is_idempotent(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "scripts").mkdir()
            installer = root / "scripts/install-runtime-skills.sh"
            installer.write_bytes((SCRIPTS / installer.name).read_bytes())
            canonical = root / "docs/ai/skills/demo"
            canonical.mkdir(parents=True)
            (canonical / "SKILL.md").write_text("current")
            legacy = root / "codex/skills/demo"
            legacy.mkdir(parents=True)
            (legacy / "SKILL.md").write_text("old custom content")
            unrelated = root / "codex/skills/unrelated"
            unrelated.mkdir()
            import os
            env = {**os.environ, "CODEX_HOME": str(root / "codex")}
            subprocess.run(["bash", str(installer), "repo-links"], check=True, capture_output=True)
            preview = subprocess.run(["bash", str(installer), "retire-codex", "--plan"], env=env, capture_output=True)
            self.assertEqual(preview.returncode, 0, preview.stderr)
            self.assertTrue(legacy.is_dir())
            self.assertFalse((root / "codex/skill-backups").exists())
            result = subprocess.run(["bash", str(installer), "retire-codex"], env=env, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse(legacy.exists())
            backups = list((root / "codex/skill-backups").glob("*/demo/SKILL.md"))
            self.assertEqual(len(backups), 1)
            self.assertEqual(backups[0].read_text(), "old custom content")
            self.assertTrue(unrelated.is_dir())
            result = subprocess.run(["bash", str(installer), "retire-codex"], env=env, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(len(list((root / "codex/skill-backups").iterdir())), 1)

    def test_links_follow_checkout_and_preserve_real_directories(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "scripts").mkdir()
            installer = root / "scripts/install-runtime-skills.sh"
            installer.write_bytes((SCRIPTS / installer.name).read_bytes())
            skill = root / "docs/ai/skills/demo"
            skill.mkdir(parents=True)
            (skill / "SKILL.md").write_text("first")
            result = subprocess.run(["bash", str(installer), "repo-links"], capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            for host in (".agents", ".claude"):
                link = root / host / "skills/demo"
                self.assertTrue(link.is_symlink())
                self.assertEqual(link.resolve(), skill)
            (skill / "SKILL.md").write_text("second")
            self.assertEqual((root / ".agents/skills/demo/SKILL.md").read_text(), "second")
            (root / ".agents/skills/demo").unlink()
            (root / ".agents/skills/demo").mkdir()
            result = subprocess.run(["bash", str(installer), "repo-links"], capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertTrue((root / ".agents/skills/demo").is_dir())


class AbiTests(unittest.TestCase):
    def test_empty_declarations_fail_closed(self):
        with self.assertRaises(ValueError):
            module("check_runtime_abi").codegen_signatures("")

    def test_unrecognized_push_fails_closed(self):
        source = '''let mut sig = module.make_signature();
sig.params.push(AbiParam::new(types::I64));
sig.params.push(AbiParam::new(types::I64).uext());
module.declare_function("dlisp_make_int", Linkage::Import, &sig);'''
        with self.assertRaises(ValueError):
            module("check_runtime_abi").codegen_signatures(source)

    def test_callback_parameter_is_one_pointer(self):
        abi = module("check_runtime_abi")
        signatures = abi.runtime_signatures('pub extern "C" fn dlisp_cb(f: extern "C" fn(*mut u8, i64) -> i64, x: bool) -> *mut u8 { }')
        self.assertEqual(signatures["dlisp_cb"], (["ptr", "I8"], ["ptr"]))

    def test_signature_mismatch_detected(self):
        abi = module("check_runtime_abi")
        source = '''let int = module.target_config().pointer_type();
let mut sig = module.make_signature();
sig.params.push(AbiParam::new(types::I32));
sig.returns.push(AbiParam::new(int));
module.declare_function("dlisp_make_int", Linkage::Import, &sig);'''
        declarations = abi.codegen_signatures(source)
        runtime = abi.runtime_signatures('pub extern "C" fn dlisp_make_int(x: i64) -> *mut u8 { }')
        self.assertTrue(abi.compare(declarations, runtime))

    def test_void_and_shared_signatures(self):
        abi = module("check_runtime_abi")
        source = '''let int = module.target_config().pointer_type();
let mut sig = module.make_signature();
sig.params.push(AbiParam::new(int));
module.declare_function("dlisp_a", Linkage::Import, &sig);
module.declare_function("dlisp_b", Linkage::Import, &sig);'''
        declarations = abi.codegen_signatures(source)
        self.assertEqual(declarations["dlisp_a"], (["ptr"], []))
        self.assertEqual(declarations["dlisp_b"], (["ptr"], []))


if __name__ == "__main__":
    unittest.main()
