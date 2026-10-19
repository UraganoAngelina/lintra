"""Check T-P0-08 fixture-plan traceability only; does not execute records or a compiler."""

from hashlib import sha256
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
KIT = ROOT.parent / "LINTRA_STARTER_KIT_FINAL_20261016"
raw = (KIT / "SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/flow_schema.json").read_bytes()
source = json.loads(raw)
catalog = json.loads((ROOT / "docs/normative/sioc/fixtures-catalog.json").read_text())
assert catalog["status"] == "DRAFT_INPUT_GAPS_NOT_EXECUTABLE"
assert catalog["source_flow_sha256"] == sha256(raw).hexdigest()
assert catalog["reference_dependencies"] == [d["dataset"] for d in source["reference_data"]]
assert catalog["mandatory_followup"]["task"] == "T-P3-04"
expected_fields = [(t["trace_id"], f["id"], i)
                   for t in source["traces"] for i, f in enumerate(t["fields"])]
actual_fields = [(f["trace_id"], f["field_id"], f["source_index"]) for f in catalog["fields"]]
assert actual_fields == expected_fields
traces = {t["trace_id"]: t for t in source["traces"]}
all_case_ids = set()
case_count = 0
used_templates = set()
for entry in catalog["fields"]:
    field = traces[entry["trace_id"]]["fields"][entry["source_index"]]
    encoded = json.dumps(field, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
    assert entry["source_field_sha256"] == sha256(encoded).hexdigest()
    cases = entry["cases"]
    for case in cases:
        assert case["id"] not in all_case_ids
        all_case_ids.add(case["id"])
        assert case["template"] in catalog["case_templates"]
        used_templates.add(case["template"])
        case_count += 1
    by_template = {c["template"]: c for c in cases if c["template"] != "constraint_witness"}
    if field["presence"]["base"] == "required":
        template = "never_missing" if field["missing_value"]["kind"] == "never_missing" else "required_raw_missing"
        assert template in by_template
    for key, template in (("required_if", "required_condition"), ("forbidden_if", "forbidden_condition")):
        assert (template in by_template) == bool(field["presence"][key])
    dtype = field["physical"]["type"]
    expected_template = {"numeric": "datatype_numeric", "date": "datatype_date", "alphanumeric": "datatype_text_pending"}[dtype]
    assert expected_template in by_template
    constraint_cases = [c for c in cases if c["template"] == "constraint_witness"]
    assert [(c["constraint_index"], c["constraint_kind"]) for c in constraint_cases] == list(enumerate(c["kind"] for c in field["constraints"]))
assert used_templates == catalog["case_templates"].keys()
expected_rules = [(t["trace_id"], r["id"], i)
                  for t in source["traces"] for i, r in enumerate(t["rules"])]
assert [(r["trace_id"], r["rule_id"], r["source_index"]) for r in catalog["rules"]] == expected_rules
for entry in catalog["rules"]:
    rule = traces[entry["trace_id"]]["rules"][entry["source_index"]]
    encoded = json.dumps(rule, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
    assert entry["source_rule_sha256"] == sha256(encoded).hexdigest()
    assert entry["when"] == rule["when"]
    assert entry["source_error_code"] == rule["error_code"]
    assert entry["assertion_kinds"] == [a["kind"] for a in rule["assertions"]]
    assert entry["case_template"] in catalog["rule_templates"]
for path in (ROOT / "docs/normative/fixtures-plan.md", ROOT / "docs/verification/T-P0-08.md"):
    for target in re.findall(r"\[[^\]]+\]\(([^)]+)\)", path.read_text()):
        assert (path.parent / target.split("#")[0]).exists(), (path, target)
assert len(expected_fields) == 135 and len(expected_rules) == 72
print(f"PASS: {len(expected_fields)} campi, {len(expected_rules)} regole, {case_count} specifiche di caso campo; hash/indici/template/lookup dichiarati tracciati")
print("PASS: ricontrollo T-P3-04 esplicito e link locali; nessun record, golden o schema canonico dichiarato eseguibile")
