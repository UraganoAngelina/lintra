"""Reproduce/lint T-P0-08 authoring draft, not records or Rust compiler.

Requires supplied starter kit. --write regenerates; default verifies without edits.
No access to patient exports.
"""
from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path
import sys
from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT.parent / 'LINTRA_STARTER_KIT_FINAL_20261016/SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/flow_schema.json'
VERSION = '2026-04-20.structural-minimal.1'
DEST = ROOT / 'schemas/sioc' / VERSION
DECISION = 'DEC-SIOC-MINIMAL-01'
REASON = 'Profilo strutturale minimale autorizzato: semantica rinviata; non equivale a conformità normativa completa.'


def normalize(v):
    if isinstance(v, list):
        return [normalize(x) for x in v]
    if not isinstance(v, dict):
        return v
    if set(v) == {'field'} and isinstance(v['field'], dict):
        return normalize(v['field'])
    return {k: normalize(x) for k, x in v.items()}


def deferred(identifier, fields, reason=REASON):
    return dict(id=identifier, fields=fields, policy='not_enforced_in_this_version',
                reason=reason, decision_reference=DECISION)


def policy(required, message):
    p = dict(policy='violation' if required else 'not_applicable',
             diagnostic=dict(code='RELATED_RECORD_ABSENT', message_template=message))
    if required:
        p['error_code'] = 'E_MISSING_PARENT_RECORD'
    return p


def relation(identifier, target, source_fields, target_fields, cardinality, required, message):
    return dict(id=identifier, target_trace=target, source_fields=source_fields,
                target_fields=target_fields, cardinality=cardinality,
                scope='current_submission', namespace_parameters=[],
                on_missing_record=policy(required, message), on_missing_trace=policy(required, message))


