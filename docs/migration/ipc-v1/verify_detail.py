"""Selected-detail contract checks; native domain readers are injected, not reimplemented."""
from copy import deepcopy
import io
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
from detail_reference import Detail, DomainPages, text_chunk
import detail_reference
from paging_reference import Paging
from reference_codec import Rejected


def main():
    checks = []
    def check(name, condition):
        assert condition, name
        checks.append(name)

    def reject(name, action, code):
        try:
            action()
            raise AssertionError(name + ' accepted')
        except Rejected as error:
            check(name, error.code == code)

    template = json.loads(Path('tests/fixtures/diagnostics.errors.jsonl').read_text().splitlines()[0])
    template['message'] = '  Sintetico\nà😀\t' * 2500
    template['record_identity']['fiscal_code'] = 'SYNTHETIC_CF'
    template['context'] *= 140
    records = [deepcopy(template) for _ in range(260)]
    for i, row in enumerate(records):
        row['line'] = i + 1
    records[-2]['field'] = None
    records[-2]['code'] = 'E_RECORD_LENGTH'
    records[-2]['actual'] = {'display': '42', 'raw_hex': None}
    records[-1]['trace'] = 'T2'
    records[-1]['field'] = {'id': 'F2', 'name': 'Intero record', 'start': 1, 'end': 20}
    records[-1]['actual'] = {'display': 'SYNTHETIC_FULL_RECORD', 'raw_hex': '00'}
    file = io.BytesIO(b''.join(json.dumps(row, ensure_ascii=False).encode() + b'\n' for row in records))
    ref = {'schemaVersion': 1, 'instance_id': 'a' * 32, 'run_id': '1'}
    view = Paging('owner', file, ref, 'SYNTHETIC', 'v1', ['T1', 'T2'], 260,
                  {'E_SYNTHETIC_RULE': 259, 'E_RECORD_LENGTH': 1})
    query = view.open_query('owner', {'schemaVersion': 1})
    page = view.page('owner', query['query_ref'], '0')
    # Captured Rust metadata: only F1 is a proper subset, F2 covers the whole record.
    detail = Detail(view, {('T1', 'F1'): (1, 3)},
                    {('T1', 'F1'): 'Guida T1 catturata', ('T2', 'F2'): 'Guida T2 catturata'})
    dto = detail.select('owner', query['query_ref'], page['rows'][0]['diagnostic_ref'])
    selection = dto['selection_ref']
    components = {item['kind']: item for item in dto['components']}
    check('detail preserves byte field coordinates and versioned run', dto['field_position'] == {
        'schemaVersion': 1, 'start': '1', 'end': '3', 'units': 'one_based_inclusive_bytes'} and dto['run'] == ref)
    check('detail has no raw hex or wildcard technical identities', 'raw_hex' not in json.dumps(dto) and
          'SYNTHETIC-A' not in json.dumps(dto) and 'technical_ids' not in json.dumps(dto))
    check('selected identity is purpose-limited to one diagnostic', components['fiscal_code']['preview']['text'] == 'SYNTHETIC_CF')
    check('proper field actual remains exact', components['actual']['preview']['text'] == template['actual']['display'])
    check('long message preview is explicitly incomplete', not components['message']['preview']['complete'])
    message_ref = components['message']['component_ref']
    parts, offset = [], '0'
    while True:
        chunk = detail.read_text('owner', selection, message_ref, offset)
        parts.append(chunk['text'])
        check('text chunk is bounded and advances ' + offset, len(chunk['text'].encode()) <= 16384 and int(chunk['next_offset']) > int(offset))
        if not chunk['has_more']:
            break
        offset = chunk['next_offset']
    check('chunk reassembly preserves Unicode whitespace and exact message', ''.join(parts) == template['message'])
    end = detail.read_text('owner', selection, message_ref, str(len(template['message'].encode())))
    check('exact EOF explicit empty final chunk', end['text'] == '' and not end['has_more'])
    for offset, limit in [('1', 1), ('0', 1), ('0', 2)]:
        reject('UTF8 partial boundary ' + offset + '/' + str(limit), lambda offset=offset, limit=limit:
               text_chunk('😀x', offset, limit), 'IPC_INVALID_MESSAGE')
    check('UTF8 boundary exact emoji', text_chunk('😀x', '0', 4)['text'] == '😀')
    for offset, limit in [('6', 4), ('00', 4), ('0', 0), ('0', 16385), ('0', True)]:
        reject('invalid text range ' + repr((offset, limit)), lambda offset=offset, limit=limit:
               text_chunk('😀x', offset, limit), 'IPC_INVALID_MESSAGE')
    reject('unpaired captured surrogate is technical', lambda: text_chunk('\ud800', '0', 10), 'IPC_TECHNICAL_FAILURE')
    reject('wrong detail owner', lambda: detail.read_text('other', selection, message_ref, '0'), 'IPC_UNAUTHORIZED')
    reject('arbitrary component ref is stale', lambda: detail.read_text('owner', selection, 'path_to_file', '0'), 'IPC_STALE_CONTEXT')
    reject('component ref has no coercion', lambda: detail.read_text('owner', selection, [], '0'), 'IPC_INVALID_MESSAGE')
    context = detail.context_page('owner', selection, '0')
    check('whole-record context is withheld with explicit status', len(context['entries']) == 64 and context['has_more'] and
          all(item['value'] is None and item['visibility'] == 'local_report_only_record_scope' for item in context['entries']))
    # Same synthetic source through a captured proper-subset field is allowed.
    detail.subsets[('T2', 'F2')] = (2, 3)
    context = detail.context_page('owner', selection, '0')
    context_ref = context['entries'][0]['component_ref']
    check('context display exact without raw hex', context['entries'][0]['value']['text'] == '01' and 'raw_hex' not in json.dumps(context))
    detail.context_page('owner', selection, '64')
    reject('only one context page remains retained', lambda: detail.read_text('owner', selection, context_ref, '0'), 'IPC_STALE_CONTEXT')
    check('context final page exact length', len(detail.context_page('owner', selection, '128')['entries']) == 12)
    reject('context page cannot exceed64', lambda: detail.context_page('owner', selection, '0', 65), 'IPC_INVALID_MESSAGE')
    view.page('owner', query['query_ref'], '64')
    view.page('owner', query['query_ref'], '128')
    check('selected pin survives viewport eviction', detail.read_text('owner', selection, message_ref, '0', 16)['text'] == '  Sintetico\nà')

    calls = []
    captured = ('00', '01', '␠A␠', '😀' * 500)
    def reader(position, limit):
        calls.append((position, limit))
        return (list(captured[:2]), 2) if position is None else (list(captured[position:]), None)
    source = {'kind': 'reference', 'availability': 'verified', 'expected_sha256': 'b' * 64, 'actual_sha256': 'b' * 64}
    domain = DomainPages(detail, 'owner', selection, source, reader)
    source['availability'] = 'unavailable'
    first = domain.page('owner')
    token = first['entries'][0]['component_ref']
    check('reference snapshot ignores later catalog/source mutations', first['status'] == 'available' and first['expected_sha256'] == first['actual_sha256'] == 'b' * 64)
    check('lookup literals preserve zero padding and bounded native request', [item['preview']['text'] for item in first['entries']] == ['00', '01'] and calls == [(None, 64)])
    second = domain.page('owner', first['next_cursor'])
    check('continuation stays private and values stay exact', calls[-1] == (2, 64) and second['entries'][0]['preview']['text'] == '␠A␠' and second['next_cursor'] is None)
    reject('old domain page text is stale', lambda: detail.read_text('owner', selection, token, '0'), 'IPC_STALE_CONTEXT')
    last_token = second['entries'][-1]['component_ref']
    check('long domain value remains available by bounded text read', not second['entries'][-1]['preview']['complete'] and
          detail.read_text('owner', selection, last_token, '0')['text'] == captured[-1])
    reject('consumed cursor is stale', lambda: domain.page('owner', first['next_cursor']), 'IPC_STALE_CONTEXT')
    for availability in ['degraded', 'unavailable', 'not_captured']:
        before = len(calls)
        unavailable = DomainPages(detail, 'owner', selection, {**source, 'availability': availability}, reader).page('owner')
        check('reference status ' + availability + ' is never empty valid domain', unavailable['status'] == availability and unavailable['entries'] is None and len(calls) == before)
    for kind in ['regex', 'non_enumerable']:
        before = len(calls)
        value = DomainPages(detail, 'owner', selection, {'kind': kind}, reader).page('owner')
        check(kind + ' does not invent a finite enumeration', value['status'] == 'not_enumerable' and value['entries'] is None and len(calls) == before)
    empty = DomainPages(detail, 'owner', selection, {**source, 'availability': 'verified'}, lambda *_: ([], None)).page('owner')
    check('verified empty is a successful read only', empty['status'] == 'available_empty' and empty['entries'] == [])
    unsupported = DomainPages(detail, 'owner', selection, {**source, 'availability': 'verified'}, lambda *_: None).page('owner')
    check('provider unsupported is not empty', unsupported['status'] == 'unsupported' and unsupported['entries'] is None)
    before = len(calls)
    cancelled = DomainPages(detail, 'owner', selection, {**source, 'availability': 'verified'}, reader).page('owner', cancelled=True)
    check('cancelled domain read is explicit not a run verdict', cancelled['status'] == 'cancelled' and cancelled['entries'] is None and len(calls) == before)
    mismatch = DomainPages(detail, 'owner', selection, {**source, 'availability': 'verified', 'actual_sha256': 'c' * 64}, reader)
    reject('hash mismatch cannot become verified domain', lambda: mismatch.page('owner'), 'IPC_TECHNICAL_FAILURE')
    too_big = DomainPages(detail, 'owner', selection, {'kind': 'allowed_values'}, lambda *_: (['X'] * 65, None))
    reject('oversized native page rejected whole', lambda: too_big.page('owner'), 'IPC_TECHNICAL_FAILURE')
    no_advance = DomainPages(detail, 'owner', selection, {'kind': 'range'}, lambda *_: ([], 1))
    reject('empty page cannot promise a continuation', lambda: no_advance.page('owner'), 'IPC_TECHNICAL_FAILURE')
    def fail_reader(*_):
        raise OSError('synthetic native read failure')
    failed = DomainPages(detail, 'owner', selection, {'kind': 'allowed_values'}, fail_reader)
    reject('native read failure stays technical', lambda: failed.page('owner'), 'IPC_TECHNICAL_FAILURE')
    reject('unknown trusted source kind is technical', lambda: DomainPages(detail, 'owner', selection, {'kind': 'invented'}, reader), 'IPC_TECHNICAL_FAILURE')
    declared = DomainPages(detail, 'owner', selection, {'kind': 'range', 'conditional': True}, lambda *_: (['9' * 70, '1' + '0' * 70], None))
    value = declared.page('owner')
    check('arbitrary precision range literals and conditional provenance are retained', value['conditional'] and value['kind'] == 'range' and value['entries'][1]['preview']['text'] == '1' + '0' * 70)
    fallback = DomainPages(detail, 'owner', selection, {'kind': 'fallback'}, lambda *_: (['F'], None)).page('owner')
    check('fallback is distinct from verified dataset membership', fallback['kind'] == 'fallback' and fallback['actual_sha256'] is None)
    reject('previous source reader cannot replay after source change', lambda: declared.page('owner'), 'IPC_STALE_CONTEXT')

    page = view.page('owner', query['query_ref'], '258', 2)
    length = detail.select('owner', query['query_ref'], page['rows'][0]['diagnostic_ref'])
    check('record length count is metadata not a record payload', length['field_position'] is None and length['actual_visibility'] == 'available' and
          next(item for item in length['components'] if item['kind'] == 'actual')['preview']['text'] == '42')
    reject('new selection invalidates previous details', lambda: detail.read_text('owner', selection, message_ref, '0'), 'IPC_STALE_CONTEXT')
    full = detail.select('owner', query['query_ref'], page['rows'][1]['diagnostic_ref'])
    check('Full guide comes from selected trace rather than trace-only scope', next(item for item in full['components'] if item['kind'] == 'guide')['preview']['text'] == 'Guida T2 catturata')
    check('field range mismatch cannot expose whole-record actual', full['actual_visibility'] == 'local_report_only_record_scope' and 'SYNTHETIC_FULL_RECORD' not in json.dumps(full))
    previous = detail.selection['ref']
    original_nonce = detail_reference.secrets.token_hex
    try:
        def fail_nonce(_):
            raise OSError('synthetic entropy failure')
        detail_reference.secrets.token_hex = fail_nonce
        reject('selection mint failure is technical', lambda: detail.select('owner', query['query_ref'], page['rows'][0]['diagnostic_ref']), 'IPC_TECHNICAL_FAILURE')
        check('failed candidate preserves previous selection', detail.selection['ref'] == previous)
    finally:
        detail_reference.secrets.token_hex = original_nonce
    view.open_query('owner', {'schemaVersion': 1, 'line': '1'})
    reject('new query invalidates pinned selection', lambda: detail.matching('owner', full['selection_ref']), 'IPC_STALE_CONTEXT')
    print(json.dumps({'passed': True, 'cases': len(checks), 'checks': checks,
                      'scope': 'P2-06 contract adapter, injected captured Rust metadata/domain readers; no native bridge qualification'}, indent=2))


if __name__ == '__main__':
    main()
