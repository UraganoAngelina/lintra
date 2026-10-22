#!/usr/bin/env python3
"""Independent, explicit DSL oracle. Never reads output produced by Rust."""
from pathlib import Path
from collections import Counter
import hashlib
import json
ROOT=Path(__file__).resolve().parents[2]/"tests/golden/dsl/v1"
FLOW="SYNTHETIC-DSL"
IDENTITY=dict(sioc_record_id=None,fiscal_code=None,episode_id=None,technical_ids={})
def encoded(value):
    return (json.dumps(value,ensure_ascii=False,separators=(",",":"))+"\n").encode()
def context(field,raw):
    return dict(trace="T1",field_id=field,value=dict(display=raw,raw_hex=None))
def error(code,line,field,actual,expected,items,message):
    return dict(code=code,flow=FLOW,schema_version="v1",trace="T1",line=line,
                record_identity=IDENTITY,field=field,actual=actual,expected=expected,context=items,message=message)
def main():
    # These are the two specified violations, in source row/rule order.
    errors=[error("E_CONDITIONAL_REQUIRED",1,dict(id="DETAIL",name="Synthetic detail",start=1,end=3),
                  dict(display="   ",raw_hex=None),"valore presente",
                  [context("ORIGIN","05"),context("DETAIL","   ")],
                  'Regola REQUIRED-DETAIL: DETAIL; ORIGIN NOT IN ["01", "02"].'),
            error("E_CROSS_FIELD",2,None,None,"condizione della regola soddisfatta",
                  [context("DETAIL","X  "),context("ORIGIN","05")],
                  'Regola FORBID-X: (DETAIL presente AND DETAIL = "X").')]
    schema_hash=hashlib.sha256((ROOT/"schema.json").read_bytes()).hexdigest()
    for scenario,processed,valid,items in [("valid",3,3,[]),("invalid",4,2,errors)]:
        directory=ROOT/scenario;source=(directory/"input.txt").read_bytes()
        counts=Counter(item["code"] for item in items)
        summary=dict(result="INVALID" if items else "VALID",validation_mode="local_only",flow=FLOW,schema_version="v1",
                     scope=dict(kind="trace",trace_id="T1"),records_processed=processed,valid_records=valid,
                     invalid_records=processed-valid,total_errors=len(items),errors_by_code=dict(sorted(counts.items())),
                     skipped_rules=0,truncated=False,validator_version="golden-test",schema_hash=schema_hash,
                     reference_data_versions={},started_at="1970-01-01T00:00:00Z",elapsed=dict(seconds=0,nanoseconds=0),
                     inputs=[dict(trace_id="T1",file_name="input.txt",file_size_bytes=len(source),bytes_read=len(source),
                                  completion=dict(kind="complete",sha256=hashlib.sha256(source).hexdigest()))])
        (directory/"expected.errors.jsonl").write_bytes(b"".join(map(encoded,items)))
        (directory/"expected.summary.json").write_bytes(encoded(summary))
if __name__=="__main__":
    main()
