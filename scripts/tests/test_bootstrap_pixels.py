"""Exact native comparator rejects empty, changed and truncated RGB artifacts."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location("bootstrap_pixels", Path(__file__).parents[2] / "ui/compare_pixels.py")
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class BootstrapPixelTests(unittest.TestCase):
    def test_exact_inventory_dimensions_and_bytes(self):
        with tempfile.TemporaryDirectory() as scratch:
            a, b = Path(scratch) / "a", Path(scratch) / "b"
            a.mkdir()
            b.mkdir()
            with self.assertRaises(ValueError):
                guard.compare(a, b)
            for name, size in guard.EXPECTED.items():
                for root in (a, b):
                    (root / name).write_bytes(bytes(size))
            guard.compare(a, b)
            name = next(iter(guard.EXPECTED))
            data = bytearray((a / name).read_bytes())
            data[0] = 1
            (b / name).write_bytes(data)
            with self.assertRaisesRegex(ValueError, "pixels differ"):
                guard.compare(a, b)
            (b / name).write_bytes(b"")
            with self.assertRaisesRegex(ValueError, "dimensions"):
                guard.compare(a, b)