def build():
    raw = SOURCE.read_bytes()
    source = json.loads(raw)
    ledger = dict(decision=DECISION, source_flow_sha256=sha256(raw).hexdigest(),
                  schema_version=VERSION, constraints=[], rules=[], groups=[])
    flow = dict(schema_format_version=1, flow_id='SIOC', schema_version=VERSION,
        regulatory_version=source['regulatory_version'],
        source=dict(status='DRAFT', baseline_sha256=sha256(raw).hexdigest(), decision=DECISION,
                    scope='structural-minimal', full_normative_compliance=False,
                    trace_regulatory_version='2025-12-16', deferred_leap_day_anniversary='mar01'),
        encoding=dict(mode='runtime_required', position_unit='byte'),
        required_capabilities=['local.v1', 'expressions.v1', 'joins.v1', 'joins.sets.v1',
                               'derived.substring.v1', 'persistence.audit.v1'],
        parameters=[dict(id='organisation_namespace', type='integer', min=1, max=4294967295)],
        reference_data=[], traces=[], persistence_policy=None)
    for original in source['traces']:
        t = dict(id=original['trace_id'], name=original['trace_name'], record_length=original['record_length'],
                 fields=[], record_key=deepcopy(original['record_key']), joins=[], identity=[],
                 named_conditions=normalize(original['named_conditions']), derived_values={},
                 repeated_groups=[], rules=[], documented_non_executable_rules=[])
        flow['traces'].append(t)
        omissions = t['documented_non_executable_rules']
        for field in original['fields']:
            f = deepcopy(field)
            for key in list(f):
                if key not in ('id', 'name', 'physical', 'missing_value', 'presence', 'constraints', 'regulatory_metadata'):
                    f.setdefault('regulatory_metadata', {})['source_' + key] = f.pop(key)
            p = f['physical']
            p.pop('source_type')
            a = p['alignment']
            if a['kind'] == 'left':
                assert a['pad'] == ' '
                p['alignment'] = dict(kind='right_pad', fill_byte=32, content_policy='strip_trailing_fill')
            elif a['kind'] == 'right':
                assert a['pad'] == '0'
                p['alignment'] = dict(kind='left_pad', fill_byte=48, content_policy='preserve_width')
            else:
                assert a == dict(kind='exact')
            if p['type'] == 'date':
                p['format'] = 'DDMMYYYY'
            else:
                p['lexical'] = dict(kind='encoded_text')
            if f['missing_value']['kind'] == 'spaces':
                f['missing_value'] = dict(kind='spaces')
            if 'meaning' in f['missing_value']:
                f.setdefault('regulatory_metadata', {})['source_missing_meaning'] = f['missing_value'].pop('meaning')
            f['presence'] = normalize(f['presence'])
            f['constraints'] = []
            for index, constraint in enumerate(field['constraints']):
                c = deepcopy(constraint)
                kind = c['kind'].removesuffix('_if_present')
                c['kind'] = kind
                status = 'active'
                if f['id'] in ('1.1', '2.1', '3.1'):
                    status = 'deferred'
                elif kind in ('logical_length', 'date', 'non_empty'):
                    f['constraints'].append(c)
                elif kind in ('digits_only', 'digits_only_if_present'):
                    f['constraints'].append(dict(kind='regex', pattern='^[0-9]+$', match='required', target='logical'))
                elif kind == 'composite_segments' and f['id'] in ('1.17', '2.49', '3.35'):
                    f['constraints'].append(dict(kind=kind, target='logical',
                        segments=[dict(start=1, length=3, check=dict(equals='050'))]))
                    status = 'partial_050_only'
                elif kind == 'positional_bitset':
                    positions = []
                    for pos in range(1, c['length'] + 1):
                        values = [c['forced_positions'][str(pos)]] if str(pos) in c['forced_positions'] else c['allowed_chars']
                        positions.append(dict(start=pos, length=1, values=values,
                                              when=dict(const=True), otherwise_values=values))
                    f['constraints'].append(dict(kind=kind, positions=positions))
                elif kind == 'structure_only':
                    status = 'physical_only'
                else:
                    status = 'deferred'
                ledger['constraints'].append(dict(trace=t['id'], field=f['id'], index=index,
                                                   source=constraint, disposition=status))
                if status in ('deferred', 'partial_050_only'):
                    omissions.append(deferred(f"DEFER-{f['id']}-CONSTRAINT-{index}", [f['id']]))
            if f['id'] in ('1.1', '2.1', '3.1'):
                f['constraints'].append(dict(kind='regex', pattern='^0+$', match='forbidden', target='logical'))
            if f['id'] == '4.7':
                f['constraints'].append(dict(kind='allowed_values', values=['050'], target='logical'))
            t['fields'].append(f)
        for join in original['joins']:
            required = join['required']
            message = ('Record correlato richiesto assente' if required else
                       'Anagrafica presente; dati del ricovero correlato assenti')
            t['joins'].append(relation(join['id'], join['target_trace'], join['source_fields'],
                join['target_fields'], {'at_least_one': 'one_or_many'}.get(join['cardinality'], join['cardinality']),
                required, message))
        if t['id'] in ('1', '2', '3'):
            t['identity'].append(dict(target='sioc_record_id', field=f"{t['id']}.1"))
        for name, value in original['derived_values'].items():
            if value['kind'] == 'substring':
                t['derived_values'][name] = dict(normalize(value), target='logical')
            else:
                omissions.append(deferred('DEFER-DERIVED-' + name, []))
        for group in original['repeated_groups']:
            omissions.append(deferred('DEFER-GROUP-' + group['id'], group['fields']))
            ledger['groups'].append(dict(trace=t['id'], source=group, disposition='deferred'))
        for rule in original['rules']:
            active = rule['id'] in ('R-4.30-REQUIRES-4.29', 'R-4.30-REFERENCES-4.22')
            ledger['rules'].append(dict(trace=t['id'], source=rule, disposition='active' if active else 'deferred'))
            if not active:
                omissions.append(deferred(rule['id'], []))
                continue
            if rule['id'] == 'R-4.30-REQUIRES-4.29':
                assertions = [dict(kind='field_present', field=dict(field='4.29'))]
                code = 'E_CROSS_FIELD'
            else:
                t['joins'].append(relation('revoking_act', '4.2', ['4.30'], ['4.22'],
                    'zero_or_many', False, 'Nessun atto corrispondente'))
                assertions = [dict(kind='record_exists', join='revoking_act')]
                code = 'E_RELATED_RECORD_MISSING'
            t['rules'].append(dict(id=rule['id'], when=normalize(rule['when']), assertions=assertions,
                error_code=code, message_template=rule['id'],
                regulatory_metadata=dict(source_error_code=rule['error_code'])))
        for rule in original['documented_non_executable_rules']:
            omissions.append(deferred(rule['id'], [rule['field']], rule['reason']))
    keys = [dict(trace=t['id'], key='record_key') for t in flow['traces']]
    flow['persistence_policy'] = dict(namespace_parameters=['organisation_namespace'],
        version_compatibility=dict(kind='same_schema_version'), run_uniqueness=keys,
        cross_run_audit=deepcopy(keys), history_identity_invariants=[], snapshot=None,
        temporal_rules=[], storage_protection='external_secure_configuration_required')
    return flow, ledger


