"""Test comparator failures with synthetic pixels, never synthetic UI evidence."""
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import compare_pixels


class NativeComparisonTests(unittest.TestCase):
    def test_exact_pixels_and_single_channel_difference(self):
        with tempfile.TemporaryDirectory() as temporary:
            left, right = Path(temporary) / "linux", Path(temporary) / "windows"
            left.mkdir()
            right.mkdir()
            with self.assertRaises(ValueError):
                compare_pixels.compare(left, right)
            for root in (left, right):
                for name, size in compare_pixels.EXPECTED.items():
                    (root / name).write_bytes(bytes(size))
            with patch("builtins.print"):
                compare_pixels.compare(left, right)
            target = right / "results-2x.rgb"
            with target.open("r+b") as pixels:
                pixels.seek(-1, 2)
                pixels.write(b"\x01")
            with self.assertRaisesRegex(ValueError, "pixels differ"):
                compare_pixels.compare(left, right)
            target.write_bytes(b"")
            with self.assertRaisesRegex(ValueError, "dimensions"):
                compare_pixels.compare(left, right)
            target.unlink()
            with self.assertRaisesRegex(ValueError, "inventory"):
                compare_pixels.compare(left, right)
