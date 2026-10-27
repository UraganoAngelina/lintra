"""Boundary tests for the exact-byte cross-OS comparator; no engine mocks."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location("probe", Path(__file__).parents[2] / "tests/packaging_probe.py")
probe = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(probe)


class PackagingProbeTests(unittest.TestCase):
    def test_artifact_uses_exactly_one_existing_path_from_build_log(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary, log = root / "binary with spaces", root / "build.log"
            binary.write_bytes(b"synthetic placeholder, never executed")
            log.write_text(f"reproducible=true\nartifact={binary}\n", encoding="utf-8")
            self.assertEqual(probe.artifact(log), binary)
            for text in ("", "artifact=missing\n", f"artifact={binary}\nartifact={binary}\n"):
                log.write_text(text, encoding="utf-8")
                with self.assertRaises(ValueError):
                    probe.artifact(log)

    def test_jsonl_encoding_and_newlines(self):
        for value in (b"", '{"value":"É"}\n'.encode()):
            probe.jsonl(value)
        for value in (b'\xef\xbb\xbf{}\n', b'{}\r\n', b'{}', b'[]\n', b'\xff\n'):
            with self.assertRaises(ValueError):
                probe.jsonl(value)

    def test_compare_requires_exact_inventory_and_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            a, b = Path(temporary) / "linux", Path(temporary) / "windows"
            a.mkdir()
            b.mkdir()
            with self.assertRaises(ValueError):
                probe.compare(a, b)
            for root in (a, b):
                for name in probe.CASES:
                    (root / name).write_bytes(b'{}\n' if name != 'external-override.jsonl' else b'')
            probe.compare(a, b)
            (b / "embedded.jsonl").write_bytes(b'{"different":true}\n')
            with self.assertRaises(ValueError):
                probe.compare(a, b)
            (b / "embedded.jsonl").write_bytes(b'{}\n')
            (b / "unexpected").write_bytes(b'')
            with self.assertRaises(ValueError):
                probe.compare(a, b)

    def test_empty_diagnostics_are_not_a_cross_os_proof(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for name in probe.CASES:
                (root / name).write_bytes(b'')
            with self.assertRaises(ValueError):
                probe.compare(root, root)
