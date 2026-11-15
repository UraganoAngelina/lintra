"""Foreground synthetic React/raw-IPC smoke; owns and joins the native process."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

binary = Path(sys.argv[1]).resolve(strict=True)
with tempfile.TemporaryDirectory(prefix="lintra-desktop-smoke-") as temporary:
    env = os.environ.copy()
    for key, name in [("XDG_DATA_HOME", "data"), ("XDG_CONFIG_HOME", "config"),
                      ("XDG_CACHE_HOME", "cache")]:
        directory = Path(temporary) / name
        directory.mkdir(mode=0o700)
        env[key] = str(directory)
    result = subprocess.run([str(binary), "--smoke-test"], env=env, cwd=temporary,
                            capture_output=True, text=True, timeout=45)
    print(json.dumps({"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                      "exit_code": result.returncode, "stdout": result.stdout,
                      "stderr": result.stderr}), flush=True)
    assert result.returncode == 0, "native process failed"
    assert result.stdout.splitlines().count("LINTRA_DESKTOP_READY") == 1, "missing React/IPC ack"
    print("NATIVE_DESKTOP_SMOKE_PASS", flush=True)
