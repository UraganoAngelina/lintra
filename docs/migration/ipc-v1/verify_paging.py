"""P2-05 page contracts over private synthetic JSONL; never loads a dataset in JS."""
from collections import Counter
from copy import deepcopy
import hashlib
import io
import json
from pathlib import Path
import sys
import tempfile

sys.dont_write_bytecode = True
from paging_reference import Paging
import paging_reference
from reference_codec import Rejected


def main():
    checks = []
    template = json.loads(Path('tests/fixtures/diagnostics.errors.jsonl').read_text().splitlines()[0])
    ref = {"schemaVersion": 1, "instance_id": "a" * 32, "run_id": "1"}

    def record(name, condition):
        assert condition, name
        checks.append(name)

    def reject(name, action, code):
        try:
            action()
            raise AssertionError(name + " accepted")
        except Rejected as error:
            record(name, error.code == code)

    def row(i):
        value = deepcopy(template)
        value['line'] = i + 1
        value['trace'] = 'T1' if i < 256 else 'T2'
        value['record_identity']['fiscal_code'] = 'SYNTH_A' if i % 2 == 0 else 'SYNTH_B'
        value['record_identity']['sioc_record_id'] = 'record_' + str(i)
        return value

    def server(file, total=512, codes=None):
        return Paging('native-owner', file, ref, 'SYNTHETIC', 'v1', ['T1', 'T2'], total,
                      codes if codes is not None else {'E_SYNTHETIC_RULE': total} if total else {})

    with tempfile.TemporaryDirectory(prefix='lintra-p2-paging-') as folder:
        path = Path(folder) / 'synthetic.jsonl'
        with path.open('wb') as file:
            for i in range(512):
                file.write(json.dumps(row(i), ensure_ascii=False, separators=(',', ':')).encode() + b'\n')
        original = hashlib.sha256(path.read_bytes()).hexdigest()
        with path.open('rb') as file:
            view = server(file)
            query = view.open_query('native-owner', {'schemaVersion': 1})
            record('counts distinguish matched and total diagnostics', query['total_matches'] == '512' and query['total_diagnostics'] == '512')
            first = view.page('native-owner', query['query_ref'], '0')
            record('default page64 is explicit bounded data', len(first['rows']) == 64 and first['next_offset'] == '64' and first['has_more'])
            record('row projection contains no raw/context/identities', set(first['rows'][0]) == {'schemaVersion', 'diagnostic_ref', 'ordinal', 'line', 'trace_ref', 'code', 'field'})
            initial_token = first['rows'][0]['diagnostic_ref']
            view.page('native-owner', query['query_ref'], '64')
            record('two viewport pages retain valid selection tokens', view.selected('native-owner', query['query_ref'], initial_token)['line'] == 1)
            view.page('native-owner', query['query_ref'], '128')
            record('row-token registry bounded to two pages', len(view.pages) == 2 and sum(map(len, view.pages.values())) <= 256)
            reject('evicted page tokens are stale', lambda: view.selected('native-owner', query['query_ref'], initial_token), 'IPC_STALE_CONTEXT')
            for limit, expected in [(1, 1), (127, 127), (128, 128)]:
                page = view.page('native-owner', query['query_ref'], '0', limit)
                record('page row boundary ' + str(limit), len(page['rows']) == expected)
            for limit in [0, 129, -1, True, '128']:
                reject('invalid limit ' + repr(limit), lambda limit=limit: view.page('native-owner', query['query_ref'], '0', limit), 'IPC_INVALID_MESSAGE')
            for offset in ['-1', '01', '513', '18446744073709551616', 0, True, '١']:
                reject('invalid ordinal offset ' + repr(offset), lambda offset=offset: view.page('native-owner', query['query_ref'], offset), 'IPC_INVALID_MESSAGE')
            end = view.page('native-owner', query['query_ref'], '512')
            record('exact end has empty page with no fake run status', end['rows'] == [] and not end['has_more'] and 'status' not in end)
            tail = view.page('native-owner', query['query_ref'], '500', 128)
            record('tail continuation is exact', len(tail['rows']) == 12 and tail['next_offset'] == '512' and not tail['has_more'])
            reject('foreign owner cannot read', lambda: view.page('other', query['query_ref'], '0'), 'IPC_UNAUTHORIZED')
            reject('foreign owner cannot create query', lambda: view.open_query('other', {'schemaVersion': 1}), 'IPC_UNAUTHORIZED')
            cases = [
                ({'fiscal_code': 'SYNTH_A'}, '256'),
                ({'fiscal_code': 'synth_a'}, '0'),
                ({'fiscal_code': ' SYNTH_A'}, '0'),
                ({'code': 'E_SYNTHETIC_RULE', 'field': 'F1', 'record': 'record_6', 'line': '0007'}, '1'),
                ({'line': '0' * 100 + '7'}, '1'),
                ({'trace_ref': 't1'}, '256'),
                ({'trace_ref': 't0', 'line': '300'}, '0'),
            ]
            for filters, expected in cases:
                old = query
                query = view.open_query('native-owner', {'schemaVersion': 1, **filters})
                record('exact AND filter ' + repr(filters), query['total_matches'] == expected and query['total_diagnostics'] == '512')
                reject('query replacement invalidates old handle ' + str(len(checks)), lambda old=old: view.page('native-owner', old['query_ref'], '0'), 'IPC_STALE_CONTEXT')
            before = view.query['ref']
            for filters in [{'schemaVersion': 1, 'sql': 'SELECT'}, {'schemaVersion': 1, 'line': '0'},
                            {'schemaVersion': 1, 'line': 1}, {'schemaVersion': 1, 'fiscal_code': None}]:
                reject('invalid filter ' + repr(filters), lambda filters=filters: view.open_query('native-owner', filters), 'IPC_INVALID_MESSAGE')
                record('invalid candidate preserves old query ' + str(len(checks)), view.query['ref'] == before)
            reject('unknown trace handle', lambda: view.open_query('native-owner', {'schemaVersion': 1, 'trace_ref': 'not_captured'}), 'IPC_STALE_CONTEXT')
            reject('unknown DTO version', lambda: view.open_query('native-owner', {'schemaVersion': 2}), 'IPC_UNSUPPORTED_VERSION')
            reject('opaque query is not arbitrary offset authority', lambda: view.page('native-owner', 'q9999', '0'), 'IPC_STALE_CONTEXT')
            query = view.open_query('native-owner', {'schemaVersion': 1})
            tokens = []
            for offset in range(0, 512, 128):
                page = view.page('native-owner', query['query_ref'], str(offset), 128)
                tokens.extend(int(r['line']) for r in page['rows'])
            record('all pages preserve file order without duplicates', tokens == list(range(1, 513)))
            previous_ref = view.query['ref']
            previous_pages = deepcopy(view.pages)
            previous_generation = view.query['page_generation']
            original_nonce = paging_reference.secrets.token_hex
            try:
                def unavailable(_):
                    raise OSError('synthetic OS entropy failure')
                paging_reference.secrets.token_hex = unavailable
                reject('failed query handle mint preserves old view', lambda: view.open_query('native-owner', {'schemaVersion': 1}), 'IPC_TECHNICAL_FAILURE')
                record('query identity unchanged on OS nonce failure', view.query['ref'] == previous_ref)
                reject('failed row handle mint publishes no partial page', lambda: view.page('native-owner', previous_ref, '0', 128), 'IPC_TECHNICAL_FAILURE')
                record('failed page preserves handle registry and generation', view.pages == previous_pages and view.query['page_generation'] == previous_generation)
                paging_reference.secrets.token_hex = lambda _: 'c' * 32
                reject('duplicate row handles fail closed', lambda: view.page('native-owner', previous_ref, '0', 128), 'IPC_TECHNICAL_FAILURE')
            finally:
                paging_reference.secrets.token_hex = original_nonce
        record('querying never rewrites canonical JSONL', hashlib.sha256(path.read_bytes()).hexdigest() == original)

        # Scope/count validation cannot stop after the first matching row.
        for name, raw, total, codes in [
            ('malformed tail', json.dumps(row(0)).encode() + b'\n{}\n', 2, {'E_SYNTHETIC_RULE': 2}),
            ('truncated final LF', json.dumps(row(0)).encode(), 1, {'E_SYNTHETIC_RULE': 1}),
            ('scope mismatch', json.dumps({**row(0), 'flow': 'OTHER'}).encode() + b'\n', 1, {'E_SYNTHETIC_RULE': 1}),
            ('count mismatch', json.dumps(row(0)).encode() + b'\n', 2, {'E_SYNTHETIC_RULE': 2}),
            ('code-count mismatch', json.dumps(row(0)).encode() + b'\n', 1, {'E_OTHER': 1}),
        ]:
            view = server(io.BytesIO(raw), total, codes)
            reject(name + ' is technical, not an empty result', lambda: view.open_query('native-owner', {'schemaVersion': 1, 'line': '1'}), 'IPC_TECHNICAL_FAILURE')
            record(name + ' never publishes partial index', view.query is None)
        empty = server(io.BytesIO(b''), 0)
        query = empty.open_query('native-owner', {'schemaVersion': 1})
        record('valid empty report remains a query result', empty.page('native-owner', query['query_ref'], '0')['rows'] == [])

        # Captured metadata can contain long/control/UTF-8 text, not HTML authority.
        raw = b''.join(json.dumps({**row(i), 'field': {**row(i)['field'], 'name': '\u0001è😀' * 128}}, ensure_ascii=False).encode() + b'\n' for i in range(128))
        view = server(io.BytesIO(raw), 128)
        query = view.open_query('native-owner', {'schemaVersion': 1})
        page = view.page('native-owner', query['query_ref'], '0', 128)
        record('long metadata preview is explicit, never a silent full label', all(not r['field']['complete'] and len(r['field']['text'].encode()) <= 256 for r in page['rows']))
        frame = {'schemaVersion': 1, 'request_id': 'x' * 64, 'kind': 'result', 'payload': page}
        record('max page includes envelope/escaping within1MiB', len(json.dumps(frame, ensure_ascii=False).encode()) < 1_048_576)
    print(json.dumps({'passed': True, 'cases': len(checks), 'checks': checks,
                      'scope': 'P2-05 private-file paging reference; canonical Rust validation not reimplemented'}, indent=2))


if __name__ == '__main__':
    main()
