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
# T-MIG-P5-09: the "Tipo di analisi" radio group (one Tab stop, LocalOnly
# checked) sits between the version and the trace.
SELECT = ["Tab"] * 6 + ["Down", "wait", "Tab", "Down", "wait", "Tab", "Tab", "Down", "wait"]
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
# T-MIG-P5-09: Full on an external synthetic bundle with traces S and T and
# two parameters (FULL_SCHEMA below). Down on the radio group selects Full;
# Tab then reaches the file button of S, then the one of T, then the
# parameters unit and year. "type:<text>" types into the focused field.
FULL = ["Tab"] * 6 + ["Down", "wait", "Tab", "Down", "wait", "Tab", "Down", "wait"]
FULL_S = FULL + ["Tab", "Return", f"dialog:{SAMPLE_DIR}/fulls.txt", "wait"]
FULL_ST = FULL_S + ["Tab", "Return", f"dialog:{SAMPLE_DIR}/fullt.txt", "wait"]
# T-MIG-P5-10: parameters applied, then the four Tab to «Avvia analisi».
FULL_PARAMS_START = ["Tab", "type:a", "Tab", "wait", "type:2026", "Tab", "wait"] + ["Tab"] * 3 + ["Return"]
# S is big.txt (245 MB, records 001A: PK 00, KEY 1A; no second large file),
# T the one record every S row joins.
FULL_BIG = FULL + ["Tab", "Return", f"dialog:{SAMPLE_DIR}/big.txt", "wait",
                   "Tab", "Return", f"dialog:{SAMPLE_DIR}/onet.txt", "wait"]
