"""Archive policy tests use synthetic payloads; native executables are gated in CI."""
import importlib.util
from pathlib import Path
import tempfile
import sys
import unittest

spec = importlib.util.spec_from_file_location("release_ui_test", Path(__file__).resolve().parents[1] / "release.py")
r = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r)


class DesktopArchiveTests(unittest.TestCase):
    def test_ui_archives_are_deterministic_include_exact_licenses_and_reject_cli_policy(self):
        for target in r.TARGETS:
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                binary = root / "synthetic"
                binary.write_bytes(b"synthetic desktop executable bytes")
                first = r.pack(binary, root / "first", "0.1.1-rc.1", "a" * 40, target, {}, product="ui")
                second = r.pack(binary, root / "second", "0.1.1-rc.1", "a" * 40, target, {}, product="ui")
                self.assertEqual(first.read_bytes(), second.read_bytes())
                self.assertTrue(first.name.startswith("lintra-ui-"))
                audit = r.inspect_archive(first, "0.1.1-rc.1", "a" * 40, target, product="ui")
                self.assertEqual(audit["product"], "ui")
                with self.assertRaises(ValueError):
                    r.inspect_archive(first, "0.1.1-rc.1", "a" * 40, target)
                with self.assertRaises(ValueError):
                    r.inspect_archive(first, "0.1.1-rc.1", "b" * 40, target, product="ui")
                with self.assertRaises(FileExistsError):
                    r.pack(binary, root / "first", "0.1.1-rc.1", "a" * 40, target, {}, product="ui")
                with self.assertRaises(ValueError):
                    r.product_files("unknown", target)


class RuntimeObservationTests(unittest.TestCase):
    def test_runtime_trace_requires_real_observations_and_rejects_extra_libraries(self):
        sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
        try:
            import ui_package
            allowed = {"libc.so.6", "libX11.so.6"}
            trace = "123: calling init: /usr/lib/libc.so.6\n123: calling init: /usr/lib/libX11.so.6\n"
            self.assertEqual(ui_package.runtime_libraries(trace, allowed), ["libX11.so.6", "libc.so.6"])
            for invalid in ["", trace + "123: calling init: /usr/lib/libunknown.so.1\n", "123: calling init: /tmp/libc.so.6\n"]:
                with self.assertRaises(ValueError):
                    ui_package.runtime_libraries(invalid, allowed)
        finally:
            sys.path.pop(0)
