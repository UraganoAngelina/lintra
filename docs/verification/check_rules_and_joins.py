"""Check T-P0-05 documentary coverage and references against received schemas."""

import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
KIT = ROOT.parent / "LINTRA_STARTER_KIT_FINAL_20261016"
CONTEXT_KEYS = (
    "trace_id", "record_key", "joins", "named_conditions", "derived_values",
    "repeated_groups", "documented_non_executable_rules", "rules",
)


def check_references(node, trace_id, traces, owner):
    """Resolve explicit AST field/condition/derived references, including cross-trace."""
    if isinstance(node, list):
        for item in node:
            check_references(item, trace_id, traces, owner)
    elif isinstance(node, dict):
        local_trace = node.get("trace", trace_id)
        assert local_trace in traces, (owner, local_trace)
        field = node.get("field")
        if isinstance(field, str):
            assert field in {f["id"] for f in traces[local_trace]["fields"]}, (owner, field)
        if "condition" in node:
            assert node["condition"] in traces[trace_id]["named_conditions"], owner
        if "derived_value" in node:
            assert node["derived_value"] in traces[trace_id]["derived_values"], owner
        for value in node.values():
            check_references(value, local_trace, traces, owner)


for flow, directory, expected_traces in (
    ("sioc", "SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN", 5),
    ("cad", "CAD_REVISED_CANDIDATE", 5),
):
    traces = {
        d["trace_id"]: d
        for path in sorted((KIT / directory).glob("trace_*.json"))
        for d in [json.loads(path.read_text())]
    }
    doc_path = ROOT / "docs/normative" / flow / "rules-and-joins.md"
    doc = doc_path.read_text()
    blocks = [json.loads(b) for b in re.findall(r"```json\n(.*?)\n```", doc, re.S)]
    expected_block_count = expected_traces + (flow == "cad")
    assert len(blocks) == expected_block_count
    contexts = {b["trace_id"]: b for b in blocks if "trace_id" in b}
    assert len(contexts) == expected_traces and contexts.keys() == traces.keys()
    rule_count = join_count = presence_count = group_count = 0
    all_field_ids = {f["id"] for t in traces.values() for f in t["fields"]}
    for trace_id, trace in traces.items():
        context = contexts[trace_id]
        expected = {key: trace[key] for key in CONTEXT_KEYS}
        expected["conditional_presence"] = [
            {"field": f["id"], "presence": f["presence"]}
            for f in trace["fields"]
            if f["presence"]["required_if"] or f["presence"]["forbidden_if"]
        ]
        assert context == expected, (flow, trace_id, "projection differs")
        local_ids = {f["id"] for f in trace["fields"]}
        assert set(trace["record_key"]["fields"]) <= local_ids
        assert trace["record_key"]["uniqueness_scope"] == "trace"
        assert len({r["id"] for r in trace["rules"]}) == len(trace["rules"])
        for join in trace["joins"]:
            target_ids = {f["id"] for f in traces[join["target_trace"]]["fields"]}
            assert join["source_fields"] and len(join["source_fields"]) == len(join["target_fields"])
            assert set(join["source_fields"]) <= local_ids
            assert set(join["target_fields"]) <= target_ids
            rows = [line for line in doc.splitlines() if line.startswith(f'| {join["id"]} |')]
            assert len(rows) == 1, join["id"]
            columns = rows[0].split(" | ")
            assert columns[2] == join["cardinality"], join["id"]
            assert any(x in columns[3] for x in ("Violation", "Unknown", "NotApplicable", "DEC-"))
            if not join["required"]:
                assert "DEC-05-SIOC-OPTIONAL-PARENT" in columns[3]
            join_count += 1
        for group in trace["repeated_groups"]:
            assert set(group["fields"]) <= local_ids
        for policy in trace["documented_non_executable_rules"]:
            assert set(policy.get("fields", [])) <= all_field_ids
        for item in (trace["rules"], trace["named_conditions"], trace["derived_values"], expected["conditional_presence"]):
            check_references(item, trace_id, traces, (flow, trace_id))
        rule_count += len(trace["rules"])
        presence_count += len(expected["conditional_presence"])
        group_count += len(trace["repeated_groups"])
    if flow == "cad":
        profile = json.loads((KIT / directory / "CAD_RUNTIME_PROFILE.json").read_text())
        assert blocks[-1] == profile
        assert len(profile["compiler_extensions_required"]) == 7
        assert "DEC-05-CAD-SET-CONTRACT" in doc and "DEC-05-CAD-EVENT-CONTEXT" in doc
        assert "DEC-05-CAD-VALUTAZIONE-INIZIALE" in doc
        assert "E_CAD_MISSING_ACCESS_FOR_PROVISION" in doc
    else:
        exists_rules = [r for t in traces.values() for r in t["rules"]
                        if any(a["kind"] == "record_exists" for a in r["assertions"])]
        assert len(exists_rules) == 1
        assert "DEC-05-SIOC-REVOKING-ACT" in doc
        assert "DEC-05-SIOC-ID-2026" in doc
    for label, link in re.findall(r"\[([^\]]+)\]\(([^)]+)\)", doc):
        assert (doc_path.parent / link.split("#")[0]).exists(), (label, link)
    print(f"PASS {flow}: {len(contexts)} trace, {rule_count} regole, {join_count} join, "
          f"{presence_count} presence condizionali, {group_count} gruppi; "
          "proiezioni, riferimenti e policy/DEC verificati")
