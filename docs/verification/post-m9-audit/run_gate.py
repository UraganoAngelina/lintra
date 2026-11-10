"""Run one foreground gate and retain its exact output, exit and source identity.

Usage from the repository root: python3 .../run_gate.py LABEL COMMAND ARG...
This wrapper never turns ignored tests into PASS. Labels must be new, so earlier
attempts and their logs cannot be overwritten.
"""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent


def head():
    return subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
    ).strip()


def check_inputs():
    manifest = json.loads((OUT / "qualified-inputs.json").read_text())
    return [name for name, expected in manifest.items()
            if hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != expected]


def main():
    label, *command = sys.argv[1:]
    if not label or Path(label).name != label or not command:
        raise SystemExit("Expected a simple new label and an explicit command")
    result_path = OUT / (label + ".json")
    log_path = OUT / (label + ".log")
    if log_path.exists() or result_path.exists():
        raise SystemExit("Use a new label; previous evidence is preserved")
    before = head()
    changed_before = check_inputs()
    start = time.monotonic()
    with log_path.open("xb") as output:
        result = subprocess.run(command, cwd=ROOT, stdout=output, stderr=subprocess.STDOUT)
    record = {
        "label": label, "command": command, "exit": result.returncode,
        "seconds": round(time.monotonic() - start, 3),
        "completed_utc": datetime.now(timezone.utc).isoformat(),
        "head_before": before, "head": head(),
        "changed_qualified_inputs_before": changed_before,
        "changed_qualified_inputs_after": check_inputs(),
        "log_sha256": hashlib.sha256(log_path.read_bytes()).hexdigest(),
    }
    result_path.write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps(record))
    print("\n".join(log_path.read_text(errors="replace").splitlines()[-12:]))
    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
