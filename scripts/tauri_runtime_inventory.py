"""Record the personal CachyOS probe's ELF closure and owning OS packages.

Only the trusted probe and installed system binaries are passed to ldd. This
is a test/build tool, not code shipped to users and not a minimal Linux bundle.
WebKit/GTK resources and dynamically loaded platform plugins remain supplied
by complete OS packages. This profile qualifies this host, not other distros.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def command(*args):
    return subprocess.check_output(args, text=True, env={**os.environ, "LC_ALL": "C"})


exe = Path(sys.argv[1]).resolve(strict=True)
out = Path(sys.argv[2])
assert not out.exists(), "Evidence must not be overwritten"
roots = [exe, *map(Path, [
    "/usr/lib/webkit2gtk-4.1/WebKitWebProcess",
    "/usr/lib/webkit2gtk-4.1/WebKitNetworkProcess",
    "/usr/lib/webkit2gtk-4.1/WebKitGPUProcess",
    "/usr/lib/webkit2gtk-4.1/injected-bundle/libwebkit2gtkinjectedbundle.so",
    "/usr/bin/bwrap",
    "/usr/bin/xdg-dbus-proxy",
])]
paths = set()
closures = {}
for root in roots:
    listing = command("ldd", str(root))
    if "not found" in listing:
        raise RuntimeError(f"Missing dependency for {root}")
    closures[str(root)] = listing.splitlines()
    paths.add(root.resolve(strict=True))
    for line in listing.splitlines():
        match = re.search(r"(?:=>\s+|^\s*)(/\S+)\s+\(", line)
        if match:
            paths.add(Path(match[1]).resolve(strict=True))

system_paths = sorted(str(p) for p in paths if p != exe)
ownership = command("pacman", "-Qo", "--", *system_paths)
packages = {}
for line in ownership.splitlines():
    match = re.fullmatch(r"(.+) is owned by (\S+) (\S+)", line)
    if not match:
        raise RuntimeError("Unrecognized package ownership metadata")
    _, name, version = match.groups()
    packages[name] = {"version": version}
for name, metadata in packages.items():
    info = command("pacman", "-Qi", name)
    for line in info.splitlines():
        if line.startswith("Licenses"):
            metadata["licenses_as_reported_by_OS"] = line.split(":", 1)[1].strip()
            break

dynamic = command("readelf", "-d", str(exe))
if re.search(r"\((RPATH|RUNPATH)\)", dynamic):
    raise RuntimeError("Unexpected embedded runtime search path")
record = {
    "scope": "CachyOS personal build/smoke only; not an official Linux GUI release",
    "binary_sha256": digest(exe),
    "source_git_head": command("git", "rev-parse", "HEAD").strip(),
    "os_release": Path("/etc/os-release").read_text().splitlines(),
    "machine": command("uname", "-m").strip(),
    "root_ELFs": closures,
    "ELF_files": {str(p): {"sha256": digest(p), "bytes": p.stat().st_size} for p in sorted(paths)},
    "owning_OS_packages": dict(sorted(packages.items())),
    "ownership_evidence": ownership.splitlines(),
    "binary_dynamic_section": dynamic.splitlines(),
    "missing_libraries": [],
    "RPATH_RUNPATH": "absent",
    "deployment_contract": "Keep the complete recorded OS packages installed, including WebKit/GTK resources; rebuild and rerun the native smoke after host ABI/runtime changes.",
    "limits": "ELF closure includes WebKit auxiliaries and sandbox helpers. Late-loaded driver/media/accessibility plugins depend on the installed desktop; this does not certify a hand-extracted library bundle or another host.",
}
out.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"ELF_files": len(paths), "OS_packages": len(packages), "missing": 0,
                  "binary_sha256": record["binary_sha256"], "profile_sha256": digest(out)}))
