#!/usr/bin/env python3
"""Independent normative oracle: never reads Rust output. --write is explicit.
Rows are synthetic; rule obligations/policy/counts are the ratified SIOC component.
"""
from pathlib import Path
from hashlib import sha256
from collections import Counter
from copy import deepcopy
import json
import sys
ROOT=Path(__file__).resolve().parents[2]
DEST=ROOT/'tests/golden/cross-trace/sioc-referential/v1'
VERSION='2026-04-20.references-golden.1'
TRACE_IDS=['1','2','3','4.1','4.2']
RULES=[('2','REL-trace2_to_trace1'),('3','REL-trace3_to_trace2'),('3','REL-trace3_to_trace1'),('4.2','R-4.30-REFERENCES-4.22'),('4.2','REL-trace42_to_trace41')]

def encoded(value):
    return (json.dumps(value,ensure_ascii=False,separators=(',',':'))+'\n').encode()

def row(schema,trace_id,values):
    trace=next(t for t in schema['traces'] if t['id']==trace_id)
    data=bytearray(b' '*trace['record_length'])
    for field_id,value in values.items():
        field=next(f for f in trace['fields'] if f['id']==field_id)
        start,length=field['physical']['start']-1,field['physical']['length']
        value=value.encode('ascii');assert len(value)<=length
        align=field['physical']['alignment']
        if align['kind']=='exact':assert len(value)==length
        padding=bytes([align.get('fill_byte',32)])*(length-len(value))
        value=padding+value if align['kind']=='left_pad' else value+padding
        data[start:start+length]=value
    return bytes(data)

def rows():
    a={'1.1':'E001','1.17':'050001','1.18':'STS001'}
    b={'2.1':'E001','2.49':'050001','2.50':'STS001'}
    c={'3.1':'E001','3.2':'V001','3.35':'050001','3.36':'STS001'}
    d={'4.1':'000001','4.2':'MRA001','4.7':'050','4.8':'001','4.19.1':'STS001'}
    d2={**d,'4.19.1':'STS002'} # Different PK, same two-field parent join.
    e={'4.20':'000001','4.21':'MRA001','4.22':'ACT0000001'}
    e2={**e,'4.22':'ACT0000002','4.29':'31122026','4.30':'ACT0000001'}
    valid={'1':[a],'2':[b],'3':[c],'4.1':[d,d2],'4.2':[e,e2]}
    missing={'1':[],'2':[b],'3':[{**c,'3.1':'E002','3.36':'STS002'}],'4.1':[],
             '4.2':[{**e2,'4.30':'ACT9999999'}]}
    cardinality={'1':[a,a],'2':[b],'3':[c],'4.1':[d,d2],'4.2':[e]}
    return {'valid':valid,'missing':missing,'cardinality':cardinality}

def identity(episode=None):
    # Diagnostic identity preserves the source field's physical 20 bytes
    # (IdentityExtractor contract), independently of logical join-key stripping.
    return dict(sioc_record_id=episode.ljust(20) if episode is not None else None,fiscal_code=None,episode_id=None,technical_ids={})

def error(code,trace,line,items,message,episode=None,expected='record correlato richiesto'):
    return dict(code=code,flow='SIOC',schema_version=VERSION,trace=trace,line=line,
        record_identity=identity(episode),field=None,actual=None,expected=expected,context=items,message=message)

def context(schema,trace_id,values,field_ids):
    record=row(schema,trace_id,values);trace=next(t for t in schema['traces'] if t['id']==trace_id)
    output=[]
    for field_id in field_ids:
        field=next(f for f in trace['fields'] if f['id']==field_id);start,length=field['physical']['start']-1,field['physical']['length']
        output.append(dict(trace=trace_id,field_id=field_id,value=dict(display=record[start:start+length].decode('ascii'),raw_hex=None)))
    return output

def controls(scenario,records):
    output=[]
    for trace,rule in RULES:
        n=len(records[trace]);unknown=scenario=='cardinality' and rule in ('REL-trace2_to_trace1','REL-trace3_to_trace1')
        violations=n if scenario=='missing' else 0
        causes=[]
        if unknown:
            join=rule.removeprefix('REL-')
            causes=[dict(result=dict(kind='unknown',reason='ambiguous_join'),join_id=join,diagnostic_code=None,message_template=None,evaluations=n)]
        output.append(dict(trace_id=trace,rule_id=rule,evaluations=n,satisfied=0 if violations or unknown else n,
                           violations=violations,not_applicable=0,unknown=n if unknown else 0,skipped=0,causes=causes))
    return output