# T-MIG-P6-01: invalid.txt has 100 000 records of 4 bytes instead of 6, one
# length diagnostic each (782 pages of 128). After the run, shift-Tab x11
# from «Avvia analisi» reaches Risultati (Informazioni is x10); the section
# title then has the focus: Tab reaches «Nuova analisi», the five filters
# (T-MIG-P6-02: Codice, ID campo, ID record, Codice fiscale, Riga), «Azzera»,
# «Applica filtri», the grid viewport, «Pagina precedente», «Pagina 1», «Pagina 2».
FILE_INVALID = SELECT + ["Tab", "Return", f"dialog:{SAMPLE_DIR}/invalid.txt", "wait"]
TO_RESULTS = ["wait", "wait"] + ["shift-Tab"] * 11 + ["Return", "wait", "wait", "wait"]
GRID = FILE_INVALID + START + TO_RESULTS
# T-MIG-P6-02: Tab x2 from the title reaches «Codice», x6 «Riga». Enter in a
# field applies the filters; a filtered query is built by the Rust worker.
TO_CODE = ["Tab"] * 2
TO_LINE = ["Tab"] * 6

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
    ("keyboard-selection-1366x768", (1366, 768), SELECT),
    ("keyboard-selection-640x480", (640, 480), SELECT),
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
    # scrollable text. After a completed run, ten shift-Tab from «Avvia
    # analisi» (nine before the P5-09 mode group) reach the Informazioni
    # button: report provenance beside the selection's.
    ("keyboard-informazioni-license-1366x768", (1366, 768), ["Tab"] * 4 + ["Return", "Tab", "Return", "Tab"]),
    ("keyboard-informazioni-license-640x480", (640, 480), ["Tab"] * 4 + ["Return", "Tab", "Return", "Tab"]),
    # T-MIG-P6-01: Risultati with the summary and the virtualised grid.
    ("results-grid-1366x768", (1366, 768), GRID),
    ("results-grid-640x480", (640, 480), GRID),
    ("results-grid-workarea", None, GRID),
    ("results-grid-page-2-1366x768", (1366, 768), GRID + ["Tab"] * 12 + ["Return", "wait"]),
    ("results-grid-scrolled-1366x768", (1366, 768), GRID + ["Tab"] * 9 + ["End", "wait"]),
    # From the section title one shift-Tab reaches the theme group: Scuro.
    ("results-grid-dark-1366x768", (1366, 768), GRID + ["shift-Tab", "Down", "Down", "wait"]),
    ("results-valid-1366x768", (1366, 768), FILE + START + TO_RESULTS),
    # From the section title one shift-Tab reaches the theme group: Chiaro.
    ("results-grid-light-1366x768", (1366, 768), GRID + ["shift-Tab", "Down", "wait"]),
    # T-MIG-P6-02: the five exact filters. Line 42: one match. The code in
    # lower case: no match (exact, case-sensitive). The exact code: every
    # diagnostic, counted as matches. Line 0: refused next to the field, no
    # query. Then «Azzera» (Tab from «Riga») goes back to the whole report.
    # Tab x3 from «Riga» (x7 from «Codice») focuses the grid viewport, scrolling the rows into view.
    ("results-filter-line-1366x768", (1366, 768), GRID + TO_LINE + ["type:42", "Return", "wait", "wait"] + ["Tab"] * 3 + ["wait"]),
    ("results-filter-line-640x480", (640, 480), GRID + TO_LINE + ["type:42", "Return", "wait", "wait"]),
    ("results-filter-code-nomatch-1366x768", (1366, 768), GRID + TO_CODE + ["type:e_record_length", "Return", "wait", "wait"]),
    ("results-filter-code-1366x768", (1366, 768), GRID + TO_CODE + ["type:E_RECORD_LENGTH", "Return", "wait", "wait"] + ["Tab"] * 7 + ["wait"]),
    ("results-filter-line-invalid-1366x768", (1366, 768), GRID + TO_LINE + ["type:0", "Return", "wait"]),
    # T-MIG-P6-03: Tab x9 from the title focuses the grid viewport; Down
    # selects the first row and Rust pins its detail (panel on the right at
    # 1366, under the table at 640). With line 42 there is one row and no
    # pager: Tab from the row reaches «Mostra il contesto» in the panel.
    ("results-detail-1366x768", (1366, 768), GRID + ["Tab"] * 9 + ["Down", "wait", "wait"]),
    # At 640 the panel is under the table: focusing «Mostra il contesto» scrolls it into view.
    ("results-detail-640x480", (640, 480),
     GRID + TO_LINE + ["type:42", "Return", "wait", "wait"] + ["Tab"] * 3 + ["Down", "wait", "wait", "Tab", "wait"]),
    ("results-detail-keys-1366x768", (1366, 768), GRID + ["Tab"] * 9 + ["Down", "Down", "Down", "wait", "wait"]),
    ("results-detail-context-1366x768", (1366, 768),
     GRID + TO_LINE + ["type:42", "Return", "wait", "wait"] + ["Tab"] * 3 + ["Down", "wait", "wait", "Tab", "Return", "wait", "wait"]),
    ("results-filter-cleared-1366x768", (1366, 768),
     GRID + TO_LINE + ["type:42", "Return", "wait", "wait", "Tab", "Return", "wait", "wait"]),
    ("keyboard-informazioni-report-1366x768", (1366, 768), FILE + START + ["wait", "wait"] + ["shift-Tab"] * 10 + ["Return", "wait"]),
    # T-MIG-P5-09: Full selection on the external bundle (last item True).
    ("full-selection-1366x768", (1366, 768), FULL, None, True),
    ("full-selection-640x480", (640, 480), FULL, None, True),
    # Both files assigned; unit applied on leaving the field; year "20x6"
    # is refused in React (never sent) and announced at the field; the last
    # Tab (no new request: the field is already failed) scrolls it into view.
    ("full-parameter-blocked-1366x768", (1366, 768),
     FULL_ST + ["Tab", "type:a", "Tab", "wait", "type:20x6", "Return", "wait", "Tab"], None, True),
    # Only S assigned, no parameter: Rust refuses the plan at Avvia.
    ("full-prepare-refused-1366x768", (1366, 768), FULL_S + ["Tab", "Tab", "Tab"] + START, None, True),
    # S and T, unit a and year 2026: the run completes in Full.
    ("full-run-completed-1366x768", (1366, 768),
     FULL_ST + ["Tab", "type:a", "Tab", "wait", "type:2026", "Tab", "wait"] + ["Tab"] * 3 + ["Return", "wait", "wait", "wait"], None, True),
    # T-MIG-P5-10: Full monitor. PASS1 of the large S, named from the plan
    # handle in the sample; a later capture of the same run; a cancel in
    # PASS1 with the last phase Rust reported; an invalid bundle (T's DATA
    # differs from S's) whose terminal is Rust's "Violazioni rilevate".
    ("full-monitor-pass1-1366x768", (1366, 768), FULL_BIG + FULL_PARAMS_START + ["wait"], None, True),
    ("full-monitor-later-1366x768", (1366, 768), FULL_BIG + FULL_PARAMS_START + ["wait"] * 6, None, True),
    ("full-run-cancelled-1366x768", (1366, 768), FULL_BIG + FULL_PARAMS_START + ["wait", "Tab", "Return", "wait", "wait", "wait"], None, True),
    ("full-run-invalid-1366x768", (1366, 768),
     FULL + ["Tab", "Return", f"dialog:{SAMPLE_DIR}/fulls.txt", "wait", "Tab", "Return", f"dialog:{SAMPLE_DIR}/badt.txt", "wait"]
     + FULL_PARAMS_START + ["wait", "wait", "wait"], None, True),
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


