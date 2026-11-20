"""Native shell captures and keyboard interaction on Hyprland (T-MIG-P5-01).

Personal CachyOS evidence, not a CI or release gate. Every scenario starts its
own process with isolated XDG directories, acts only on the window owned by
that PID (Hyprland dispatchers always receive ``window=address:...``; keys go
through ``send_shortcut`` to that window, no global input device), captures
that window's area with grim, then terminates and joins the process.

Usage: python3 tests/native_capture.py <binary> <output-dir>
Writes <scenario>.png and receipt.json with binary/image SHA-256.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

# name, logical window size, keys sent to the owned window.
# Tab order: skip link, Analisi, Risultati, Informazioni, theme radio group.
# The session theme of this PC is dark; light is exercised by the explicit
# "Chiaro" choice. 1920x1080 cannot fit the work area beside the panel, so
# the largest scenario uses the whole work area instead.
SCENARIOS = [
    ("analisi-640x480", (640, 480), []),
    ("analisi-1024x768", (1024, 768), []),
    ("analisi-1366x768", (1366, 768), []),
    ("analisi-workarea", None, []),
    ("keyboard-skip-link-1024x768", (1024, 768), ["Tab"]),
    ("keyboard-risultati-1366x768", (1366, 768), ["Tab", "Tab", "Tab", "Return"]),
    ("keyboard-informazioni-640x480", (640, 480), ["Tab", "Tab", "Tab", "Tab", "Return"]),
    ("keyboard-theme-light-1366x768", (1366, 768), ["Tab"] * 5 + ["Down"]),
    ("keyboard-theme-light-640x480", (640, 480), ["Tab"] * 5 + ["Down"]),
    ("keyboard-theme-dark-1366x768", (1366, 768), ["Tab"] * 5 + ["Down", "Down"]),
]


def hypr(expression):
    result = subprocess.run(["hyprctl", "dispatch", expression], capture_output=True, text=True, timeout=10)
    if result.returncode != 0 or result.stdout.strip() != "ok":
        raise RuntimeError(f"dispatch failed: {expression}: {result.stdout}{result.stderr}")


def work_area():
    """Focused monitor area minus reserved space (panels, gaps)."""
    monitors = json.loads(subprocess.run(["hyprctl", "monitors", "-j"], capture_output=True,
                                         text=True, timeout=10, check=True).stdout)
    monitor = next(item for item in monitors if item["focused"])
    left, top, right, bottom = monitor["reserved"]
    return ((monitor["x"] + left, monitor["y"] + top),
            (monitor["width"] - left - right, monitor["height"] - top - bottom))


def owned_window(pid, deadline):
    while time.monotonic() < deadline:
        clients = json.loads(subprocess.run(["hyprctl", "clients", "-j"], capture_output=True,
                                            text=True, timeout=10, check=True).stdout)
        for client in clients:
            if client["pid"] == pid and client["mapped"]:
                return client
        time.sleep(0.2)
    raise RuntimeError("owned window not mapped")


def scenario(binary, output, name, size, keys):
    with tempfile.TemporaryDirectory(prefix="lintra-desktop-capture-") as temporary:
        env = os.environ.copy()
        for key, folder in [("XDG_DATA_HOME", "data"), ("XDG_CONFIG_HOME", "config"),
                            ("XDG_CACHE_HOME", "cache")]:
            directory = Path(temporary) / folder
            directory.mkdir(mode=0o700)
            env[key] = str(directory)
        process = subprocess.Popen([str(binary)], env=env, cwd=temporary,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            client = owned_window(process.pid, time.monotonic() + 30)
            target = f'window="address:{client["address"]}"'
            (left, top), area = work_area()
            size = size or area
            if size[0] > area[0] or size[1] > area[1]:
                raise RuntimeError(f"{size} does not fit the work area {area}")
            hypr(f'hl.dsp.window.float({{action="enable", {target}}})')
            hypr(f'hl.dsp.window.resize({{x={size[0]}, y={size[1]}, {target}}})')
            hypr(f'hl.dsp.window.move({{x={left}, y={top}, {target}}})')
            # Session look (opacity/blur) must not alter the captured pixels.
            for prop in ("opaque", "no_blur"):
                hypr(f'hl.dsp.window.set_prop({{prop="{prop}", value="1", {target}}})')
            hypr(f'hl.dsp.focus({{{target}}})')
            time.sleep(3)  # bootstrap IPC, fonts and two animation frames
            for key in keys:
                # Re-focus: focus rings are painted only in the active window.
                hypr(f'hl.dsp.focus({{{target}}})')
                hypr(f'hl.dsp.send_shortcut({{mods="", key="{key}", {target}}})')
                time.sleep(0.4)
            hypr(f'hl.dsp.focus({{{target}}})')
            time.sleep(0.6)
            client = owned_window(process.pid, time.monotonic() + 5)
            if client["pid"] != process.pid:
                raise RuntimeError("window ownership changed")
            (x, y), (w, h) = client["at"], client["size"]
            image = output / f"{name}.png"
            subprocess.run(["grim", "-g", f"{x},{y} {w}x{h}", str(image)], check=True, timeout=15)
            return {"name": name, "requested": list(size), "captured": [w, h],
                    "keys": keys,
                    "image_sha256": hashlib.sha256(image.read_bytes()).hexdigest()}
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)


def main():
    binary = Path(sys.argv[1]).resolve(strict=True)
    output = Path(sys.argv[2]).resolve()
    output.mkdir(parents=True, exist_ok=True)
    results = [scenario(binary, output, *item) for item in SCENARIOS]
    # A key sequence that changes the view or theme must change the pixels
    # compared with the untouched Analisi capture of the same size.
    baseline = {tuple(item["captured"]): item["image_sha256"] for item in results if not item["keys"]}
    for item in results:
        if item["keys"]:
            assert item["image_sha256"] != baseline.get(tuple(item["captured"])), item["name"]
    receipt = {"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "scenarios": results}
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print("NATIVE_SHELL_CAPTURE_DONE", flush=True)


if __name__ == "__main__":
    main()
