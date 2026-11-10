"""Measure the existing public synthetic CAD83 runner, sequentially on Linux.

Build separately with the release-cad83-build command in the gate ledger. Only
the direct test process is measured here; compiler/Cargo memory is excluded.
The test's six sample timings exclude fixture construction. wait4 RSS/CPU/IO
include the entire test process, including fixture construction and cleanup.
"""
from datetime import datetime, timezone
import hashlib
import gzip
import json
import os
from pathlib import Path
import platform
import subprocess
import time

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent


def main():
    if platform.system() != "Linux":
        raise SystemExit("This wait4/RSS/IO measurement is Linux-specific")
    artifacts = []
    build_log = OUT / "release-cad83-build.log"
    build_text = (build_log.read_text() if build_log.is_file() else
                  gzip.decompress(build_log.with_suffix(".log.gz").read_bytes()).decode())
    for line in build_text.splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if (event.get("reason") == "compiler-artifact"
                and event.get("target", {}).get("name") == "cad_history_runtime"
                and event.get("executable")):
            artifacts.append(Path(event["executable"]))
    if len(artifacts) != 1:
        raise SystemExit("Expected exactly one release test executable from Cargo")
    executable = artifacts[0]
    inputs = json.loads((OUT / "qualified-inputs.json").read_text())
    changed = [name for name, digest in inputs.items()
               if hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != digest]
    if changed:
        raise SystemExit(f"Qualified sources changed: {changed}")
    result = {
        "date": "2026-11-10", "host": platform.platform(),
        "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "executable_sha256": hashlib.sha256(executable.read_bytes()).hexdigest(),
        "build_profile": "release", "units": "RSS KiB; Linux IO blocks of 512 bytes",
        "thresholds": None, "measurements": [],
    }
    if (OUT / "performance.json").exists() or any(
            (OUT / f"perf-cad83-{count}{suffix}").exists()
            for count in [100, 300] for suffix in [".log", ".log.gz"]):
        raise SystemExit("Preserve previous measurement outputs before a new run")
    for count in [100, 300]:
        log_path = OUT / f"perf-cad83-{count}.log"
        command = [str(executable), "measure_full_cad83_history_application",
                   "--exact", "--ignored", "--nocapture"]
        environment = dict(os.environ, LINTRA_CAD83_EVENT_COUNT=str(count))
        started = time.monotonic()
        with log_path.open("xb") as output:
            process = subprocess.Popen(command, cwd=ROOT, env=environment,
                                       stdout=output, stderr=subprocess.STDOUT)
            _, status, usage = os.wait4(process.pid, 0)
            process.returncode = os.waitstatus_to_exitcode(status)
        elapsed = time.monotonic() - started
        if process.returncode != 0:
            raise SystemExit(f"CAD83 measurement failed: {log_path.name}")
        observations = [json.loads(line.removeprefix("CAD83_MEASUREMENT "))
                        for line in log_path.read_text().splitlines()
                        if line.startswith("CAD83_MEASUREMENT ")]
        if len(observations) != 1 or observations[0]["approved_snapshots"] != 6:
            raise SystemExit("Missing complete six-approval measurement")
        sample = dict(observations[0], command=command, exit=process.returncode,
                      wall_seconds=elapsed, peak_rss_kib=usage.ru_maxrss,
                      user_cpu_seconds=usage.ru_utime, system_cpu_seconds=usage.ru_stime,
                      input_blocks=usage.ru_inblock, output_blocks=usage.ru_oublock,
                      log=log_path.name,
                      log_sha256=hashlib.sha256(log_path.read_bytes()).hexdigest())
        result["measurements"].append(sample)
        print(json.dumps(sample))
    result["completed_utc"] = datetime.now(timezone.utc).isoformat()
    result["sources_unchanged"] = all(
        hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == digest
        for name, digest in inputs.items())
    with (OUT / "performance.json").open("x") as output:
        json.dump(result, output, indent=2)
        output.write("\n")


if __name__ == "__main__":
    main()
