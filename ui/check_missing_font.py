"""Run the ACTUAL Cargo build script against an isolated source copy missing a font.

The original UI/assets/lock are never modified. Dependency artifacts can be reused;
the failing package has a distinct path and its own temporary manifest/lock.
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

repo = Path(__file__).resolve().parents[1]
ui = repo / "ui"
with tempfile.TemporaryDirectory(prefix="lintra-missing-font-") as scratch:
    target = Path(scratch) / "ui"
    target.mkdir()
    for name in ("src", "ui", "build_support"):
        shutil.copytree(ui / name, target / name)
    shutil.copyfile(ui / "build.rs", target / "build.rs")
    # The UI now shares the parent's provenance script. Preserve that source
    # relationship so failure still demonstrates the missing face, not a module.
    shutil.copyfile(repo / "build.rs", Path(scratch) / "build.rs")
    shutil.copyfile(repo / "Cargo.lock", target / "Cargo.lock")
    manifest = (ui / "Cargo.toml").read_text()
    manifest = manifest.replace('path = ".."', 'path = ' + json.dumps(str(repo)))
    (target / "Cargo.toml").write_text(manifest + '\n[workspace]\nresolver = "3"\n')
    (target / "ui/fonts/IBMPlexMono-SemiBold.ttf").unlink()
    env = dict(os.environ, CARGO_TARGET_DIR=str(repo / "target"))
    command = ["cargo", "check", "--manifest-path", str(target / "Cargo.toml"), "--offline"]
    if native_target := env.get("LINTRA_FONT_CHECK_TARGET"):
        command.extend(["--target", native_target])
    result = subprocess.run(
        command,
        cwd=repo, env=env, capture_output=True, timeout=300,
    )
    if result.returncode == 0 or b"required embedded IBM Plex fonts are missing or invalid" not in result.stderr:
        raise RuntimeError(f"missing-font build contract not demonstrated: {result.stderr!r}")
print("actual Cargo build rejects a missing embedded face; original fonts untouched")
