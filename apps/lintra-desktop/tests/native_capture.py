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

# Synthetic inputs for the native file dialog (only [a-z0-9/.] so they can be
# typed key by key); created before the scenarios and removed afterwards.
SAMPLE_DIR = str(Path.home() / "lintrap503")
SELECT = ["Tab"] * 6 + ["Down", "wait", "Tab", "Down", "wait", "Tab", "Down", "wait"]
# P5-05: the 140000-byte file assigned, then four Tab (XLSX, JSONL, CSV,
# Avvia analisi) and Return. A configured input threshold of 64 KiB is
# crossed by that file; the invalid file has an empty measurement source.
FILE = SELECT + ["Tab", "Return", f"dialog:{SAMPLE_DIR}/partial.txt", "wait"]
START = ["Tab"] * 4 + ["Return", "wait", "wait"]
BUDGET = b'{"version":1,"measurement_source":"misura sintetica","input_bytes":65536,"reference_bytes":null}'
# P5-06: a 245 MB input (about 10 s of validation) leaves time to cancel.
FILE_BIG = SELECT + ["Tab", "Return", f"dialog:{SAMPLE_DIR}/big.txt", "wait"]
START_NOWAIT = ["Tab"] * 4 + ["Return", "wait"]
INVALID_BUDGET = b'{"version":1,"measurement_source":"","input_bytes":65536,"reference_bytes":null}'

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
    # T-MIG-P5-03: the real native dialog opened by Rust. "dialog:<path>" types
    # the path in the GTK chooser of the owned process; synthetic files only.
    ("keyboard-input-partial-1366x768", (1366, 768), SELECT + ["Tab", "Return", f"dialog:{SAMPLE_DIR}/partial.txt", "wait"]),
    ("keyboard-input-whole-640x480", (640, 480), SELECT + ["Tab", "Return", f"dialog:{SAMPLE_DIR}/whole.txt", "wait"]),
    ("keyboard-input-cancel-1366x768", (1366, 768), SELECT + ["Tab", "Return", "dialog-cancel", "wait"]),
    # P503-02: after Esc the button keeps focus, so Space alone reopens the
    # dialog (drive_dialog fails if it does not appear) without new Tab.
    ("keyboard-input-cancel-reopen-1366x768", (1366, 768), SELECT + ["Tab", "Return", "dialog-cancel", "wait", "space", "dialog-cancel", "wait"]),
    # T-MIG-P5-04: report formats after the file button; Space unchecks XLSX
    # (default on) and checks CSV (default off), focus left on CSV.
    ("keyboard-reports-1366x768", (1366, 768), SELECT + ["Tab", "Tab", "space", "Tab", "Tab", "space"]),
    ("keyboard-reports-640x480", (640, 480), SELECT + ["Tab", "Tab", "space", "Tab", "Tab", "space"]),
    # T-MIG-P5-05: run screen and the D09 soft budget. The optional fourth
    # item is resource-budget.json, written in the scenario's private
    # XDG_CONFIG_HOME/it.lintra.desktop before launch (absent: unconfigured).
    ("keyboard-run-unconfigured-1366x768", (1366, 768), FILE + START + ["wait", "wait"]),
    ("keyboard-run-unconfigured-640x480", (640, 480), FILE + START + ["wait", "wait"]),
    ("keyboard-budget-invalid-1366x768", (1366, 768), FILE, INVALID_BUDGET),
    ("keyboard-budget-warning-1366x768", (1366, 768), FILE + START, BUDGET),
    ("keyboard-budget-warning-640x480", (640, 480), FILE + START, BUDGET),
    ("keyboard-budget-proceed-1366x768", (1366, 768), FILE + START + ["Tab", "Return", "wait", "wait", "wait"], BUDGET),
    ("keyboard-budget-annulla-1366x768", (1366, 768), FILE + START + ["Return", "wait", "wait"], BUDGET),
    ("keyboard-budget-escape-1366x768", (1366, 768), FILE + START + ["Escape", "wait", "wait"], BUDGET),
    # P505-02: the file is corrected (or removed) while the app runs; the
    # next Avvia reads it again. Invalid → valid shows the warning, invalid →
    # removed runs without a budget. First Avvia is refused while invalid.
    ("keyboard-budget-invalid-fixed-1366x768", (1366, 768),
     FILE + START + ["budget-valid"] + ["Return", "wait", "wait"], INVALID_BUDGET),
    ("keyboard-budget-invalid-removed-1366x768", (1366, 768),
     FILE + START + ["budget-remove"] + ["Return", "wait", "wait"], INVALID_BUDGET),
    # T-MIG-P5-06: operator cancel. "Interrompi analisi" follows "Avvia".
    ("keyboard-run-interrupt-offered-1366x768", (1366, 768), FILE_BIG + START_NOWAIT + ["Tab"]),
    ("keyboard-run-cancelled-1366x768", (1366, 768), FILE_BIG + START_NOWAIT + ["Tab", "Return", "wait", "wait", "wait"]),
    # A completed run publishes a report; a newer run is cancelled: the old
    # report is named as the previous analysis', never as this one's result.
    ("keyboard-run-cancelled-previous-report-1366x768", (1366, 768),
     FILE + START + ["wait", "wait"] + ["shift-Tab"] * 4 + ["Return", f"dialog:{SAMPLE_DIR}/big.txt", "wait"]
     + START_NOWAIT + ["Tab", "Return", "wait", "wait", "wait"]),
    # T-MIG-P5-07: Informazioni. After Return focus is on the h1, so Tab
    # reaches the first license disclosure; Return opens it, Tab focuses its
    # scrollable text. After a completed run, nine shift-Tab from «Avvia
    # analisi» reach the Informazioni button: report provenance beside the
    # selection's.
    ("keyboard-informazioni-license-1366x768", (1366, 768), ["Tab"] * 4 + ["Return", "Tab", "Return", "Tab"]),
    ("keyboard-informazioni-license-640x480", (640, 480), ["Tab"] * 4 + ["Return", "Tab", "Return", "Tab"]),
    ("keyboard-informazioni-report-1366x768", (1366, 768), FILE + START + ["wait", "wait"] + ["shift-Tab"] * 9 + ["Return", "wait"]),
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