def main():
    assert sys.argv[1:] in ([],['--write'])
    write=sys.argv[1:]==['--write']
    schema=json.loads((DEST/'schema.json').read_bytes())
    # Raw DTOs omit absent optional properties; the authored component supplies
    # all required members. Canonical JSON sorts object keys, preserves arrays,
    # exact integers/Unicode and omits the runtime encoding binding.
    canonical=json.dumps(schema,sort_keys=True,ensure_ascii=False,separators=(',',':')).encode()
    schema_hash=sha256(canonical).hexdigest()
    files={}
    specifications=[]
    for scenario,records in rows().items():
        data={trace:b''.join(row(schema,trace,values)+b'\n' for values in records[trace]) for trace in TRACE_IDS}
        errors=[]
        if scenario=='missing':
            for trace,fields,episode in [('2',['2.1','2.49','2.50'],'E001'),('3',['3.1','3.35','3.36'],'E002'),('3',['3.1','3.35','3.36'],'E002')]:
                errors.append(error('E_MISSING_PARENT_RECORD',trace,1,context(schema,trace,records[trace][0],fields),'Record correlato richiesto assente',episode))
            errors.append(error('E_RELATED_RECORD_MISSING','4.2',1,context(schema,'4.2',records['4.2'][0],['4.30']),'Atto revocante indicato non presente tra gli atti disponibili'))
            errors.append(error('E_MISSING_PARENT_RECORD','4.2',1,context(schema,'4.2',records['4.2'][0],['4.20','4.21']),'Record correlato richiesto assente'))
        if scenario=='cardinality':
            errors=[error('E_DUPLICATE_RECORD_KEY','1',2,[],'Chiave record duplicata; prima occorrenza alla riga 1.','E001',"chiave univoca nel tracciato dell’invio corrente")]
        total=sum(len(values) for values in records.values());invalid=3 if scenario=='missing' else 1 if scenario=='cardinality' else 0
        partial=scenario=='cardinality';cross=controls(scenario,records)
        trace_evidence=[dict(trace_id=trace,coverage='complete',records_processed=len(records[trace]),unprojectable_records=0,
            cross_rules_declared=sum(source==trace for source,rule in RULES),record_keys_checked=len(records[trace]),
            duplicate_record_keys=int(scenario=='cardinality' and trace=='1'),inconclusive_record_keys=0) for trace in TRACE_IDS]
        summary=dict(result='INVALID' if errors else 'VALID',validation_mode='full',flow='SIOC',schema_version=VERSION,
            scope=dict(kind='flow',trace_ids=TRACE_IDS),records_processed=total,valid_records=total-invalid,invalid_records=invalid,
            total_errors=len(errors),errors_by_code=dict(sorted(Counter(item['code'] for item in errors).items())),skipped_rules=0,truncated=False,
            validator_version='0.1.0',schema_hash=schema_hash,reference_data_versions={},
            full_validation=dict(completeness='partial' if partial else 'complete',parameters={'organisation_namespace':dict(type='integer',value='1')},traces=trace_evidence,controls=cross),
            completeness='partial' if partial else 'complete',started_at='1970-01-01T00:00:00Z',elapsed=dict(seconds=0,nanoseconds=0),
            inputs=[dict(trace_id=trace,file_name=f'trace-{trace}.dat',file_size_bytes=len(data[trace]),bytes_read=len(data[trace]),completion=dict(kind='complete',sha256=sha256(data[trace]).hexdigest())) for trace in TRACE_IDS])
        for trace,body in data.items():files[DEST/scenario/f'trace-{trace}.dat']=body
        files[DEST/scenario/'expected.errors.jsonl']=b''.join(encoded(item) for item in errors)
        files[DEST/scenario/'expected.summary.json']=encoded(summary)
        specifications.append(dict(case=scenario,processed=total,invalid=invalid,errors=len(errors),
            motivation={'valid':'Four explicit parent obligations satisfied; multiple administrative parents allowed; guarded revoking act found.',
                        'missing':'Four distinct applicable parent obligations fail plus E012 revoking act negative; source records invalid once each.',
                        'cardinality':'Duplicate source anagraphic PK diagnosed once; scalar joins retain both parents and return Unknown, not first(); one_or_many administrative join remains satisfied.'}[scenario]))
    files[DEST/'oracle-manifest.json']=(json.dumps(dict(schema_canonical_sha256=schema_hash,decision='CNF-SIOC-REFERENTIAL-G0-01',
        expected_source='Independent normative case specification; no Rust output read.',cases=specifications,
        artifacts={str(path.relative_to(DEST)):sha256(body).hexdigest() for path,body in files.items()}),ensure_ascii=False,indent=2)+'\n').encode()
    for path,body in files.items():
        if write:path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(body)
        else:assert path.read_bytes()==body,f'golden differs: {path}'
    print(f'{len(files)} synthetic input/oracle artifacts {"written" if write else "verified"}; normative expectations independent of Rust')

if __name__=='__main__':main()
