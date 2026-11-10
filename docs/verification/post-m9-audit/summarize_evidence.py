"""Validate retained evidence and summarize real outcomes without counting ignored as PASS.

This is an audit artifact checker, not a product test or a replacement for CI.
It accepts storage-red's two expected failures as a reproduction, and keeps
pending native checks visibly pending. Rerun after the final native read.
"""
from datetime import datetime, timezone
import hashlib
import gzip
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
REPORTS = ["LINTRA_RETROSPECTIVE_AUDIT_REPORT.md", "LINTRA_POST_M9_GAP_REGISTER.md",
           "LINTRA_POST_M9_ENGINEERING_ROADMAP.md"]
SHA = "af3bcccb6246a7ff00463c595f4c780a9f5c5195"
REQUIRED_JOBS = {
    "Verify (x86_64-unknown-linux-gnu)", "Verify (x86_64-pc-windows-msvc)",
    "Compare native desktop theme pixels", "Compare production UI regression pixels",
    "Compare Linux and Windows JSONL bytes",
    "Reopen counterpart snapshot (x86_64-unknown-linux-gnu)",
    "Reopen counterpart snapshot (x86_64-pc-windows-msvc)",
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    failures = []
    gates = []
    for path in sorted(OUT.glob("*.json")):
        value = json.loads(path.read_text())
        if not isinstance(value, dict) or not {"label", "command", "exit"} <= value.keys():
            continue
        log = OUT / (value["label"] + ".log")
        archived = log.with_suffix(".log.gz")
        raw_log = (log.read_bytes() if log.is_file() else
                   gzip.decompress(archived.read_bytes()) if archived.is_file() else b"")
        valid = (log.is_file() or archived.is_file()) and hashlib.sha256(raw_log).hexdigest() == value["log_sha256"]
        if not valid:
            failures.append(f"Log missing or changed: {value['label']}")
        expected_failure = value["label"] == "storage-red"
        instrumentation_failure = value["label"] == "cad83-performance"
        expected_exit = 101 if expected_failure else (1 if instrumentation_failure else 0)
        if value["exit"] != expected_exit:
            failures.append(f"Unexpected exit: {value['label']}")
        text = raw_log.decode(errors="replace")
        summaries = [tuple(map(int, row)) for row in re.findall(
            r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored", text)]
        value["log_verified"] = valid
        if archived.is_file():
            value["archived_log"] = archived.name
            value["archived_log_sha256"] = digest(archived)
        value["role"] = ("expected_failure_reproduction" if expected_failure else
                         "measurement_parser_failure_not_qualified" if instrumentation_failure
                         else "verification")
        value["rust_summary_count"] = len(summaries)
        value["rust_outcomes_including_child_summaries"] = (
            dict(zip(["passed", "failed", "ignored"], map(sum, zip(*summaries))))
            if summaries else None)
        gates.append(value)
    manifest = json.loads((OUT / "qualified-inputs.json").read_text())
    changed = [name for name, expected in manifest.items() if digest(ROOT / name) != expected]
    failures.extend(f"Qualified source changed: {name}" for name in changed)
    broken = []
    # Check only local file links, excluding anchors and URLs.
    for name in REPORTS:
        for destination in re.findall(r"\[[^\]]+\]\(([^)]+)\)", (ROOT / name).read_text()):
            destination = destination.strip("<>").split("#", 1)[0]
            if destination and "://" not in destination and not (ROOT / destination).is_file():
                broken.append((name, destination))
    failures.extend(f"Broken report link: {name} -> {destination}" for name, destination in broken)
    previous_readme = subprocess.check_output(["git", "show", "bfe51ac:README.md"], cwd=ROOT, text=True)
    current_readme = (ROOT / "README.md").read_text()
    for language in ["bash", "powershell"]:
        pattern = rf"<!-- lintra-quickstart:{language} -->(.*?)<!-- /lintra-quickstart -->"
        if re.findall(pattern, previous_readme, re.S) != re.findall(pattern, current_readme, re.S):
            failures.append(f"Quickstart changed: {language}")
    product_diff = subprocess.check_output(
        ["git", "diff", SHA, "--", "src", "ui", "tests", "scripts", "xtask", "benches",
         "Cargo.toml", "Cargo.lock", ".github", "docs/migration"], cwd=ROOT, text=True)
    if product_diff:
        failures.append("Product or migration files changed after the qualified code commit")
    native_path = OUT / "native-ci-final.json"
    native = {"status": "pending", "code_sha": SHA, "run": 42789338814}
    if native_path.is_file():
        run = json.loads(native_path.read_text())
        jobs = {job["name"]: job for job in run["jobs"]}
        complete = (run["status"] == "completed" and run["conclusion"] == "success"
                    and run["headSha"] == SHA and REQUIRED_JOBS <= jobs.keys()
                    and all(jobs[name]["conclusion"] == "success" for name in REQUIRED_JOBS))
        native = dict(status="pass" if complete else "not_qualified", code_sha=run["headSha"],
                      run=run["databaseId"], required_jobs=len(REQUIRED_JOBS),
                      jobs={name: job["conclusion"] for name, job in jobs.items()})
        if not complete:
            failures.append("Native CI has not qualified the exact code SHA")
        evidence = json.loads((OUT / "native-log-verification.json").read_text())
        for host, record in evidence["native"].items():
            archive = OUT / record["compressed_log"]
            raw = gzip.decompress(archive.read_bytes())
            if (digest(archive) != record["compressed_sha256"]
                    or hashlib.sha256(raw).hexdigest() != record["raw_log_sha256"]):
                failures.append(f"Native log changed: {host}")
            for name in ["audit_generation_read_is_coherent_across_concurrent_publication",
                         "audit_identity_lookup_rejects_dangling_latest"]:
                if raw.count(("test persistence::sqlite::tests::" + name + " ... ok").encode()) != 3:
                    failures.append(f"Missing native regression/configuration: {host}/{name}")
        imports = list((OUT / "native-imports").rglob("*.json"))
        if len(imports) != 2:
            failures.append("Expected two actual opposite-OS import results")
        directions = set()
        for path in imports:
            imported = json.loads(path.read_text())
            directions.add((imported["producer_os"], imported["consumer_os"]))
            if (imported["snapshots_verified"] != 4 or imported["namespaces_verified"] != 2
                    or imported["logical_fact_formats_verified"] != [1, 2]
                    or not all(imported[key] for key in ["history_batch_verified",
                        "snapshot_manifests_verified", "nullable_format2_verified",
                        "protection_verified", "retained_occurrence_verified"])):
                failures.append(f"Incomplete actual native import: {path.name}")
        if directions != {("linux", "windows"), ("windows", "linux")}:
            failures.append("Native imports do not cover both directions")
        native["native_logs_and_actual_imports_verified"] = not failures
    result = dict(captured_utc=datetime.now(timezone.utc).isoformat(), code_sha=SHA,
                  qualified_input_count=len(manifest), changed_qualified_inputs=changed,
                  report_links_valid=not broken, product_and_migration_unchanged=not product_diff,
                  gates=gates, native=native, integrity_failures=failures)
    (OUT / "gates.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(dict(gates=len(gates), native=native, integrity_failures=failures), indent=2))
    if failures:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
