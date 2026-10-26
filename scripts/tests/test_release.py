"""Synthetic archives and isolated API doubles; these never publish a release."""
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import urllib.error
import zipfile

SPEC = importlib.util.spec_from_file_location("release", Path(__file__).parents[1] / "release.py")
r = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(r)
SHA = "a" * 40
VERSION = r.tomllib.loads((r.ROOT / "Cargo.toml").read_text())["package"]["version"]
PREFIX = "/repos/" + r.DESTINATION


class Api:
    """Models only release protocol outcomes, not engine/provider capabilities."""
    def __init__(self, *, existing=False, draft=False, bad_digest=False, fail_upload=False, immutable=True):
        self.existing, self.draft, self.bad_digest = existing, draft, bad_digest
        self.fail_upload, self.immutable = fail_upload, immutable
        self.calls, self.assets = [], []

    def request(self, method, path, payload=None, *, upload=None, missing=False):
        self.calls.append((method, path, payload))
        if "/actions/workflows/" in path:
            return {"workflow_runs": [{"head_sha": SHA, "conclusion": "success", "status": "completed", "run_number": 1, "run_attempt": 1}]}
        if path == PREFIX:
            return {"full_name": r.DESTINATION, "private": True, "default_branch": "main"}
        if "/git/ref/tags/" in path:
            return {} if self.existing else None
        if "/releases?" in path:
            return [{"tag_name": "v" + VERSION}] if self.draft else []
        if "/commits/" in path:
            return {"sha": "b" * 40}
        if path.endswith("/git/refs"):
            return {}
        if path.endswith("/releases") and method == "POST":
            return {"id": 7}
        if upload is not None:
            if self.fail_upload:
                raise RuntimeError("synthetic upload failure")
            value = {"name": path.split("name=")[1], "size": len(upload), "state": "uploaded", "digest": "sha256:" + ("0" * 64 if self.bad_digest else r.digest(upload))}
            self.assets.append(value)
            return value
        if path.endswith("/assets"):
            return self.assets
        return {"tag_name": "v" + VERSION, "assets": self.assets, "draft": False, "immutable": self.immutable, "html_url": "https://github.com/" + r.DESTINATION + "/releases/tag/v" + VERSION}


