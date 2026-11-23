#!/usr/bin/env python3
"""T-MIG-P5-05 acceptance on the real app: a synthetic run of at least 60 s
keeps the UI responsive with bounded memory, and the messages follow Rust.

Usage: python3 tests/native_run.py <binary> <output-dir> [records]

CachyOS/Hyprland only (hyprctl, grim), like native_capture.py, whose helpers
it reuses. The input is synthetic ("001A  " records of the embedded SYNTHETIC
schema: one valid key, every later record is a duplicate-key violation), the
run spool goes to a private disk directory (TMPDIR) and everything is removed
afterwards. Evidence in <output-dir>: screenshots every 5 s, a keyboard probe
during the run, the terminal screen and run.json (durations, RSS samples of
the app process and of the WebKit renderer, image hashes).
"""
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from native_capture import SELECT, drive_dialog, hypr, owned_window  # noqa: E402

WORK = Path.home() / "lintrarun"  # typeable path for the GTK dialog
MIN_SECONDS = 60
SAMPLE_SECONDS = 5
# Renderer memory must stay flat: no per-record data reaches JavaScript.
RENDERER_GROWTH_LIMIT_KIB = 64 * 1024


def rss_kib(pid):
    try:
        for line in Path(f"/proc/{pid}/status").read_text().splitlines():
            if line.startswith("VmRSS:"):
                return int(line.split()[1])
    except OSError:
        return None
    return None


def renderer_pids(root):
    """WebKitGTK web processes descending from the app process."""
    children, names = {}, {}
    for entry in Path("/proc").iterdir():
        if not entry.name.isdigit():
            continue
        try:
            stat = (entry / "stat").read_text()
        except OSError:
            continue
        name = stat[stat.index("(") + 1:stat.rindex(")")]
        parent = int(stat[stat.rindex(")") + 2:].split()[1])
        children.setdefault(parent, []).append(int(entry.name))
        names[int(entry.name)] = name
    found, stack = [], [root]
    while stack:
        for child in children.get(stack.pop(), []):
            stack.append(child)
            if names[child].startswith("WebKitWebProces"):
                found.append(child)
    return found


def capture(client, path):
    (x, y), (w, h) = client["at"], client["size"]
    subprocess.run(["grim", "-g", f"{x},{y} {w}x{h}", str(path)], check=True, timeout=15)
    return hashlib.sha256(path.read_bytes()).hexdigest()


def published(spool_root):
    """The run's private report is renamed from .part when Rust publishes it."""
    return any(path.name == "errors.jsonl" for path in spool_root.glob("lintra-desktop-run-*/errors.jsonl"))


