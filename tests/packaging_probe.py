"""T-P15-07: exercise real native releases, then compare unmodified JSONL bytes.

Only synthetic data is used. Temporary installs isolate executable-relative
resolution from the checkout and caller cwd; no test-only runtime hooks exist.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import zipfile


CASES = {"embedded.jsonl", "external.jsonl", "external-override.jsonl"}


def artifact(log):
    """xtask builds in unique clean trees; never guess a stale target path."""
    paths = [line.removeprefix("artifact=") for line in log.read_text(encoding="utf-8").splitlines() if line.startswith("artifact=")]
    if len(paths) != 1 or not Path(paths[0]).is_file():
        raise ValueError("build log must identify exactly one existing artifact")
    return Path(paths[0])


def jsonl(data):
    """Reject encoding/newline regressions before comparing exact raw bytes."""
    data.decode("utf-8")
    if data.startswith(b"\xef\xbb\xbf") or b"\r" in data or (data and not data.endswith(b"\n")):
        raise ValueError("JSONL must be UTF-8 without BOM and use final LF")
    for line in data.splitlines():
        if not isinstance(json.loads(line), dict):
            raise ValueError("each JSONL line must be an object")


def compare(left, right):
    """Exact inventory prevents two missing/empty artifact sets from passing."""
    for root in (left, right):
        if {p.name for p in root.iterdir()} != CASES:
            raise ValueError("cross-OS artifact inventory differs")
    for name in sorted(CASES):
        a, b = (left / name).read_bytes(), (right / name).read_bytes()
        jsonl(a)
        jsonl(b)
        if a != b:
            raise ValueError(f"cross-OS JSONL differs: {name}")
    if not (left / "embedded.jsonl").read_bytes():
        raise ValueError("the diagnostic scenario must produce errors")


def produce(binary, bundle, output):
    """Use a qualified bundle archive and the actual feature-enabled release.

    First run without external directories proves embedded fallback. Extracting
    the real bundle beside the copied executable then proves level 3 without
    CLI/env overrides. A changed, rehashed synthetic constraint proves external
    precedence by changing the validation result, rather than just an audit flag.
    """
    binary, bundle, output = binary.resolve(), bundle.resolve(), output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    with tempfile.TemporaryDirectory(prefix="lintra packaging ") as temporary:
        root = Path(temporary)
        install, work = root / "bundle with spaces", root / "different working directory"
        install.mkdir()
        work.mkdir()
        executable = install / binary.name
        shutil.copy2(binary, executable)
        original = hashlib.sha256(executable.read_bytes()).digest()
        env = {k: v for k, v in os.environ.items() if k not in ("LINTRA_SCHEMA_DIR", "LINTRA_REFERENCE_DATA_DIR")}

        def run(data, code, source=None):
            (work / "input.txt").write_bytes(data)
            for name in ("errors.jsonl", "summary.json"):
                (work / name).unlink(missing_ok=True)
            result = subprocess.run([
                str(executable), "validate", "--flow", "SYNTHETIC", "--schema-version", "v1",
                "--trace", "T1", "--encoding", "utf8", "--input", "input.txt",
                "--errors", "errors.jsonl", "--summary", "summary.json",
            ], cwd=work, env=env, capture_output=True, timeout=60)
            if result.returncode != code:
                raise ValueError(f"unexpected CLI exit: {result.returncode}, expected {code}: {result.stderr!r}")
            if source is None:
                if any((work / name).exists() for name in ("errors.jsonl", "summary.json")):
                    raise ValueError("failed preflight published reports")
                return None
            summary = json.loads((work / "summary.json").read_bytes())
            if summary["bundle_source"] != source:
                raise ValueError("unexpected bundle source")
            errors = (work / "errors.jsonl").read_bytes()
            jsonl(errors)
            return errors, summary

        # UTF-8 accented identity, invalid logical value, short/long records,
        # multiple diagnostics and CRLF input exercise actual serializer bytes.
        sample = "002É \r\n001Z  \r\nABC\r\n1234567\r\n".encode()
        embedded, _ = run(sample, 2, "embedded")
        if not embedded or b"\xc3\x89" not in embedded:
            raise ValueError("diagnostic scenario did not exercise UTF-8 identity")
        (output / "embedded.jsonl").write_bytes(embedded)
        # Archive is produced locally by the qualified Rust bundler; it is not
        # an arbitrary remote archive. Its validation precedes this extraction.
        with zipfile.ZipFile(bundle) as archive:
            archive.extractall(install)
        external, audit = run(sample, 2, "external")
        if external != embedded or audit["bundle_manifest_status"] != "present":
            raise ValueError("extracted bundle changed errors or lost manifest audit")
        (output / "external.jsonl").write_bytes(external)
        schema = install / "schemas/SYNTHETIC/v1/flow.json"
        raw = json.loads(schema.read_bytes())
        raw["traces"][0]["fields"][0]["constraints"][0]["values"] = ["2"]
        schema.write_bytes((json.dumps(raw, sort_keys=True) + "\n").encode())
        manifest = install / "MANIFEST.json"
        raw_manifest = json.loads(manifest.read_bytes())
        raw_manifest["bundle_version"] = "synthetic-cross-os-override-v1"
        raw_manifest["files"]["schemas/SYNTHETIC/v1/flow.json"] = hashlib.sha256(schema.read_bytes()).hexdigest()
        manifest.write_bytes((json.dumps(raw_manifest, sort_keys=True) + "\n").encode())
        errors, audit = run("002É \n".encode(), 0, "external")
        if errors or audit["bundle_version"] != "synthetic-cross-os-override-v1":
            raise ValueError("external constraint did not override embedded snapshot")
        # This record was a violation in the embedded run, now it is valid.
        (output / "external-override.jsonl").write_bytes(errors)
        schema.write_bytes(b"{")
        run(b"001A  \n", 1)  # A broken external bundle must never silently fall back.
        if hashlib.sha256(executable.read_bytes()).digest() != original:
            raise ValueError("data update changed the executable")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="operation", required=True)
    producer = sub.add_parser("produce")
    source = producer.add_mutually_exclusive_group(required=True)
    source.add_argument("--binary", type=Path)
    source.add_argument("--build-log", type=Path)
    for name in ("bundle", "output"):
        producer.add_argument("--" + name, required=True, type=Path)
    comparator = sub.add_parser("compare")
    for name in ("left", "right"):
        comparator.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    if args.operation == "produce":
        produce(args.binary if args.binary else artifact(args.build_log), args.bundle, args.output)
    else:
        compare(args.left, args.right)
    print("packaging cross-OS probe: PASS")


if __name__ == "__main__":
    main()
