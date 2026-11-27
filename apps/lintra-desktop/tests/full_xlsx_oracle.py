"""T-MIG-P6-10: the Full XLSX the desktop saved in the native campaign,
checked cell by cell against the CLI on the same inputs.

Usage: python3 tests/full_xlsx_oracle.py <lintra CLI> <capture receipt.json>

The capture receipt holds the cells of rf.xlsx (read back with zipfile by
native_capture.py). This script rebuilds the campaign's Full bundle and
inputs (same functions and bytes), runs `lintra validate-flow` with the same
parameters (unit=a, year=2026) and requires:
- «Errori»: the header, then one row per CLI diagnostic, in order, with the
  14 report columns taken from the CLI JSONL;
- «Riepilogo»: the CLI summary flattened to JSON Pointer paths (keys in
  order, as Rust's JSON map holds them), except started_at and elapsed
  (another run, another clock) and the bundle_* provenance, which the
  workbook does not carry (it is in the JSON sidecar).
"""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import native_capture  # noqa: E402  (same bundle builder as the campaign)

COLUMNS = ["code", "flow", "schema_version", "trace", "line", "record_id", "fiscal_code", "field_id",
           "field_name", "start", "end", "actual", "expected", "message"]


def text(value):
    return "" if value is None else value if isinstance(value, str) else json.dumps(value)


def cells(row):
    field = row["field"] or {}
    actual = row["actual"] or {}
    identity = row["record_identity"]
    return [text(v) for v in (row["code"], row["flow"], row["schema_version"], row["trace"], row["line"],
                              identity["sioc_record_id"], identity["fiscal_code"], field.get("id"), field.get("name"),
                              field.get("start"), field.get("end"), actual.get("display"), row["expected"], row["message"])]


def flatten(path, value, rows):
    if isinstance(value, dict) and value:
        for key, item in sorted(value.items()):  # serde_json's map is ordered by key
            flatten(f"{path}/{key.replace('~', '~0').replace('/', '~1')}", item, rows)
    elif isinstance(value, list) and value:
        for index, item in enumerate(value):
            flatten(f"{path}/{index}", item, rows)
    else:
        rows.append([path, value if isinstance(value, str) else json.dumps(value, separators=(",", ":"))])


def main():
    cli, receipt = Path(sys.argv[1]), json.loads(Path(sys.argv[2]).read_text())
    sheets = receipt["export"]["rf.xlsx"]["sheets"]
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        schemas, references = native_capture.full_bundle(root / "bundle")
        # Same names and bytes as the campaign: the summary records file names.
        (root / "fullsr.txt").write_bytes(b"01AAXX\n02CCYY\nbad\n")
        (root / "badt.txt").write_bytes(b"01AAZZ\n02BBYY\n")
        flow = next(schemas.iterdir())
        version = next(flow.iterdir())
        run = subprocess.run([str(cli), "validate-flow", "--flow", flow.name, "--schema-version", version.name,
                              "--encoding", "utf8", "--schema-dir", str(schemas), "--reference-data-dir", str(references),
                              "--trace-input", f"S={root / 'fullsr.txt'}", "--trace-input", f"T={root / 'badt.txt'}",
                              "--parameter", "unit=a", "--parameter", "year=2026",
                              "--errors", str(root / "e.jsonl"), "--summary", str(root / "s.json")],
                             capture_output=True, text=True, timeout=120)
        assert run.returncode == 2, (run.returncode, run.stderr)  # INVALID
        rows = [json.loads(line) for line in (root / "e.jsonl").read_text().splitlines()]
        summary = json.loads((root / "s.json").read_text())
    errors = sheets["Errori"]
    assert errors[0] == COLUMNS, errors[0]
    assert errors[1:] == [cells(row) for row in rows], (errors[1:], rows)
    expected = [["property", "value"]]
    flatten("", summary, expected)
    # Another run, another clock; bundle provenance is in the CLI summary and
    # in the desktop's JSON sidecar, not in the workbook (write_run_summary).
    ignored = ("/started_at", "/elapsed", "/bundle_")
    keep = lambda table: [r for r in table if not r[0].startswith(ignored)]
    assert keep(sheets["Riepilogo"]) == keep(expected), (sheets["Riepilogo"], expected)
    print(json.dumps({"diagnostics": len(rows), "codes": [row["code"] for row in rows],
                      "summary_rows": len(expected) - 1, "cli_exit": run.returncode}))
    print("FULL_XLSX_ORACLE_PASS")


if __name__ == "__main__":
    main()
