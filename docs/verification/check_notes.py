"""Verify documentary note coverage and source references; no normative execution."""

from collections import Counter
from hashlib import sha256
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
KIT = ROOT.parent / "LINTRA_STARTER_KIT_FINAL_20261016"
ROW = re.compile(
    r"^\| `([^`]+)` / `fields\[(\d+)\]`(?:; PDF p\.(\d+))?"
    r" \| (metadata|regola eseguibile candidata|ambigua) \| `([a-f0-9]{64})`"
)

for flow, directory, expected in (
    ("sioc", "SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN", 135),
    ("cad", "CAD_REVISED_CANDIDATE", 83),
):
    source = {
        trace["trace_id"]: trace
        for path in (KIT / directory).glob("trace_*.json")
        for trace in [json.loads(path.read_text())]
    }
    text = (ROOT / "docs/normative" / flow / "notes-matrix.md").read_text()
    seen, classes = set(), Counter()
    trace_id = None
    for line in text.splitlines():
        heading = re.match(r"## Tracciato (\S+) —", line)
        if heading:
            trace_id = heading[1]
        row = ROW.match(line)
        if not row:
            continue
        field_id, index, page, category, digest = row.groups()
        field = source[trace_id]["fields"][int(index)]
        assert field["id"] == field_id
        key = (trace_id, field_id)
        assert key not in seen, key
        seen.add(key)
        note = field["regulatory_metadata"]["source_note"]
        assert sha256(note.encode()).hexdigest() == digest, key
        if page is not None:
            assert int(page) == field["regulatory_metadata"]["source_page"]
        candidate_kinds = re.search(r"C: (.*?); P:", line)[1]
        listed = [] if candidate_kinds == "nessuno" else candidate_kinds.split(", ")
        assert listed == [item["kind"] for item in field["constraints"]], key
        inventory = line.split(" | ")[-1]
        known_rules = {rule["id"] for rule in source[trace_id]["rules"]}
        listed_rules = (
            inventory.split("; R: ", 1)[1].rstrip(" |").split(", ")
            if "; R: " in inventory else []
        )
        assert set(listed_rules) <= known_rules, key
        if category == "ambigua":
            assert "DEC-05-" in line or any(
                f"{flow}/{trace_id}/{field_id}`" in block
                for block in re.split(r"\n### DEC-05-", text)[1:]
            ), key
        classes[category] += 1
    expected_keys = {
        (trace_id, field["id"])
        for trace_id, trace in source.items()
        for field in trace["fields"]
    }
    assert seen == expected_keys and len(seen) == expected
    for category, count in classes.items():
        assert f"| {category} | {count} |" in text
    for block in re.split(r"\n### (DEC-05-[^\n]+)\n", text)[1:][1::2]:
        references = re.findall(rf"`{flow}/([^/`]+)/([^`]+)`", block)
        fields = [
            next(f for f in source[t]["fields"] if f["id"] == field_id)
            for t, field_id in references
        ]
        assert fields, "DEC-05 without source fields"
        quotes = re.findall(r"(?:^> .*\n?)+", block, re.M)
        assert quotes, "DEC-05 without source quote"
        for quote in quotes:
            literal = "\n".join(line.removeprefix("> ") for line in quote.splitlines())
            assert all(literal in f["regulatory_metadata"]["source_note"] for f in fields)
    print(f"PASS {flow}: {len(seen)} note, hash/indici/constraint/regole/DEC-05 verificati; {dict(classes)}")
