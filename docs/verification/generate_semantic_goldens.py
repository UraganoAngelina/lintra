#!/usr/bin/env python3
"""T-P9-06 independent oracle: explicit contract cases, never Rust output.
Run only deliberately to create/review the new synthetic semantic expectations.
JSON insertion order follows the public report contract; hashes use original bytes.
"""
from collections import Counter
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parents[2] / "tests/golden/semantic/v1"
FLOW = "SYNTHETIC-SEMANTIC"
IDENTITY = dict(sioc_record_id=None, fiscal_code=None, episode_id=None, technical_ids={})
FIELDS = {
    "NUMBER": dict(id="NUMBER", name="Synthetic integer", start=1, end=3),
    "DATE": dict(id="DATE", name="Synthetic date", start=4, end=11),
    "CODE": dict(id="CODE", name="Synthetic domain", start=12, end=14),
}
# Code, expected and prose are reviewed constants of existing validators/factory.
VIOLATIONS = {
    "range": ("E_OUT_OF_RANGE", "un intero nell'intervallo dichiarato, estremi inclusi", "valore fuori intervallo"),
    "date": ("E_INVALID_DATE", "data DDMMYYYY valida, anno da 0001 a 9999", "data non valida"),
    "domain": ("E_INVALID_DOMAIN", "uno dei valori del dominio dichiarato", "valore fuori dominio"),
    "type": ("E_INVALID_TYPE", "almeno una cifra ASCII da 0 a 9", "tipo di dato non valido"),
}
# Explicit expected cases: field order within each line is physically declared.
CASES = [
    (2, "NUMBER", "000", "range"),
    (3, "DATE", "29022023", "date"), (3, "CODE", "B  ", "domain"),
    (4, "NUMBER", "005", "range"), (4, "DATE", "31042026", "date"),
    (4, "CODE", "C  ", "domain"), (5, "NUMBER", "00X", "type"),
]


def encoded(document):
    """Encode exactly one UTF-8/LF compact JSON document; no normalizing reader."""
    return (json.dumps(document, ensure_ascii=False, separators=(",", ":")) + "\n").encode()


def error(line, field_id, actual, violation):
    """Construct the public report shape directly from the reviewed constants."""
    code, expected, description = VIOLATIONS[violation]
    field = FIELDS[field_id]
    return dict(code=code, flow=FLOW, schema_version="v1", trace="T1", line=line,
                record_identity=IDENTITY, field=field, actual=dict(display=actual, raw_hex=None),
                expected=expected, context=[], message=f"Campo {field_id} ({field['name']}): {description}.")


def main():
    schema_hash = hashlib.sha256((ROOT / "schema.json").read_bytes()).hexdigest()
    for scenario, processed, valid, cases in [("valid", 2, 2, []), ("invalid", 5, 1, CASES)]:
        directory = ROOT / scenario
        source = (directory / "input.txt").read_bytes()
        errors = [error(*case) for case in cases]
        counts = Counter(item["code"] for item in errors)
        summary = dict(result="INVALID" if errors else "VALID", validation_mode="local_only",
                       flow=FLOW, schema_version="v1", scope=dict(kind="trace", trace_id="T1"),
                       records_processed=processed, valid_records=valid, invalid_records=processed-valid,
                       total_errors=len(errors), errors_by_code=dict(sorted(counts.items())),
                       skipped_rules=0, truncated=False, validator_version="golden-test",
                       schema_hash=schema_hash, reference_data_versions={},
                       started_at="1970-01-01T00:00:00Z", elapsed=dict(seconds=0, nanoseconds=0),
                       inputs=[dict(trace_id="T1", file_name="input.txt", file_size_bytes=len(source),
                                    bytes_read=len(source), completion=dict(kind="complete", sha256=hashlib.sha256(source).hexdigest()))])
        (directory / "expected.errors.jsonl").write_bytes(b"".join(map(encoded, errors)))
        (directory / "expected.summary.json").write_bytes(encoded(summary))


if __name__ == "__main__":
    main()
