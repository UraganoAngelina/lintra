"""Native shell captures and keyboard interaction on Hyprland (T-MIG-P5-01).

Personal CachyOS evidence, not a CI or release gate. Every scenario starts its
own process with isolated XDG directories, acts only on the window owned by
that PID (Hyprland dispatchers always receive ``window=address:...``; keys go
through ``send_shortcut`` to that window, no global input device), captures
that window's area with grim, then terminates and joins the process.

Usage: python3 tests/native_capture.py <binary> <output-dir> [<monitor>]
Writes <scenario>.png and receipt.json with binary/image SHA-256.
With <monitor> (e.g. a headless output created at scale 2 for the 200% check)
the scaled scenarios run on that monitor's active workspace instead.
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
    # T-MIG-P5-02: flow, explicit version and trace from the real Rust catalog.
    # "wait" lets Rust capture the schema before the trace field is enabled.
    ("keyboard-selection-1366x768", (1366, 768), ["Tab"] * 6 + ["Down", "wait", "Tab", "Down", "wait", "Tab", "Down", "wait"]),
    ("keyboard-selection-640x480", (640, 480), ["Tab"] * 6 + ["Down", "wait", "Tab", "Down", "wait", "Tab", "Down", "wait"]),
]
# 200% scale: logical sizes inside the scaled monitor's work area. The
# minimum window (640x480 logical) and the whole work area cover both sides
# of the breakpoint only if the work area is wider than 960; on a 1920x1080
# output at scale 2 it is not, so both use the top bar. Tab reaches every
# control; the captures show whether any command falls outside the window.
SCALED_SCENARIOS = [
    ("scale2-analisi-640x480", (640, 480), []),
    ("scale2-analisi-workarea", None, []),
    ("scale2-keyboard-skip-link-640x480", (640, 480), ["Tab"]),
    ("scale2-keyboard-risultati-workarea", None, ["Tab", "Tab", "Tab", "Return"]),
    ("scale2-keyboard-informazioni-640x480", (640, 480), ["Tab", "Tab", "Tab", "Tab", "Return"]),
    ("scale2-keyboard-theme-light-640x480", (640, 480), ["Tab"] * 5 + ["Down"]),
    ("scale2-keyboard-theme-dark-workarea", None, ["Tab"] * 5 + ["Down", "Down"]),
]


def hypr(expression):
    result = subprocess.run(["hyprctl", "dispatch", expression], capture_output=True, text=True, timeout=10)
    if result.returncode != 0 or result.stdout.strip() != "ok":
        raise RuntimeError(f"dispatch failed: {expression}: {result.stdout}{result.stderr}")


def monitor_info(name):
    """Named (or focused) monitor; work area in logical pixels minus reserved space."""
    monitors = json.loads(subprocess.run(["hyprctl", "monitors", "-j"], capture_output=True,
                                         text=True, timeout=10, check=True).stdout)
    monitor = next(item for item in monitors if (item["name"] == name if name else item["focused"]))
    scale = monitor["scale"]
    width, height = round(monitor["width"] / scale), round(monitor["height"] / scale)
    left, top, right, bottom = monitor["reserved"]
    return {"name": monitor["name"], "scale": scale, "workspace": monitor["activeWorkspace"]["name"],
            "origin": (monitor["x"] + left, monitor["y"] + top),
            "area": (width - left - right, height - top - bottom)}


def png_size(path):
    """Width/height from the PNG header: physical pixels of the capture."""
    header = path.read_bytes()[16:24]
    return int.from_bytes(header[:4], "big"), int.from_bytes(header[4:], "big")


def owned_window(pid, deadline):
    while time.monotonic() < deadline:
        clients = json.loads(subprocess.run(["hyprctl", "clients", "-j"], capture_output=True,
                                            text=True, timeout=10, check=True).stdout)
        for client in clients:
            if client["pid"] == pid and client["mapped"]:
                return client
        time.sleep(0.2)
    raise RuntimeError("owned window not mapped")


def scenario(binary, output, monitor_name, name, size, keys):
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
            monitor = monitor_info(monitor_name)
            (left, top), area = monitor["origin"], monitor["area"]
            size = size or area
            if size[0] > area[0] or size[1] > area[1]:
                raise RuntimeError(f"{size} does not fit the work area {area}")
            hypr(f'hl.dsp.window.float({{action="enable", {target}}})')
            if monitor_name:
                hypr(f'hl.dsp.window.move({{workspace="{monitor["workspace"]}", {target}}})')
            hypr(f'hl.dsp.window.resize({{x={size[0]}, y={size[1]}, {target}}})')
            hypr(f'hl.dsp.window.move({{x={left}, y={top}, {target}}})')
            # Session look (opacity/blur) must not alter the captured pixels.
            for prop in ("opaque", "no_blur"):
                hypr(f'hl.dsp.window.set_prop({{prop="{prop}", value="1", {target}}})')
            hypr(f'hl.dsp.focus({{{target}}})')
            time.sleep(3)  # bootstrap IPC, fonts and two animation frames
            for key in keys:
                if key == "wait":  # a Rust round trip (schema capture, plan) in progress
                    time.sleep(1.5)
                    continue
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
            if client["monitor"] != next(m["id"] for m in json.loads(subprocess.run(
                    ["hyprctl", "monitors", "-j"], capture_output=True, text=True, timeout=10,
                    check=True).stdout) if m["name"] == monitor["name"]):
                raise RuntimeError("window is not on the requested monitor")
            return {"name": name, "requested": list(size), "captured": [w, h],
                    "monitor": monitor["name"], "scale": monitor["scale"], "work_area": list(area),
                    "image_pixels": list(png_size(image)), "keys": keys,
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
    monitor_name = sys.argv[3] if len(sys.argv) > 3 else None
    scenarios = SCALED_SCENARIOS if monitor_name else SCENARIOS
    results = [scenario(binary, output, monitor_name, *item) for item in scenarios]
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
