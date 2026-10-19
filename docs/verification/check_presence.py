import hashlib
import json
from pathlib import Path
import re

repo = Path(__file__).resolve().parents[2]
kit = repo.parent / 'LINTRA_STARTER_KIT_FINAL_20261016'
manifest = {v['path']: v for v in json.loads((kit / 'KIT_MANIFEST.json').read_text())['files']}
counts = {}
for name, relative, expected_count in (
    ('sioc', 'SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN', 135),
    ('cad', 'CAD_REVISED_CANDIDATE', 83),
):
    folder = kit / relative
    document = (repo / 'docs/normative' / name / 'presence-matrix.md').read_text()
    blocks = [json.loads(b) for b in re.findall(r'```json\n(.*?)\n```', document, re.S)]
    flow = json.loads((folder / 'flow_schema.json').read_text())
    assert len(blocks) == 6, name
    assert blocks[0] == {k: v for k, v in flow.items() if k != 'traces'}, name
    traces = {b['trace_context']['trace_id']: b for b in blocks[1:]}
    count = 0
    for path in sorted(folder.glob('trace_*.json')):
        source = json.loads(path.read_text())
        trace = traces.pop(source['trace_id'])
        assert trace['trace_context'] == {k: v for k, v in source.items() if k != 'fields'}, path
        fields = trace['field_policies']
        approved = json.loads((repo / 'docs/normative' / name / (source['trace_id'] + '-fields.json')).read_text())['fields']
        assert len(fields) == len(source['fields']) == len(approved), path
        for index, (policy, original, table) in enumerate(zip(fields, source['fields'], approved)):
            assert policy == {k: original[k] for k in policy}, (path, original['id'])
            assert {'id', 'physical', 'missing_value', 'presence', 'constraints'} <= policy.keys()
            assert table['id'] == original['id'] and table['physical'] == original['physical']
            assert policy['missing_value']['kind'] in {'spaces', 'exact', 'never_missing'}
            assert policy['physical']['alignment']['kind'] in {'left', 'right', 'exact'}
            row = next(line for line in document.splitlines() if line.startswith('| [' + original['id'] + ']'))
            assert f'| {index} |' in row
            assert f"{original['presence']['base']}; R={len(original['presence']['required_if'])}, F={len(original['presence']['forbidden_if'])}" in row
            start, length = original['physical']['start'], original['physical']['length']
            assert f'{start}–{start + length - 1} ({length})' in row
            count += 1
    assert not traces and count == expected_count
    for path in [*folder.glob('trace_*.json'), folder / 'flow_schema.json', folder / 'manifest.json']:
        relative_path = str(path.relative_to(kit))
        data = path.read_bytes()
        digest = hashlib.sha256(data).hexdigest()
        assert len(data) == manifest[relative_path]['bytes']
        assert digest == manifest[relative_path]['sha256'] and digest in document
    counts[name] = count
    print(f'PASS {name}: {count} campi, 5 trace, policy/AST/contesti fedeli; fonti integre')
assert sum(counts.values()) == 218
print('PASS totale: 218/218 campi; nessuna policy normativa modificata')
