"""Validate authoring schemas/examples; this is not the Lintra compiler or a record validator.

Run with: uv run --cache-dir /tmp/lintra-authoring-uv --no-project
  --with jsonschema==4.26.0 python docs/verification/check_authoring.py
"""

from copy import deepcopy
import hashlib
import json
from pathlib import Path
import re

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[2]
DOC = ROOT / "docs/schema-authoring"
KIT = ROOT.parent / "LINTRA_STARTER_KIT_FINAL_20261016"


def read(path):
    return json.loads(path.read_text())


def walk(value):
    yield value
    if isinstance(value, dict):
        for item in value.values():
            yield from walk(item)
    elif isinstance(value, list):
        for item in value:
            yield from walk(item)


flow_schema = read(DOC / "flow.schema.json")
manifest_schema = read(DOC / "manifest.schema.json")
for schema in (flow_schema, manifest_schema):
    Draft202012Validator.check_schema(schema)
flow_validator = Draft202012Validator(flow_schema)
manifest_validator = Draft202012Validator(manifest_schema)
examples = [read(p) for p in sorted((DOC / "examples/schemas").glob("*.json"))]
assert len(examples) == 3
for example in examples:
    flow_validator.validate(example)
    traces = {t["id"]: t for t in example["traces"]}
    assert len(traces) == len(example["traces"])
    for trace in traces.values():
        ids = {f["id"] for f in trace["fields"]}
        assert len(ids) == len(trace["fields"])
        start = 1
        for field in trace["fields"]:
            assert field["physical"]["start"] == start
            start += field["physical"]["length"]
        assert start - 1 == trace["record_length"]
        assert set(trace["record_key"]["fields"]) <= ids
        for binding in trace["identity"]:
            assert binding["field"] in ids
        for join in trace["joins"]:
            target_ids = {f["id"] for f in traces[join["target_trace"]]["fields"]}
            assert len(join["source_fields"]) == len(join["target_fields"])
            assert set(join["source_fields"]) <= ids and set(join["target_fields"]) <= target_ids
        for node in walk(trace):
            if isinstance(node, dict) and set(node) == {"trace", "field", "join"}:
                join = next(j for j in trace["joins"] if j["id"] == node["join"])
                assert join["target_trace"] == node["trace"]
                assert node["field"] in {f["id"] for f in traces[node["trace"]]["fields"]}
    for dependency in example["reference_data"]:
        data = DOC / "examples" / dependency["path"]
        assert hashlib.sha256(data.read_bytes()).hexdigest() == dependency["sha256"]
        payload = read(data)
        assert payload["dataset_id"] == dependency["dataset_id"]
        assert payload["version"] == dependency["version"]
manifest = read(DOC / "examples/MANIFEST.json")
manifest_validator.validate(manifest)
expected_files = {str(p.relative_to(DOC / "examples"))
                  for folder in ("schemas", "reference-data")
                  for p in (DOC / "examples" / folder).glob("*.json")}
assert set(manifest["files"]) == expected_files
for name, digest in manifest["files"].items():
    assert hashlib.sha256((DOC / "examples" / name).read_bytes()).hexdigest() == digest
print("PASS: 2 JSON Schema meta-validi; 3 flow e manifest validi; layout, join/identity refs e 4 hash verificati")

# Coverage is grammar/example coverage, not runtime proof of the plans.
local = next(e for e in examples if e["flow_id"] == "EXAMPLE-LOCAL")
nodes = [n for e in examples for n in walk(e) if isinstance(n, dict)]
for definition, property_name in (("constraint", "kind"), ("assertion", "kind"),
                                   ("derived", "kind"), ("missing", "kind"),
                                   ("alignment", "kind"), ("lexical", "kind"),
                                   ("temporal", "kind"), ("physical", "type")):
    variants = {v["properties"][property_name]["const"]
                for v in flow_schema["$defs"][definition]["oneOf"]}
    fragment = Draft202012Validator({"$ref": "#/$defs/" + definition,
                                    "$defs": flow_schema["$defs"]})
    observed = {n[property_name] for n in nodes
                if n.get(property_name) in variants and fragment.is_valid(n)}
    assert variants <= observed, (definition, variants - observed)
exprs = {next(iter(v["properties"])) for v in flow_schema["$defs"]["expr"]["oneOf"]}
expr_validator = Draft202012Validator({"$ref": "#/$defs/expr", "$defs": flow_schema["$defs"]})
observed_exprs = {next(iter(n)) for n in nodes if len(n) == 1
                  and next(iter(n)) in exprs and expr_validator.is_valid(n)}
