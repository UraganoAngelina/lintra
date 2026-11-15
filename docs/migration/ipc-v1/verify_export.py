"""Export authorization protocol checks with synthetic native-only paths/receipts."""
from copy import deepcopy
import json
import sys

sys.dont_write_bytecode = True
from export_reference import Exports
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

    run = {'schemaVersion': 1, 'instance_id': 'a' * 32, 'run_id': '1'}
    captured = {'run': run, 'status': 'completed_invalid', 'truncated': False, 'private_report': 'SYNTHETIC_REPORT'}
    reports = {'r_native': deepcopy(captured)}
    events = []
    assessment = {'paths': ['/SYNTHETIC_PRIVATE/report.jsonl', '/SYNTHETIC_PRIVATE/report.jsonl.summary.json'],
                  'locations': ['local', 'local'], 'exists': [False, False]}
    native_answer = {'shared': False, 'overwrite': False}
    outcome = {'success': True, 'published_roles': ['report', 'summary'], 'cleanup_failed': False,
               'cancelled': False, 'durability_confirmed': True}
    def choose(format):
        events.append(('choose', format))
        return 'PRIVATE_NATIVE_SELECTION'
    def inspect(selected, format):
        assert selected == 'PRIVATE_NATIVE_SELECTION'
        events.append(('inspect', format))
        return deepcopy(assessment)
    def confirm(value, warning, overwrite):
        events.append(('confirm', warning, overwrite))
        assert value['paths'] == assessment['paths']
        return deepcopy(native_answer)
    def writer(report, selected, format, overwrite, warning, current):
        events.append(('write', format, overwrite, warning, report['run']['run_id']))
        return deepcopy(outcome)
    manager = Exports('owner', reports, choose, inspect, confirm, writer)
    request = {'schemaVersion': 1, 'report_ref': 'r_native', 'format': 'jsonl'}
    def execute(prepared):
        return manager.execute('owner', {'schemaVersion': 1, 'export_ref': prepared['export_ref']})

    prepared = manager.prepare('owner', request)
    check('local accessible export needs no extra privacy confirmation', not any(event[0] == 'confirm' for event in events))
    check('prepare creates no staging or writer call', not any(event[0] == 'write' for event in events))
    check('no selected path crosses IPC', 'SYNTHETIC_PRIVATE' not in json.dumps(prepared) and 'PRIVATE_NATIVE_SELECTION' not in json.dumps(prepared))
    result = execute(prepared)
    check('canonical Rust writer receives captured report and native selection', events[-1] == ('write', 'jsonl', False, False, '1'))
    check('success reports two per-file roles without fake validation status', result['state'] == 'succeeded' and result['published_roles'] == ['report', 'summary'] and 'run_status' not in result)
    check('reinspect occurs before writer', events[-2] == ('inspect', 'jsonl'))
    reject('authorization cannot be replayed', lambda: execute(prepared), 'IPC_INVALID_STATE')
    for extra in [{'path': '/forged'}, {'overwrite': True}, {'shared_confirmed': True}, {'sql': 'x'}]:
        reject('renderer cannot forge ' + next(iter(extra)), lambda extra=extra: manager.prepare('owner', {**request, **extra}), 'IPC_INVALID_MESSAGE')
    reject('execute cannot retarget path', lambda: manager.execute('owner', {'schemaVersion': 1, 'export_ref': prepared['export_ref'], 'path': '/forged'}), 'IPC_INVALID_MESSAGE')
    reject('foreign owner prepare denied', lambda: manager.prepare('other', request), 'IPC_UNAUTHORIZED')
    reject('foreign owner cannot consume ready intent', lambda: manager.execute('other', {'schemaVersion': 1, 'export_ref': prepared['export_ref']}), 'IPC_UNAUTHORIZED')
    reject('unknown report is stale', lambda: manager.prepare('owner', {**request, 'report_ref': 'r_other'}), 'IPC_STALE_CONTEXT')
    reject('DTO version is explicit', lambda: manager.prepare('owner', {**request, 'schemaVersion': 2}), 'IPC_UNSUPPORTED_VERSION')
    reject('format has no arbitrary extension', lambda: manager.prepare('owner', {**request, 'format': 'exe'}), 'IPC_INVALID_MESSAGE')
    for locations in [['remote', 'remote'], ['undetermined', 'undetermined'], ['local', 'remote']]:
        assessment['locations'] = locations
        native_answer.update(shared=False, overwrite=False)
        before = sum(event[0] == 'write' for event in events)
        denied = manager.prepare('owner', request)
        check('warning declined creates no write ' + repr(locations), denied['state'] == 'cancelled' and sum(event[0] == 'write' for event in events) == before)
        native_answer['shared'] = True
        prepared = manager.prepare('owner', request)
        result = execute(prepared)
        check('shared/undetermined permitted after explicit native warning ' + repr(locations), result['state'] == 'succeeded' and result['destination']['warning_acknowledged'])
    assessment.update(locations=['local', 'local'], exists=[False, True])
    native_answer.update(shared=True, overwrite=False)
    check('summary sidecar overwrite needs independent native confirmation', manager.prepare('owner', request)['state'] == 'cancelled')
    native_answer.update(shared=False, overwrite=True)
    prepared = manager.prepare('owner', request)
    execute(prepared)
    check('local overwrite does not invent a shared warning', events[-1] == ('write', 'jsonl', True, False, '1'))
    assessment['locations'] = ['remote', 'remote']
    check('overwrite permission cannot substitute shared warning', manager.prepare('owner', request)['state'] == 'cancelled')
    native_answer.update(shared=True, overwrite=False)
    check('shared warning cannot substitute overwrite', manager.prepare('owner', request)['state'] == 'cancelled')
    native_answer['overwrite'] = True
    prepared = manager.prepare('owner', request)
    check('both confirmations lead to scoped plan', prepared['state'] == 'ready')

    for change in ['path', 'classification', 'exists']:
        prepared = manager.prepare('owner', request)
        old = deepcopy(assessment)
        if change == 'path': assessment['paths'][1] = '/SYNTHETIC_OTHER/summary.json'
        if change == 'classification': assessment['locations'][1] = 'local'
        if change == 'exists': assessment['exists'][0] = True
        before = sum(event[0] == 'write' for event in events)
        reject('reinspect detects changed ' + change, lambda: execute(prepared), 'IPC_STALE_CONTEXT')
        check('changed ' + change + ' creates no staging/write', sum(event[0] == 'write' for event in events) == before)
        reject('changed intent remains consumed ' + change, lambda: execute(prepared), 'IPC_INVALID_STATE')
        assessment.clear(); assessment.update(old)

    prepared = manager.prepare('owner', request)
    reports['r_native']['run']['run_id'] = '2'
    reports['r_native']['private_report'] = 'CHANGED_CURRENT_REPORT'
    result = execute(prepared)
    check('new displayed run cannot retarget captured export', result['run']['run_id'] == '1' and events[-1][-1] == '1')
    reports['r_native'] = deepcopy(captured)
    for status, truncated in [('cancelled', False), ('completed_invalid', True), ('technical_failure', False)]:
        reports['r_native'].update(status=status, truncated=truncated)
        reject('incomplete/technical report refused ' + str((status, truncated)), lambda: manager.prepare('owner', request), 'IPC_INVALID_STATE')
    reports['r_native'] = deepcopy(captured)
    for name, update in [
        ('failure before publish', {'success': False, 'published_roles': [], 'durability_confirmed': False}),
        ('second publication failed', {'success': False, 'published_roles': ['report'], 'durability_confirmed': True}),
        ('rename succeeded but directory sync failed', {'success': False, 'published_roles': ['report'], 'durability_confirmed': False}),
        ('cleanup failed', {'success': False, 'published_roles': [], 'cleanup_failed': True, 'durability_confirmed': False}),
        ('cancelled before publication', {'success': False, 'published_roles': [], 'cancelled': True, 'durability_confirmed': False}),
        ('cancelled after first publication', {'success': False, 'published_roles': ['report'], 'cancelled': True, 'durability_confirmed': True}),
    ]:
        outcome.clear(); outcome.update(success=False, published_roles=[], cleanup_failed=False, cancelled=False, durability_confirmed=False)
        outcome.update(update)
        result = execute(manager.prepare('owner', request))
        check(name + ' preserves exact publication facts', result['published_roles'] == update['published_roles'] and result['cleanup_failed'] == outcome['cleanup_failed'] and result['durability_confirmed'] == outcome['durability_confirmed'] and result['state'] != 'succeeded')
        check(name + ' is separate from normative outcome', result['error'] is None if outcome['cancelled'] else result['error'] == {'schemaVersion': 1, 'category': 'technical', 'code': 'IPC_TECHNICAL_FAILURE'})
    outcome.update(success=True, published_roles=['report'], cancelled=False)
    prepared = manager.prepare('owner', request)
    reject('incoherent success cannot hide missing second file', lambda: execute(prepared), 'IPC_TECHNICAL_FAILURE')
    first = manager.prepare('owner', request)
    second = manager.prepare('owner', {**request, 'format': 'csv'})
    reject('format change invalidates old native intent', lambda: execute(first), 'IPC_STALE_CONTEXT')
    manager.close('owner')
    reject('closing dialog invalidates authorization', lambda: execute(second), 'IPC_STALE_CONTEXT')
    original_confirm = manager.confirm
    def close_during_confirm(*args):
        manager.close('owner')
        return {'shared': True, 'overwrite': True}
    manager.confirm = close_during_confirm
    reject('late native confirmation cannot resurrect closed intent', lambda: manager.prepare('owner', request), 'IPC_STALE_CONTEXT')
    manager.confirm = original_confirm
    original_choose = manager.choose
    manager.choose = lambda _: None
    check('native chooser cancel creates no authorization', manager.prepare('owner', request)['state'] == 'cancelled' and manager.plan is None)
    manager.choose = original_choose
    for format in ['csv', 'xlsx']:
        outcome.update(success=True, published_roles=['report', 'summary'], cancelled=False, cleanup_failed=False, durability_confirmed=True)
        result = execute(manager.prepare('owner', {**request, 'format': format}))
        check('format dispatched in Rust ' + format, result['format'] == format and result['state'] == 'succeeded')
    print(json.dumps({'passed': True, 'cases': len(checks), 'checks': checks,
                      'scope': 'P2-07 native export authorization/receipt reference; no real dialog, writes, atomic rename or durability qualification'}, indent=2))


if __name__ == '__main__':
    main()
