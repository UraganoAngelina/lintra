"""Check Full design examples/anchors and existing lifecycle integration, not the Rust service."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
from lifecycle_reference import Lifecycle
from reference_codec import decode, Rejected


def main():
    root = Path(__file__).resolve().parents[3]
    contract = json.loads((Path(__file__).parent / 'full-cases.json').read_text())
    checks, sources = [], {}
    def check(name, value):
        assert value, name
        checks.append(name)
    check('Full matrix has eighteen independent expected cases', len(contract['cases']) == 18 and len({c['id'] for c in contract['cases']}) == 18)
    for case in contract['cases']:
        data = (root / case['source']).read_bytes()
        check('real Rust test anchor ' + case['id'], re.search(r'#\[test\]\s+fn ' + re.escape(case['test']) + r'\(', data.decode()) is not None)
        sources[case['source']] = hashlib.sha256(data).hexdigest()
    for file in sources:
        original = subprocess.check_output(['git', 'show', '3aa0d14:' + file], cwd=root)
        check('qualified Rust test source unchanged ' + file, (root / file).read_bytes() == original)
    for index, example in enumerate(contract['examples']):
        frame = {'schemaVersion': 1, 'request_id': 'full_' + str(index), 'kind': 'request', **example}
        check('Full DTO fits approved wire grammar ' + example['operation'], decode(json.dumps(frame).encode(), 'request') == frame)
    operations = {e['operation'] for e in contract['examples']}
    check('no historical approval or generic filesystem operation', not operations.intersection(contract['forbidden_operations']))
    lifecycle = Lifecycle('a' * 32, 'owner')
    run = lifecycle.start('owner', 'full_1', 'p_prepared', 'full')
    lifecycle.enter_worker(run)
    lifecycle.service_completed(run)
    check('Full service completion waits for writer/join', lifecycle.snapshot('owner', run)['state'] != 'terminal')
    lifecycle.finish_validation(run, 'completed_valid', 'partial', False, True)
    snapshot = lifecycle.snapshot('owner', run)
    check('Full valid+partial remains explicit with one run', snapshot['mode'] == 'full' and snapshot['outcome']['status'] == 'completed_valid' and snapshot['outcome']['completeness'] == 'partial')
    lifecycle.acknowledge('owner', run)
    next_run = lifecycle.start('owner', 'full_2', 'p_second', 'full')
    lifecycle.enter_worker(next_run)
    lifecycle.fail(next_run)
    check('Full technical failure never becomes normative invalid', lifecycle.snapshot('owner', next_run)['outcome']['kind'] == 'technical_failure')
    check('old report keeps old run provenance', lifecycle.last_report['run'] == run)
    print(json.dumps({'passed': True, 'checks': checks, 'rust_test_source_sha256': sources,
                      'native_service_tests': 'NOT_RERUN; unchanged qualified test sources',
                      'scope': 'Full contract examples, source anchors and lifecycle reference only; gate approval pending'}, indent=2))


if __name__ == '__main__':
    main()
