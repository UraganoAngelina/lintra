#!/usr/bin/env python3
"""Reproduce authorized derived schemas; default verifies, --write creates only new artifacts.
No patient data. The supplied FROZEN authoring is read-only provenance, not runtime input.
"""
from copy import deepcopy
from hashlib import sha256
from pathlib import Path
import json
import sys

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / 'schemas/sioc/2026-04-20.structural-minimal.1/flow_schema.json'
FROZEN = ROOT.parent / 'LINTRA_STARTER_KIT_FINAL_20261016/SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/flow_schema.json'
VERSION = '2026-04-20.structural-minimal.2'
SUBSET_VERSION = '2026-04-20.references-golden.1'
DEST = ROOT / 'schemas/sioc' / VERSION
GOLDEN = ROOT / 'tests/golden/cross-trace/sioc-referential/v1'
DECISION = 'CNF-SIOC-REFERENTIAL-G0-01'
PROFILE_HASH = '2592c46889a2c06fc9dd5da5b9220f2f1c2a3ac42985f56bd24e476058128c31'
FROZEN_HASH = '3567bafd3cb9b762becdd46a356454b89df094c7b88c5b1538c97a1bbaf26c45'
RELATIONS = [('2', 'trace2_to_trace1'), ('3', 'trace3_to_trace2'), ('3', 'trace3_to_trace1'), ('4.2', 'trace42_to_trace41')]