def main():
    binary = Path(sys.argv[1]).resolve(strict=True)
    output = Path(sys.argv[2]).resolve()
    records = int(sys.argv[3]) if len(sys.argv) > 3 else 4_000_000
    output.mkdir(parents=True, exist_ok=True)
    WORK.mkdir(exist_ok=False)
    spool = WORK / "tmp"
    spool.mkdir(mode=0o700)
    try:
        with (WORK / "run.txt").open("wb") as file:
            chunk = b"001A  \n" * 100_000
            for _ in range(records // 100_000):
                file.write(chunk)
        with tempfile.TemporaryDirectory(prefix="lintra-desktop-run-test-") as temporary:
            env = os.environ.copy()
            for key, folder in [("XDG_DATA_HOME", "data"), ("XDG_CONFIG_HOME", "config"), ("XDG_CACHE_HOME", "cache")]:
                (Path(temporary) / folder).mkdir(mode=0o700)
                env[key] = str(Path(temporary) / folder)
            env["TMPDIR"] = str(spool)
            process = subprocess.Popen([str(binary)], env=env, cwd=temporary,
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                result = drive(process, output, records, spool)
            finally:
                process.terminate()
                try:
                    process.wait(timeout=20)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=10)
    finally:
        shutil.rmtree(WORK)
    result["binary_sha256"] = hashlib.sha256(binary.read_bytes()).hexdigest()
    (output / "run.json").write_text(json.dumps(result, indent=2) + "\n")
    print("NATIVE_RUN_DONE", flush=True)


def drive(process, output, records, spool):
    client = owned_window(process.pid, time.monotonic() + 30)
    target = f'window="address:{client["address"]}"'
    hypr(f'hl.dsp.window.float({{action="enable", {target}}})')
    hypr(f'hl.dsp.window.resize({{x=1366, y=768, {target}}})')
    for prop in ("opaque", "no_blur"):
        hypr(f'hl.dsp.window.set_prop({{prop="{prop}", value="1", {target}}})')
    hypr(f'hl.dsp.focus({{{target}}})')
    time.sleep(3)

    def key(name):
        hypr(f'hl.dsp.focus({{{target}}})')
        hypr(f'hl.dsp.send_shortcut({{mods="", key="{name}", {target}}})')
        time.sleep(0.4)

    for name in SELECT + ["Tab", "Return"]:
        if name == "wait":
            time.sleep(1.5)
        else:
            key(name)
    drive_dialog(process.pid, client["address"], f"dialog:{WORK}/run.txt")
    time.sleep(1.5)
    for name in ["Tab"] * 4:
        key(name)
    hypr(f'hl.dsp.focus({{{target}}})')
    hypr(f'hl.dsp.send_shortcut({{mods="", key="Return", {target}}})')
    started = time.monotonic()
    samples, images = [], []
    probe = None
    while not published(spool):
        elapsed = time.monotonic() - started
        if elapsed > 30 * 60:
            raise RuntimeError("run did not finish in 30 minutes")
        client = owned_window(process.pid, time.monotonic() + 5)
        renderers = renderer_pids(process.pid)
        samples.append({"t": round(elapsed, 1), "app_rss_kib": rss_kib(process.pid),
                        "renderer_rss_kib": sum(rss_kib(pid) or 0 for pid in renderers),
                        "renderers": len(renderers)})
        name = f"run-{int(elapsed):04d}s.png"
        images.append({"t": round(elapsed, 1), "name": name, "sha256": capture(client, output / name)})
        if probe is None and elapsed >= 20:
            # Responsiveness during the run: Shift+Tab moves the focus ring
            # away from "Avvia analisi"; the next frame must show it within 1 s.
            before = capture(client, output / "probe-before.png")
            hypr(f'hl.dsp.focus({{{target}}})')
            hypr(f'hl.dsp.send_shortcut({{mods="SHIFT", key="Tab", {target}}})')
            reacted = None
            for attempt in range(10):
                time.sleep(0.1)
                if capture(client, output / "probe-after.png") != before:
                    reacted = round(0.1 * (attempt + 1), 1)
                    break
            probe = {"t": round(elapsed, 1), "key": "Shift+Tab", "reacted_within_s": reacted}
            if reacted is None:
                raise RuntimeError("UI did not react to the keyboard during the run")
        time.sleep(SAMPLE_SECONDS)
    duration = time.monotonic() - started
    time.sleep(3)  # the next poll shows the terminal and acknowledges it
    client = owned_window(process.pid, time.monotonic() + 5)
    final = capture(client, output / "run-terminal.png")
    if duration < MIN_SECONDS:
        raise RuntimeError(f"run lasted {duration:.1f}s < {MIN_SECONDS}s: raise the record count")
    progress_frames = len({image["sha256"] for image in images})
    if progress_frames < 3:
        raise RuntimeError("the run screen did not repaint during the run")
    renderer = [s["renderer_rss_kib"] for s in samples if s["renderers"]]
    growth = max(renderer) - min(renderer) if renderer else None
    if growth is None or growth > RENDERER_GROWTH_LIMIT_KIB:
        raise RuntimeError(f"renderer memory not bounded: growth {growth} KiB")
    return {"records": records, "input_bytes": records * 7, "run_seconds": round(duration, 1),
            "samples": samples, "images": images, "distinct_run_frames": progress_frames,
            "keyboard_probe": probe, "renderer_rss_growth_kib": growth,
            "app_rss_kib_min_max": [min(s["app_rss_kib"] for s in samples), max(s["app_rss_kib"] for s in samples)],
            "terminal_image_sha256": final}


if __name__ == "__main__":
    main()