class ReleaseTests(unittest.TestCase):
    def test_identity_and_ci_guards_reject_missing_or_unrelated_context(self):
        r.identity(VERSION, SHA)
        for version, sha in [("../bad", SHA), ("0.0.1", SHA), (VERSION, "short")]:
            with self.assertRaises(ValueError):
                r.identity(version, sha)
        for env in [{}, {"GITHUB_ACTIONS": "true", "GITHUB_REPOSITORY": r.SOURCE, "GITHUB_SHA": "wrong"}]:
            with self.assertRaises(ValueError):
                r.require_ci(SHA, env)

    def test_tags_and_drafts_both_reserve_version_without_mutations(self):
        for api in [Api(existing=True), Api(draft=True)]:
            with self.assertRaises(ValueError):
                r.unused_version(api, VERSION)
            self.assertTrue(all(method == "GET" for method, _, _ in api.calls))

    def test_archive_determinism_inventory_provenance_and_no_clobber(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / "synthetic-binary"
            binary.write_bytes(b"explicit synthetic executable payload")
            for target in r.TARGETS:
                left = r.pack(binary, root / (target + "1"), VERSION, SHA, target, {})
                right = r.pack(binary, root / (target + "2"), VERSION, SHA, target, {})
                self.assertEqual(left.read_bytes(), right.read_bytes())
                audit = r.inspect_archive(left, VERSION, SHA, target)
                self.assertEqual(audit["binary_sha256"], r.digest(binary.read_bytes()))
                with self.assertRaises(ValueError):
                    r.inspect_archive(left, VERSION, "c" * 40, target)
                before = left.read_bytes()
                with self.assertRaises(FileExistsError):
                    r.pack(binary, left.parent, VERSION, SHA, target, {})
                self.assertEqual(left.read_bytes(), before)

    def archives(self, root):
        binary = root / "payload"
        binary.write_bytes(b"synthetic verified-by-test payload")
        directory = root / "downloaded"
        directory.mkdir()
        for index, target in enumerate(r.TARGETS):
            source = r.pack(binary, root / str(index), VERSION, SHA, target, {"reproducible": True, "artifact_verified": True})
            source.rename(directory / source.name)
        return directory

    def test_draft_is_published_only_after_all_server_digests_match(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = self.archives(Path(temporary))
            api = Api()
            with patch.dict(os.environ, {"GITHUB_ACTIONS": "true", "GITHUB_REPOSITORY": r.SOURCE, "GITHUB_SHA": SHA}):
                with contextlib.redirect_stdout(io.StringIO()):
                    r.publish(api, Api(), VERSION, SHA, directory)
            writes = [(method, path) for method, path, _ in api.calls if method != "GET"]
            self.assertTrue(writes[0][1].endswith("/git/refs"))
            self.assertEqual(len(api.assets), 3)
            self.assertEqual(writes[-1], ("PATCH", PREFIX + "/releases/7"))
            self.assertTrue(next(data for method, path, data in api.calls if method == "POST" and path.endswith("/releases"))["draft"])

    def test_upload_errors_and_mismatched_server_digests_never_publish(self):
        for api in [Api(bad_digest=True), Api(fail_upload=True)]:
            with tempfile.TemporaryDirectory() as temporary:
                directory = self.archives(Path(temporary))
                with patch.dict(os.environ, {"GITHUB_ACTIONS": "true", "GITHUB_REPOSITORY": r.SOURCE, "GITHUB_SHA": SHA}):
                    with self.assertRaises((ValueError, RuntimeError)):
                        r.publish(api, Api(), VERSION, SHA, directory)
                self.assertFalse(any(method in ("PATCH", "DELETE") for method, _, _ in api.calls))

    def test_concurrent_tag_reservation_failure_never_creates_a_release(self):
        class Collision(Api):
            def request(self, method, path, payload=None, **kwargs):
                if method == "POST" and path.endswith("/git/refs"):
                    raise RuntimeError("synthetic ref already exists")
                return super().request(method, path, payload, **kwargs)
        with tempfile.TemporaryDirectory() as temporary:
            directory = self.archives(Path(temporary))
            api = Collision()
            with patch.dict(os.environ, {"GITHUB_ACTIONS": "true", "GITHUB_REPOSITORY": r.SOURCE, "GITHUB_SHA": SHA}):
                with self.assertRaises(RuntimeError):
                    r.publish(api, Api(), VERSION, SHA, directory)
            self.assertFalse(any(method == "POST" and path.endswith("/releases") for method, path, _ in api.calls))

    def test_newer_failed_ci_cannot_be_hidden_by_older_success(self):
        class Mixed:
            def request(self, *args, **kwargs):
                return {"workflow_runs": [
                    {"head_sha": SHA, "conclusion": "success", "status": "completed", "run_number": 1, "run_attempt": 1},
                    {"head_sha": SHA, "conclusion": "failure", "status": "completed", "run_number": 2, "run_attempt": 1}]}
        with patch.dict(os.environ, {"GITHUB_ACTIONS": "true", "GITHUB_REPOSITORY": r.SOURCE, "GITHUB_SHA": SHA}):
            with self.assertRaises(ValueError):
                r.preflight(Api(), Mixed(), VERSION, SHA)

    def test_server_immutability_is_required_after_publication(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = self.archives(Path(temporary))
            api = Api(immutable=False)
            with patch.dict(os.environ, {"GITHUB_ACTIONS": "true", "GITHUB_REPOSITORY": r.SOURCE, "GITHUB_SHA": SHA}), patch.object(r.time, "sleep"):
                with self.assertRaisesRegex(ValueError, "immutability"):
                    r.publish(api, Api(), VERSION, SHA, directory)
            self.assertFalse(any(method == "DELETE" for method, _, _ in api.calls))

    def test_missing_native_archive_or_extra_file_fails_before_remote_write(self):
        for extra in [True, False]:
            with tempfile.TemporaryDirectory() as temporary:
                directory = self.archives(Path(temporary))
                if extra:
                    (directory / "unexpected-secret.txt").write_text("must not upload")
                else:
                    next(directory.iterdir()).unlink()
                api = Api()
                with patch.dict(os.environ, {"GITHUB_ACTIONS": "true", "GITHUB_REPOSITORY": r.SOURCE, "GITHUB_SHA": SHA}):
                    with self.assertRaises(ValueError):
                        r.publish(api, Api(), VERSION, SHA, directory)
                self.assertTrue(all(method == "GET" for method, _, _ in api.calls))

    def test_tampered_inner_checksum_and_extra_zip_entry_fail(self):
        for name, data in [("SHA256SUMS", b"wrong"), ("../escape", b"bad")]:
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                source = root / "binary"
                source.write_bytes(b"synthetic")
                target = "x86_64-pc-windows-msvc"
                path = r.pack(source, root / "output", VERSION, SHA, target, {})
                # Rebuild with a changed member, avoiding duplicate-name warnings.
                with zipfile.ZipFile(path) as archive:
                    contents = {i.filename: archive.read(i) for i in archive.infolist()}
                contents[name] = data
                with zipfile.ZipFile(path, "w") as archive:
                    for member, value in contents.items():
                        archive.writestr(member, value)
                with self.assertRaises(ValueError):
                    r.inspect_archive(path, VERSION, SHA, target)

    def test_http_boundary_rejects_other_repositories_and_never_redirects_credentials(self):
        api = r.GitHub("synthetic-test-token")
        with self.assertRaises(ValueError):
            api.request("GET", "/repos/" + r.DESTINATION + "-other")
        self.assertIsNone(r.NoRedirect().redirect_request(None, None, 302, "", {}, "https://unrelated.example"))
        class Denied:
            def open(self, request, timeout):
                raise urllib.error.HTTPError(request.full_url, 403, "denied", {}, io.BytesIO(b"confidential-response"))
        api.opener = Denied()
        with self.assertRaises(RuntimeError) as failure:
            api.request("GET", PREFIX)
        self.assertNotIn("confidential-response", str(failure.exception))
        self.assertNotIn("synthetic-test-token", str(failure.exception))

    def test_unsuccessful_source_ci_fails_before_build_or_publication(self):
        api = Api()
        class Pending:
            def request(self, *args, **kwargs):
                return {"workflow_runs": [{"head_sha": SHA, "conclusion": None, "status": "in_progress", "run_number": 1, "run_attempt": 1}]}
        with patch.dict(os.environ, {"GITHUB_ACTIONS": "true", "GITHUB_REPOSITORY": r.SOURCE, "GITHUB_SHA": SHA}):
            with self.assertRaises(ValueError):
                r.preflight(api, Pending(), VERSION, SHA)
        self.assertTrue(all(method == "GET" for method, _, _ in api.calls))


if __name__ == "__main__":
    unittest.main()
