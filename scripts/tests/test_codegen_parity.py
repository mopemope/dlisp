"""The parity gate must detect a missing JIT registration, not just exports."""
import importlib.util
from pathlib import Path
import re
import shutil
import tempfile
import unittest
from contextlib import redirect_stdout
from io import StringIO


class JitRegistrationTests(unittest.TestCase):
    def test_missing_read_file_binding_fails_with_symbol_name(self):
        root = Path(__file__).resolve().parents[2]
        spec = importlib.util.spec_from_file_location("parity", root / "scripts/check_codegen_parity.py")
        parity = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(parity)
        with tempfile.TemporaryDirectory() as tmp:
            fixture = Path(tmp)
            shutil.copytree(root / "core/src", fixture / "core/src")
            shutil.copytree(root / "runtime/src", fixture / "runtime/src")
            jit = fixture / "core/src/jit.rs"
            jit.write_text(re.sub(r'builder\.symbol\(\s*"dlisp_read_file"\s*,.*?\);', "", jit.read_text(), flags=re.S))
            parity.ROOT_DIR = fixture
            output = StringIO()
            with redirect_stdout(output):
                result = parity.main()
            self.assertEqual(result, 1, output.getvalue())
            self.assertIn("dlisp_read_file", output.getvalue())