def main():
    flow, ledger = build()
    artifacts = {'flow_schema.json': flow, 'scope-ledger.json': ledger,
                 **{f"trace-{t['id']}.json": t for t in flow['traces']}}
    schema = json.loads((ROOT / 'docs/schema-authoring/flow.schema.json').read_text())
    validator = Draft202012Validator(schema)
    validator.validate(flow)
    for t in flow['traces']:
        Draft202012Validator({'$defs': schema['$defs'], '$ref': '#/$defs/trace'}).validate(t)
        cursor = 1
        for f in t['fields']:
            assert f['physical']['start'] == cursor, (t['id'], f['id'])
            cursor += f['physical']['length']
        assert cursor - 1 == t['record_length']
        def walk(node):
            if isinstance(node, list):
                for item in node:
                    walk(item)
            elif isinstance(node, dict):
                if 'field' in node and isinstance(node['field'], str) and set(node) <= {'field', 'trace', 'join'}:
                    owner = t
                    if 'trace' in node:
                        owner = next(v for v in flow['traces'] if v['id'] == node['trace'])
                        join = next(j for j in t['joins'] if j['id'] == node['join'])
                        assert join['target_trace'] == owner['id']
                        assert join['cardinality'] in ('exactly_one', 'zero_or_one')
                    assert node['field'] in {f['id'] for f in owner['fields']}
                if set(node) == {'condition'}:
                    assert node['condition'] in t['named_conditions']
                if 'derived_value' in node:
                    assert node['derived_value'] in t['derived_values']
                for child in node.values():
                    walk(child)
        walk(t)

        fields = {f['id'] for f in t['fields']}
        assert set(t['record_key']['fields']) <= fields
        for j in t['joins']:
            target = next(v for v in flow['traces'] if v['id'] == j['target_trace'])
            assert set(j['source_fields']) <= fields
            assert set(j['target_fields']) <= {f['id'] for f in target['fields']}
            assert len(j['source_fields']) == len(j['target_fields'])
    if sys.argv[1:] == ['--write']:
        DEST.mkdir(parents=True, exist_ok=True)
        for name, value in artifacts.items():
            (DEST / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')
    else:
        assert not sys.argv[1:]
        for name, value in artifacts.items():
            assert json.loads((DEST / name).read_text()) == value, name
    for mutation in ('padding', 'unknown'):
        broken = deepcopy(flow)
        if mutation == 'padding':
            del broken['traces'][0]['fields'][0]['physical']['alignment']
        else:
            broken['traces'][0]['fields'][0]['constraints'] = [dict(kind='invented')]
        assert not validator.is_valid(broken), mutation
    print('PASS: flow + 5 trace, 135 layout/presence, 5 PK, 6 join, ledger riproducibile; 2 casi negativi grammatica')
    print('NOT RUN: lint Rust T-P3-04, validazione record, golden normativi, persistenza runtime')


if __name__ == '__main__':
    main()