def encoded(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode()


def digest(value):
    return sha256(value).hexdigest()


def build():
    source_bytes = SOURCE.read_bytes()
    frozen_bytes = FROZEN.read_bytes()
    assert digest(source_bytes) == PROFILE_HASH, 'approved source profile changed'
    assert digest(frozen_bytes) == FROZEN_HASH, 'FROZEN provenance changed'
    original = json.loads(source_bytes)
    frozen = json.loads(frozen_bytes)
    flow = deepcopy(original)
    flow['schema_version'] = VERSION
    flow['source'].update(derivation_decision=DECISION, parent_schema_version=original['schema_version'],
                          parent_profile_sha256=PROFILE_HASH, status='DERIVED_OPERATIONAL_REVISION')
    ledger = dict(decision=DECISION, schema_version=VERSION, source_profile_sha256=PROFILE_HASH,
                  frozen_flow_sha256=FROZEN_HASH, normative_source='SIOC_2026-04-20_FROZEN/flow_schema.json',
                  approved_formalization='docs/normative/sioc/rules-and-joins.md', full_normative_compliance=False,
                  rules=[], changes=[], inherited_scope_ledger='schemas/sioc/2026-04-20.structural-minimal.1/scope-ledger.json')
    for trace_id, join_id in RELATIONS:
        trace = next(t for t in flow['traces'] if t['id'] == trace_id)
        join = next(j for j in trace['joins'] if j['id'] == join_id)
        source_index, source_trace = next((i, t) for i, t in enumerate(frozen['traces']) if t['trace_id'] == trace_id)
        join_index, source_join = next((i, j) for i, j in enumerate(source_trace['joins']) if j['id'] == join_id)
        assert source_join['required'] is True
        assert not any(key in source_join for key in ('when', 'condition', 'conditions', 'include_if'))
        assert all(join[key] == source_join[key] for key in ('target_trace', 'source_fields', 'target_fields'))
        expected_cardinality = 'one_or_many' if source_join['cardinality'] == 'at_least_one' else source_join['cardinality']
        assert join['cardinality'] == expected_cardinality
        assert source_join['error_code'] == join['on_missing_record']['error_code'] == 'E_MISSING_PARENT_RECORD'
        assert join['on_missing_record']['policy'] == join['on_missing_trace']['policy'] == 'violation'
        rule_id = 'REL-' + join_id
        rule = dict(id=rule_id, when={'const': True}, assertions=[dict(kind='record_exists', join=join_id)],
                    error_code=source_join['error_code'], message_template=join['on_missing_record']['diagnostic']['message_template'],
                    regulatory_metadata=dict(decision=DECISION, source_join=join_id, source_pointer=f'/traces/{source_index}/joins/{join_index}',
                                             source_required=True, source_error_code=source_join['error_code']))
        trace['rules'].append(rule)
        ledger['rules'].append(dict(rule_id=rule_id, source_pointer=rule['regulatory_metadata']['source_pointer'],
                                    source_join_snapshot=source_join, source_join_sha256=digest(encoded(source_join)),
                                    source_trace=trace_id, target_trace=join['target_trace'], join=join_id,
                                    source_fields=join['source_fields'], target_fields=join['target_fields'], cardinality=join['cardinality'],
                                    applicability={'const': True}, applicability_evidence='Approved FROZEN required=true with no guard; ratification sections 3-4 and DEC-SIOC-MINIMAL-01 structural relationships.',
                                    on_missing_record=join['on_missing_record'], on_missing_trace=join['on_missing_trace'],
                                    stable_code=rule['error_code'], legacy_code=source_join['error_code'],
                                    legacy_origin='Symbolic code supplied by approved source join, not an invented regional numeric code.',
                                    message_template=rule['message_template'], tests=['valid','missing','cardinality','full_missing_target_preflight']))
    act = next(t for t in flow['traces'] if t['id'] == '4.2')
    join = next(j for j in act['joins'] if j['id'] == 'revoking_act')
    before = deepcopy(join)
    join['on_missing_record'] = dict(policy='violation', error_code='E_RELATED_RECORD_MISSING',
                                     diagnostic=dict(code='RELATED_RECORD_ABSENT', message_template='Atto revocante indicato non presente tra gli atti disponibili'))
    join['on_missing_trace'] = dict(policy='unknown', diagnostic=dict(code='RELATED_TRACE_UNAVAILABLE',
                                     message_template="Verifica dell'atto revocante inconcludente: universo non disponibile"))
    ledger['changes'].append(dict(kind='revoking_act_policy', before=before, after=deepcopy(join), source_rule='R-4.30-REFERENCES-4.22',
                                  legacy_code='E012', source='docs/normative/sioc/rules-and-joins.md / DEC-05-SIOC-REVOKING-ACT',
                                  preserved=['4.2->4.2','4.30->4.22','zero_or_many','current_submission','guard','no UDO restriction'],
                                  tests=['valid','missing','cardinality','partial_universe_unknown']))
    # All inherited capabilities/persistence and field/rule declarations remain in .2.
    assert flow['required_capabilities'] == original['required_capabilities']
    assert flow['persistence_policy'] == original['persistence_policy']
    assert next(t for t in flow['traces'] if t['id'] == '1')['joins'] == original['traces'][0]['joins']
    subset, exclusions = subset_of(flow)
    flow_bytes, subset_bytes = encoded(flow), encoded(subset)
    ledger['artifacts'] = {'flow_schema.json': digest(flow_bytes), 'golden_subset_schema.json': digest(subset_bytes)}
    ledger['artifacts'].update({f"trace-{trace['id']}.json": digest(encoded(trace)) for trace in flow['traces']})
    ledger['outstanding_capabilities'] = dict(required_by_complete_profile=flow['required_capabilities'],
        persistence_policy_retained=True, compiler_must_reject_complete_profile=True,
        obligations='Inherited G0 deferrals, unsupported local constructs and persistence.audit remain mandatory outside the explicitly selected golden component; see inherited scope-ledger and subset exclusions.')
    files = {DEST/'flow_schema.json': flow_bytes, DEST/'scope-ledger.json': encoded(ledger),
             GOLDEN/'schema.json': subset_bytes, GOLDEN/'scope-ledger.json': encoded(exclusions)}
    for trace in flow['traces']:
        files[DEST/f"trace-{trace['id']}.json"] = encoded(trace)
    return files


def subset_of(flow):
    subset = deepcopy(flow)
    subset['schema_version'] = SUBSET_VERSION
    subset['source'].update(status='EXPLICIT_TEST_OPERATIONAL_SUBSET', parent_schema_version=VERSION,
                             scope='referential-golden-only', full_normative_compliance=False, parent_profile_sha256=digest(encoded(flow)),
                             exclusions_ledger='tests/golden/cross-trace/sioc-referential/v1/scope-ledger.json')
    subset['required_capabilities'] = ['local.v1', 'expressions.v1', 'joins.v1', 'joins.sets.v1']
    subset['persistence_policy'] = None
    exclusions = dict(decision=DECISION, subset_version=SUBSET_VERSION, parent_schema_version=VERSION, parent_profile_sha256=digest(encoded(flow)),
        purpose='Real approved structural relationships tested with synthetic rows; NOT full SIOC validation.',
        full_profile_capabilities_unchanged=flow['required_capabilities'], full_profile_persistence_unchanged=True,
        excluded_obligations=['persistence.audit and historical storage: retained in complete .2, not executed by this selected component (PERSIST tasks).',
                              'Clinical/reference datasets and inherited G0 deferrals: no replacement data and no semantic acceptance.',
                              'All unselected field checks, named conditions, derived values and non-executable rule ledgers remain in the complete profile.',
                              'Logical_length/composite_segments on selected keys are unsupported: retained in .2, excluded explicitly from this referential component.'],
        selected_controls=[], selected_fields=[], excluded_fields=[], excluded_constraints=[], opaque_spans=[])
    supported = {'regex','allowed_values','date','numeric_range'}
    for trace in subset['traces']:
        selected = set(trace['record_key']['fields']) | {f for join in trace['joins'] for f in join['source_fields']}
        for source in subset['traces']:
            for join in source['joins']:
                if join['target_trace'] == trace['id']:
                    selected.update(join['target_fields'])
        if trace['id'] == '4.2':
            selected.add('4.29')
        fields = []
        for field in trace['fields']:
            target = exclusions['selected_fields'] if field['id'] in selected else exclusions['excluded_fields']
            target.append(dict(trace=trace['id'], field=field['id'], source_field_sha256=digest(encoded(field))))
            if field['id'] not in selected:
                continue
            kept = []
            for index, constraint in enumerate(field['constraints']):
                if constraint['kind'] in supported:
                    kept.append(constraint)
                else:
                    exclusions['excluded_constraints'].append(dict(trace=trace['id'], field=field['id'], index=index,
                        constraint=constraint, reason='Unsupported construct; explicit exclusion from reference-only test component, retained in full profile.'))
            field['constraints'] = kept
            fields.append(field)
        # Opaque spans keep original byte coordinates/record width. They declare
        # only a byte container, never simulate checks for excluded normative fields.
        result, cursor = [], 1
        for field in fields:
            start = field['physical']['start']
            if cursor < start:
                result.append(opaque(trace['id'], cursor, start-cursor, exclusions))
            result.append(field)
            cursor = start + field['physical']['length']
        if cursor <= trace['record_length']:
            result.append(opaque(trace['id'], cursor, trace['record_length']+1-cursor, exclusions))
        trace['fields'] = result
        trace['named_conditions'] = {}
        trace['derived_values'] = {}
        trace['repeated_groups'] = []
        trace['documented_non_executable_rules'] = []
        exclusions['selected_controls'].extend(dict(trace=trace['id'],rule_id=rule['id'],source_rule_sha256=digest(encoded(rule))) for rule in trace['rules'])
    return subset, exclusions


def opaque(trace, start, length, exclusions):
    identifier = f'EXCLUDED-{trace}-{start}'
    exclusions['opaque_spans'].append(dict(trace=trace, field=identifier, start=start, length=length,
        semantics='Uninterpreted excluded byte span, not a regional normative field or a simulated validator.'))
    return dict(id=identifier, name='Excluded normative bytes; uninterpreted in this test subset',
        physical=dict(type='alphanumeric',start=start,length=length,alignment={'kind':'exact'},lexical={'kind':'encoded_text'}),
        missing_value={'kind':'never_missing'},presence=dict(base='optional',required_if=[],forbidden_if=[]),constraints=[],
        regulatory_metadata=dict(test_only_opaque_span=True, decision=DECISION))


def main():
    assert sys.argv[1:] in ([], ['--write']), 'use no arguments to verify or --write to reproduce authorized artifacts'
    write = sys.argv[1:] == ['--write']
    files = build()
    for path, data in files.items():
        if write:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        else:
            assert path.read_bytes() == data, f'artifact differs: {path.relative_to(ROOT)}'
    print(f'{len(files)} authorized derived artifacts {"written" if write else "verified"}; .1/FROZEN untouched')


if __name__ == '__main__':
    main()