assert exprs <= observed_exprs, exprs - observed_exprs
lowering = read(DOC / "lowering.json")
source_constraints, source_assertions = set(), set()
for directory in ("SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN", "CAD_REVISED_CANDIDATE"):
    flow = read(KIT / directory / "flow_schema.json")
    for trace in flow["traces"]:
        source_constraints.update(c["kind"] for f in trace["fields"] for c in f["constraints"])
        source_assertions.update(a["kind"] for r in trace["rules"] for a in r["assertions"])
assert source_constraints == lowering["source_constraints"].keys()
assert source_assertions == lowering["source_assertions"].keys()
assert exprs == lowering["source_expressions"].keys() | lowering["new_expressions"].keys()
for key, definition in (("source_constraints", "constraint"), ("source_assertions", "assertion")):
    known = {v["properties"]["kind"]["const"] for v in flow_schema["$defs"][definition]["oneOf"]}
    assert all(v["canonical_kind"] is None or v["canonical_kind"] in known for v in lowering[key].values())
print(f"PASS: esempi per ogni tipo di nodo; {len(source_constraints)} constraint/{len(source_assertions)} assertion legacy e {len(exprs)} operatori tracciati")

# Meaningful negative cases against the actual schema, not an imitation validator.
negative = []
def reject(label, change):
    instance = deepcopy(local)
    change(instance)
    assert not flow_validator.is_valid(instance), label
    negative.append(label)

reject("missing format", lambda d: d.pop("schema_format_version"))
reject("string format", lambda d: d.update(schema_format_version="1"))
reject("unsupported format", lambda d: d.update(schema_format_version=2))
reject("unknown top-level execution property", lambda d: d.update(script="forbidden"))
reject("zero field start", lambda d: d["traces"][0]["fields"][0]["physical"].update(start=0))
reject("zero field width", lambda d: d["traces"][0]["fields"][0]["physical"].update(length=0))
reject("decimal deferred", lambda d: d["traces"][0]["fields"][0]["physical"].update(type="decimal"))
reject("alphabetic deferred", lambda d: d["traces"][0]["fields"][0]["physical"].update(type="alphabetic"))
reject("unsupported physical type", lambda d: d["traces"][0]["fields"][0]["physical"].update(type="arbitrary"))
reject("unknown capability", lambda d: d.update(required_capabilities=["arbitrary.v1"]))
reject("reference path traversal", lambda d: d["reference_data"][0].update(path="../outside.json"))
reject("unknown AST operator", lambda d: d["traces"][1]["rules"][0].update(when={"eval":"forbidden"}))
reject("empty and", lambda d: d["traces"][1]["rules"][0].update(when={"and":[]}))
reject("unknown constraint", lambda d: d["traces"][1]["fields"][0].update(constraints=[{"kind":"arbitrary"}]))
reject("join without missing policy", lambda d: d["traces"][0]["joins"][0].pop("on_missing_record"))
reject("join without input policy", lambda d: d["traces"][0]["joins"][0].pop("on_missing_trace"))
reject("join without cardinality", lambda d: d["traces"][0]["joins"][0].pop("cardinality"))
reject("unknown cardinality", lambda d: d["traces"][0]["joins"][0].update(cardinality="first"))
reject("legacy numeric diagnostic", lambda d: d["traces"][1]["rules"][0].update(error_code="E010"))
reject("unknown derivation", lambda d: d["traces"][1]["derived_values"]["age"].update(kind="eval"))
reject("age without leap-day policy", lambda d: d["traces"][1]["derived_values"]["age"].pop("leap_day_anniversary"))
reject("date width mismatch", lambda d: d["traces"][0]["fields"][2]["physical"].update(length=7))
reject("technical identity unnamed", lambda d: d["traces"][1]["identity"][3].pop("technical_name"))
reject("violation without code", lambda d: d["traces"][1]["joins"][0]["on_missing_record"].pop("error_code"))
reject("nonapplicable carrying violation", lambda d: d["traces"][0]["joins"][0]["on_missing_record"].update(error_code="E_EXAMPLE"))
for path in ("../escape.json", "/absolute.json", "C:/drive.json", "a/../escape.json", r"a\escape.json", "a//empty.json", "a/line\n.json", "a/nul\x00.json"):
    bad = deepcopy(manifest)
    bad["files"] = {path: "0" * 64}
    assert not manifest_validator.is_valid(bad), path
    negative.append(path)
print(f"PASS: {len(negative)} casi negativi strutturali rifiutati; nessun test runtime/DSL dichiarato")

for path in (ROOT / "docs/SCHEMA_AUTHORING.md", ROOT / "docs/schema-authoring/README.md"):
    for link in re.findall(r"\[[^\]]+\]\(([^)]+)\)", path.read_text()):
        if not link.startswith("https://"):
            assert (path.parent / link.split("#")[0]).exists(), (path, link)
print("PASS: link locali del contratto e indice artefatti")