def owned_dialog(pid, main_address, deadline):
    """A second window of the same process: the native file dialog."""
    while time.monotonic() < deadline:
        clients = json.loads(subprocess.run(["hyprctl", "clients", "-j"], capture_output=True,
                                            text=True, timeout=10, check=True).stdout)
        for client in clients:
            if client["pid"] == pid and client["mapped"] and client["address"] != main_address:
                return client
        time.sleep(0.2)
    raise RuntimeError("native dialog not mapped")


def drive_dialog(pid, main_address, step):
    dialog = owned_dialog(pid, main_address, time.monotonic() + 15)
    target = f'window="address:{dialog["address"]}"'
    hypr(f'hl.dsp.focus({{{target}}})')
    time.sleep(1)
    if step == "dialog-cancel":
        keys = ["Escape"]
    else:
        path = step.removeprefix("dialog:")
        if not all(c.islower() or c.isdigit() or c in "/." for c in path):
            raise RuntimeError("dialog path must be typeable")
        names = {"/": "KP_Divide", ".": "period"}  # keypad slash: same on every layout
        keys = [names.get(c, c) for c in path]
    if step != "dialog-cancel":
        # GTK4 opens in "Recent" with type-to-search; Ctrl+L opens the location field.
        hypr(f'hl.dsp.send_shortcut({{mods="CTRL", key="l", {target}}})')
        time.sleep(0.5)
    for key in keys:
        hypr(f'hl.dsp.send_shortcut({{mods="", key="{key}", {target}}})')
        time.sleep(0.12)
    if step == "dialog-cancel":
        return
    # GTK4 may consume a first Enter to accept the autocompletion: confirm
    # until the dialog window is gone (bounded). The dialog may close between
    # the check and the key, so "window not found" on Enter means accepted.
    for _ in range(3):
        time.sleep(0.5)
        try:
            owned_dialog(pid, main_address, time.monotonic() + 0.5)
            hypr(f'hl.dsp.send_shortcut({{mods="", key="Return", {target}}})')
        except RuntimeError as error:
            if "not mapped" in str(error) or "window not found" in str(error):
                return
            raise
    raise RuntimeError("native dialog did not accept the file")


