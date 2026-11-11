"""Capture only this synthetic probe's window on the personal Hyprland host.

The child GUI is owned and joined by this foreground harness. Other desktop
windows are neither printed nor captured. Requires already installed hyprctl
and grim; neither is a product dependency or a portable GUI test runner.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import select
import subprocess
import sys
import tempfile
import time


def hypr_json(command):
    return json.loads(subprocess.check_output(["hyprctl", "-j", command]))


exe = Path(sys.argv[1]).resolve(strict=True)
output = Path(sys.argv[2]).resolve()
assert not output.exists(), "Do not overwrite capture evidence"
print("binary_sha256", hashlib.sha256(exe.read_bytes()).hexdigest(), flush=True)

with tempfile.TemporaryDirectory(prefix="lintra-native-capture-", dir="/tmp") as rt:
    env = os.environ.copy()
    for variable, dirname in [
        ("XDG_DATA_HOME", "data"),
        ("XDG_CACHE_HOME", "cache"),
        ("XDG_CONFIG_HOME", "config"),
    ]:
        dest = Path(rt) / dirname
        dest.mkdir(mode=0o700)
        env[variable] = str(dest)
    env["GDK_BACKEND"] = "wayland"
    process = subprocess.Popen(
        [str(exe)], env=env, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT, text=True,
    )
    try:
        deadline = time.monotonic() + 30
        acknowledged = False
        while time.monotonic() < deadline and process.poll() is None:
            ready, _, _ = select.select([process.stdout], [], [], 0.5)
            if not ready:
                continue
            line = process.stdout.readline()
            print(line, end="", flush=True)
            if line.startswith("LINTRA_RUNTIME_RENDER_ACK "):
                acknowledged = True
                break
        assert acknowledged, "No actual native render acknowledgment"
        # Filter before reporting anything: never log other application titles.
        clients = [w for w in hypr_json("clients") if w.get("pid") == process.pid]
        assert len(clients) == 1, "Expected one window owned by the probe process"
        window = clients[0]
        assert window["title"] == "Lintra · Verifica runtime"
        assert re.fullmatch(r"0x[0-9a-fA-F]+", window["address"])
        address = json.dumps("address:" + window["address"])
        # Hyprland 0.57 uses Lua dispatchers; old focuswindow/closewindow syntax
        # is rejected. The validated address confines both calls to our child.
        subprocess.run([
            "hyprctl", "dispatch", f"hl.dsp.focus({{window = {address}}})",
        ], check=True)
        time.sleep(0.4)
        active = hypr_json("activewindow")
        assert active.get("pid") == process.pid, "Probe must own the focused window"
        x, y = active["at"]
        width, height = active["size"]
        assert width >= 640 and height >= 480
        # Clip to this window, never to the monitor or the full desktop.
        subprocess.run(
            ["grim", "-g", f"{x},{y} {width}x{height}", str(output)], check=True,
        )
        print("capture", json.dumps({
            "class": active["class"], "title": active["title"],
            "size": [width, height], "sha256": hashlib.sha256(output.read_bytes()).hexdigest(),
        }), flush=True)
        subprocess.run([
            "hyprctl", "dispatch", f"hl.dsp.window.close({{window = {address}}})",
        ], check=True)
        remaining, _ = process.communicate(timeout=10)
        print(remaining, end="", flush=True)
        assert process.returncode == 0, process.returncode
    finally:
        if process.poll() is None:
            process.terminate()
            process.communicate(timeout=10)
