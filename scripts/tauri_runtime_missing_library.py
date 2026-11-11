"""Negative loader test in a private mount namespace; never alter host libs."""
import hashlib
from pathlib import Path
import subprocess
import sys

exe = Path(sys.argv[1]).resolve(strict=True)
library = Path("/usr/lib/libwebkit2gtk-4.1.so.0").resolve(strict=True)
before = hashlib.sha256(library.read_bytes()).hexdigest()
result = subprocess.run([
    "bwrap", "--ro-bind", "/", "/", "--dev", "/dev", "--proc", "/proc",
    "--unshare-net", "--ro-bind", "/dev/null", str(library),
    "--die-with-parent", str(exe), "--smoke-test",
], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, timeout=15)
print(result.stdout, end="", flush=True)
print("isolated_loader_exit", result.returncode, flush=True)
assert result.returncode == 127, "Expected loader failure, not a namespace setup failure"
# Depending on mount/device policy the masked object yields EACCES or an
# invalid ELF error. The contract is the loader naming this exact dependency,
# exit 127 and no application start; a generic bubblewrap failure cannot pass.
assert "error while loading shared libraries: libwebkit2gtk-4.1.so.0:" in result.stdout
assert "LINTRA_RUNTIME_RENDER_ACK" not in result.stdout, "No simulated runtime fallback"
assert hashlib.sha256(library.read_bytes()).hexdigest() == before, "Host library changed"
print("MISSING_WEBKIT_NEGATIVE_PASS host_library_unchanged", before, flush=True)