def full_bundle(root):
    """External synthetic bundle for the Full scenarios: the golden schema
    with traces S and T joined on KEY (as tests/support/full_schema.rs) and
    the parameters unit (text, a/b, a join namespace) and year (integer)."""
    raw = json.loads((Path(__file__).resolve().parents[3] / "tests/golden/synthetic/v1/schema.json").read_text())
    trace = raw["traces"][0]
    trace["record_length"] = 6
    field = trace["fields"][0]
    trace["fields"] = [{**json.loads(json.dumps(field)), "id": fid,
                        "physical": {**field["physical"], "start": i * 2 + 1, "length": 2}}
                       for i, fid in enumerate(["PK", "KEY", "DATA"])]
    trace["record_key"]["fields"] = ["PK"]
    raw["traces"] = [{**json.loads(json.dumps(trace)), "id": tid} for tid in ("S", "T")]
    raw["required_capabilities"] = ["local.v1", "expressions.v1", "joins.v1"]
    policy = {"policy": "violation", "error_code": "E_RELATED_RECORD_MISSING",
              "diagnostic": {"code": "SYNTHETIC_MISSING", "message_template": "Missing related record for {rule_id}"}}
    raw["traces"][0]["joins"] = [{"id": "S_T", "target_trace": "T", "source_fields": ["KEY"], "target_fields": ["KEY"],
                                  "cardinality": "zero_or_one", "scope": "current_submission",
                                  "namespace_parameters": ["unit"], "on_missing_record": policy, "on_missing_trace": policy}]
    raw["traces"][0]["rules"] = [{"id": "COMPARE", "when": {"const": True}, "assertions": [{
        "kind": "field_compare", "left": {"ref": {"field": "DATA"}}, "operator": "==",
        "right": {"ref": {"trace": "T", "field": "DATA", "join": "S_T"}}, "value_type": "bytes"}],
        "error_code": "E_CROSS_FIELD", "message_template": "Synthetic {rule_id}: {condition}"}]
    raw["parameters"] = [{"id": "unit", "type": "string", "allowed_values": ["a", "b"]},
                         {"id": "year", "type": "integer", "min": 1, "max": 9999}]
    folder = root / "schemas" / raw["flow_id"] / raw["schema_version"]
    folder.mkdir(parents=True)
    (folder / "flow.json").write_text(json.dumps(raw))
    (root / "reference-data").mkdir()
    return root / "schemas", root / "reference-data"


def scenario(binary, output, monitor_name, name, size, keys, budget=None, full=False):
    with tempfile.TemporaryDirectory(prefix="lintra-desktop-capture-") as temporary:
        env = os.environ.copy()
        for key, folder in [("XDG_DATA_HOME", "data"), ("XDG_CONFIG_HOME", "config"),
                            ("XDG_CACHE_HOME", "cache")]:
            directory = Path(temporary) / folder
            directory.mkdir(mode=0o700)
            env[key] = str(directory)
        # Tauri app_config_dir on Linux: $XDG_CONFIG_HOME/<identifier>.
        if full:
            schemas, references = full_bundle(Path(temporary) / "bundle")
            env["LINTRA_SCHEMA_DIR"], env["LINTRA_REFERENCE_DATA_DIR"] = str(schemas), str(references)
        else:  # the embedded bundle, whatever the calling shell exports
            env.pop("LINTRA_SCHEMA_DIR", None)
            env.pop("LINTRA_REFERENCE_DATA_DIR", None)
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
                if key.startswith("type:"):  # characters typed into the focused field
                    hypr(f'hl.dsp.focus({{{target}}})')
                    for char in key.removeprefix("type:"):
                        # P6-02: upper case and "_" for exact filter codes.
                        if char.isascii() and (char.islower() or char.isdigit()):
                            mods, key_name = "", char
                        elif char.isascii() and char.isupper():
                            mods, key_name = "SHIFT", char.lower()
                        elif char == "_":
                            # Hyprland resolves keysyms on the last layout of "us,it"
                            # (minus -> the "/" key) and the window reads us: the
                            # physical keycode of us "-" avoids the lookup.
                            mods, key_name = "SHIFT", "code:20"
                        else:
                            raise RuntimeError("typed text must be [A-Za-z0-9_]")
                        hypr(f'hl.dsp.send_shortcut({{mods="{mods}", key="{key_name}", {target}}})')
                        time.sleep(0.12)
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
                    "budget": None if budget is None else budget.decode(), "full_bundle": full,
                    "image_sha256": hashlib.sha256(image.read_bytes()).hexdigest()}
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)


