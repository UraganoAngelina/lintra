"""Run the exact probe outside the checkout with another working directory."""
import hashlib
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

exe = Path(sys.argv[1]).resolve(strict=True)
smoke = (Path(__file__).resolve().parents[1] / "spikes/tauri-runtime/tools/native_smoke.py").resolve(strict=True)
with tempfile.TemporaryDirectory(prefix="lintra-relocated-probe-", dir="/tmp") as rt:
    copied = Path(rt) / "lintra-runtime-spike"
    shutil.copy2(exe, copied)
    assert hashlib.sha256(copied.read_bytes()).digest() == hashlib.sha256(exe.read_bytes()).digest()
    subprocess.run([sys.executable, str(smoke), str(copied)], cwd=rt, check=True, timeout=50)
    print("RELOCATED_NATIVE_SMOKE_PASS independent_cwd embedded_assets", flush=True)