def owned_window(pid, deadline):
    while time.monotonic() < deadline:
        clients = json.loads(subprocess.run(["hyprctl", "clients", "-j"], capture_output=True,
                                            text=True, timeout=10, check=True).stdout)
        for client in clients:
            if client["pid"] == pid and client["mapped"]:
                return client
        time.sleep(0.2)
    raise RuntimeError("owned window not mapped")


def scenario(binary, output, monitor_name, name, size, keys, budget=None):
    with tempfile.TemporaryDirectory(prefix="lintra-desktop-capture-") as temporary:
        env = os.environ.copy()
        for key, folder in [("XDG_DATA_HOME", "data"), ("XDG_CONFIG_HOME", "config"),
                            ("XDG_CACHE_HOME", "cache")]:
            directory = Path(temporary) / folder
            directory.mkdir(mode=0o700)
            env[key] = str(directory)
        # Tauri app_config_dir on Linux: $XDG_CONFIG_HOME/<identifier>.
        config = Path(env["XDG_CONFIG_HOME"]) / "it.lintra.desktop"
        if budget is not None:
            config.mkdir(mode=0o700)
            (config / "resource-budget.json").write_bytes(budget)
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
                if key.startswith("dialog"):
                    drive_dialog(process.pid, client["address"], key)
                    continue
                if key == "budget-valid":  # the operator corrects the file
                    (config / "resource-budget.json").write_bytes(BUDGET)
                    continue
                if key == "budget-remove":  # the operator removes the file
                    (config / "resource-budget.json").unlink()
                    continue
                # Re-focus: focus rings are painted only in the active window.
                hypr(f'hl.dsp.focus({{{target}}})')
                # `key_name`, not `name`: `name` is the scenario (image file).
                mods, key_name = ("SHIFT", key.removeprefix("shift-")) if key.startswith("shift-") else ("", key)
                hypr(f'hl.dsp.send_shortcut({{mods="{mods}", key="{key_name}", {target}}})')
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
                    "budget": None if budget is None else budget.decode(),
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
    samples = Path(SAMPLE_DIR)
    samples.mkdir(exist_ok=False)
    try:
        (samples / "partial.txt").write_bytes(b"001A  \n" * 20000)  # 140000 bytes > 64 KiB sample
        (samples / "whole.txt").write_bytes(b"001A  \n001B  \r\n")  # 15 bytes, LF and CRLF
        with (samples / "big.txt").open("wb") as big:  # 35 M records, 245 MB
            for _ in range(350):
                big.write(b"001A  \n" * 100_000)
        results = [scenario(binary, output, monitor_name, *item) for item in scenarios]
    finally:
        for file in samples.iterdir():
            file.unlink()
        samples.rmdir()
    # A key sequence that changes the view or theme must change the pixels
    # compared with the untouched Analisi capture of the same size.
    baseline = {tuple(item["captured"]): item["image_sha256"] for item in results if not item["keys"]}
    for item in results:
        if item["keys"]:
            assert item["image_sha256"] != baseline.get(tuple(item["captured"])), item["name"]
    # Every scenario has its own image (a lost file would hide a scenario).
    names = [item["name"] for item in results]
    assert len(set(names)) == len(names) and all((output / f"{n}.png").is_file() for n in names), "one image per scenario"
    receipt = {"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "scenarios": results}
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print("NATIVE_SHELL_CAPTURE_DONE", flush=True)


if __name__ == "__main__":
    main()