def active_run_spools(tmp):
    """Run spools whose report is still being written: the atomic writer keeps
    `errors.jsonl.part` until publication renames it to `errors.jsonl`, so a
    completed report (spool kept with `errors.jsonl`) is never counted."""
    return sorted(p.name for p in Path(tmp).glob("lintra-desktop-run-*")
                  if (p / "errors.jsonl.part").is_file() and not (p / "errors.jsonl").exists())


def close_during_run(binary, output):
    """T-MIG-P5-08, Slint parity "Annulla/chiudi": the owned window is closed
    by the compositor while a 245 MB run is active. The process must exit on
    its own with code 0 (cancel + join, P4-12) and leave no run spool in its
    private TMPDIR. A capture just before closing shows the run active."""
    with tempfile.TemporaryDirectory(prefix="lintra-desktop-close-") as temporary:
        env = os.environ.copy()
        for key, folder in [("XDG_DATA_HOME", "data"), ("XDG_CONFIG_HOME", "config"),
                            ("XDG_CACHE_HOME", "cache"), ("TMPDIR", "tmp")]:
            directory = Path(temporary) / folder
            directory.mkdir(mode=0o700)
            env[key] = str(directory)
        env.pop("LINTRA_SCHEMA_DIR", None)
        env.pop("LINTRA_REFERENCE_DATA_DIR", None)
        process = subprocess.Popen([str(binary)], env=env, cwd=temporary,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            client = owned_window(process.pid, time.monotonic() + 30)
            target = f'window="address:{client["address"]}"'
            monitor = monitor_info(None)
            (left, top) = monitor["origin"]
            hypr(f'hl.dsp.window.float({{action="enable", {target}}})')
            hypr(f'hl.dsp.window.resize({{x=1366, y=768, {target}}})')
            hypr(f'hl.dsp.window.move({{x={left}, y={top}, {target}}})')
            for prop in ("opaque", "no_blur"):
                hypr(f'hl.dsp.window.set_prop({{prop="{prop}", value="1", {target}}})')
            hypr(f'hl.dsp.focus({{{target}}})')
            time.sleep(3)
            for key in FILE_BIG + START_NOWAIT:
                if key == "wait":
                    time.sleep(1.5)
                elif key.startswith("dialog"):
                    drive_dialog(process.pid, client["address"], key)
                else:
                    hypr(f'hl.dsp.focus({{{target}}})')
                    hypr(f'hl.dsp.send_shortcut({{mods="", key="{key}", {target}}})')
                    time.sleep(0.4)
            client = owned_window(process.pid, time.monotonic() + 5)
            (x, y), (w, h) = client["at"], client["size"]
            image = output / "close-during-run-1366x768.png"
            subprocess.run(["grim", "-g", f"{x},{y} {w}x{h}", str(image)], check=True, timeout=15)
            # Checked last before closing: a run completed earlier must fail here.
            spools_running = active_run_spools(Path(temporary) / "tmp")
            assert spools_running, "no spool with a pending report: the run was not active when closed"
            closed_at = time.monotonic()
            hypr(f'hl.dsp.window.close({{{target}}})')
            code = process.wait(timeout=60)
            elapsed = round(time.monotonic() - closed_at, 2)
            left_over = sorted(p.name for p in (Path(temporary) / "tmp").glob("lintra-desktop-run-*"))
            result = {"name": "close-during-run-1366x768", "exit_code": code, "seconds_to_exit": elapsed,
                      "spools_while_running": spools_running, "spools_after_exit": left_over,
                      "image_sha256": hashlib.sha256(image.read_bytes()).hexdigest()}
            assert code == 0 and not left_over, result
            return result
        finally:
            if process.poll() is None:
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
        (samples / "fulls.txt").write_bytes(b"01AAXX\n02BBYY\n")  # Full trace S (PK, KEY, DATA)
        (samples / "fullt.txt").write_bytes(b"01AAXX\n02BBYY\n")  # Full trace T: same KEY/DATA
        (samples / "badt.txt").write_bytes(b"01AAZZ\n02BBYY\n")  # T: DATA of AA differs from S
        (samples / "onet.txt").write_bytes(b"001A  \n")  # T: the record every big.txt row joins
        (samples / "invalid.txt").write_bytes(b"001A\n" * 100_000)  # P6-01: 100 000 length diagnostics
        with (samples / "big.txt").open("wb") as big:  # 35 M records, 245 MB
            for _ in range(350):
                big.write(b"001A  \n" * 100_000)
        results = [scenario(binary, output, monitor_name, *item) for item in scenarios]
        close = None if monitor_name else close_during_run(binary, output)
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
    if close is not None:
        receipt["close_during_run"] = close
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print("NATIVE_SHELL_CAPTURE_DONE", flush=True)


if __name__ == "__main__":
    main()
