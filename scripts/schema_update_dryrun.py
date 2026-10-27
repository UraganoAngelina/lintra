"""Rehearse a synthetic data-only update with the real CLI and Rust bundler.

Never edits source bundles, changes an executable or publishes a remote release.
The caller supplies an existing feature-enabled native release and a NEW directory.
All cases and artifacts remain there for review; no silent cleanup or overwrite.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import zipfile


def sha(data):
    return hashlib.sha256(data).hexdigest()


def save(path, value):
    path.write_bytes((json.dumps(value, indent=2, ensure_ascii=False) + "\n").encode())


def dryrun(binary, work):
    repo = Path(__file__).resolve().parents[1]
    binary = binary.resolve(strict=True)
    original_hash = sha(binary.read_bytes())
    work = work.resolve()
    work.mkdir(parents=True, exist_ok=False)
    app, cwd = work / "application with spaces", work / "working directory"
    app.mkdir()
    cwd.mkdir()
    executable = app / binary.name
    shutil.copy2(binary, executable)
    # Prevent a developer's configured bundle overriding this isolated rehearsal.
    env = {k: v for k, v in os.environ.items() if k.upper() not in
           {"LINTRA_SCHEMA_DIR", "LINTRA_REFERENCE_DATA_DIR"}}
    commands = []

    def call(args, expected, directory=cwd):
        result = subprocess.run([str(x) for x in args], cwd=directory, env=env,
                                capture_output=True, timeout=180)
        commands.append({"args": [str(x) for x in args], "exit": result.returncode})
        if result.returncode != expected:
            raise ValueError(f"unexpected exit {result.returncode}: {result.stderr!r}")
        return result

    counter = 0

    def validate(version, data, expected, bundle_source):
        nonlocal counter
        counter += 1
        name = f"case-{counter}"
        (cwd / "input.txt").write_bytes(data)
        call([executable, "validate", "--flow", "SYNTHETIC", "--schema-version", version,
              "--trace", "T1", "--encoding", "utf8", "--input", "input.txt",
              "--errors", name + ".jsonl", "--summary", name + ".json"], expected)
        summary = json.loads((cwd / (name + ".json")).read_bytes())
        errors = (cwd / (name + ".jsonl")).read_bytes()
        # Independent expected semantics, not a golden generated from actual output.
        assert summary["result"] == ("VALID" if expected == 0 else "INVALID")
        assert summary["records_processed"] == 1
        assert summary["total_errors"] == (0 if expected == 0 else 1)
        assert summary["bundle_source"] == bundle_source
        assert bool(errors) == (expected != 0)
        return errors, summary

    # Establish a real pre-update oracle, including a value absent from v1.
    call([executable, "--version"], 0)
    validate("v1", b"001A  \n", 0, "embedded")
    original_errors, original_summary = validate("v1", b"002A  \n", 2, "embedded")
    candidate = work / "candidate"
    candidate.mkdir()
    old = json.loads((repo / "bundles/embedded/MANIFEST.json").read_bytes())
    for name in old["files"]:
        target = candidate / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(repo / "bundles/embedded" / name, target)
    schema = json.loads((candidate / "schemas/SYNTHETIC/v1/flow.json").read_bytes())
    schema["schema_version"] = "v2"
    schema["regulatory_version"] = "synthetic-update-v2"
    schema["traces"][0]["fields"][0]["constraints"][0]["values"] = ["1", "2"]
    new_path = candidate / "schemas/SYNTHETIC/v2/flow.json"
    new_path.parent.mkdir()
    save(new_path, schema)
    payloads = sorted(p for p in candidate.rglob("*") if p.is_file())
    manifest = {"bundle_version": "synthetic-runbook-v2", "schema_format_version": 1,
                "files": {p.relative_to(candidate).as_posix(): sha(p.read_bytes())
                          for p in payloads}}
    save(candidate / "MANIFEST.json", manifest)
    call([executable, "schema", "lint", "--flow", "SYNTHETIC", "--schema-version", "v2",
          "--schema-dir", candidate / "schemas", "--encoding", "utf8"], 0)
    # Package data only: xtask compiles tooling if needed, never builds a new CLI.
    dist = work / "dist"
    dist.mkdir()
    call(["cargo", "xtask", "bundle-data", "--input", candidate, "--output-dir", dist],
         0, repo)
    archive = dist / "schemas-synthetic-runbook-v2.zip"
    with zipfile.ZipFile(archive) as zipped:
        expected = set(manifest["files"]) | {"schemas/", "reference-data/",
                                             "MANIFEST.json", "SHA256SUMS"}
        assert len(zipped.namelist()) == len(expected)
        assert set(zipped.namelist()) == expected
        assert zipped.read("MANIFEST.json") == (candidate / "MANIFEST.json").read_bytes()
        for name, digest in manifest["files"].items():
            assert sha(zipped.read(name)) == digest
        for line in zipped.read("SHA256SUMS").decode().splitlines():
            digest, name = line.split("  ", 1)
            assert sha(zipped.read(name)) == digest
        # Inventory was checked before extracting this locally generated fixture.
        zipped.extractall(app)
    same_errors, same_summary = validate("v1", b"002A  \n", 2, "external")
    assert same_errors == original_errors
    assert same_summary["schema_hash"] == original_summary["schema_hash"]
    _, updated = validate("v2", b"002A  \n", 0, "external")
    invalid, rejected = validate("v2", b"003A  \n", 2, "external")
    invalid_repeat, _ = validate("v2", b"003A  \n", 2, "external")
    assert invalid == invalid_repeat
    assert updated["bundle_version"] == rejected["bundle_version"] == manifest["bundle_version"]
    assert updated["schema_hash"] != original_summary["schema_hash"]
    assert sha(executable.read_bytes()) == sha(binary.read_bytes()) == original_hash
    evidence = {"task": "T-P18-02", "synthetic_only": True, "remote_published": False,
                "binary_sha256_before_after": original_hash,
                "archive_sha256": sha(archive.read_bytes()), "bundle_version": manifest["bundle_version"],
                "schema_hash_v1": original_summary["schema_hash"],
                "schema_hash_v2": updated["schema_hash"], "commands": commands,
                "v1_jsonl_preserved": True, "v2_jsonl_deterministic": True,
                "new_version_valid_invalid_checked": True}
    save(work / "evidence.json", evidence)
    print(work / "evidence.json")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--work-dir", type=Path, required=True)
    options = parser.parse_args()
    dryrun(options.binary, options.work_dir)
