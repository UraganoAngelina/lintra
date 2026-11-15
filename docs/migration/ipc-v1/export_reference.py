"""P2-07 native export-plan contract; does not implement writers or filesystem policy."""
from copy import deepcopy
import secrets

from detail_reference import checked_payload
from lifecycle_reference import validate_ref
from reference_codec import Rejected, require_version


class Exports:
    def __init__(self, owner, reports, choose, inspect, confirm, writer):
        # Callbacks are private native capabilities, never deserialized from IPC.
        self.owner, self.reports = owner, reports
        self.choose, self.inspect, self.confirm, self.writer = choose, inspect, confirm, writer
        self.plan = None

    def authorize(self, owner):
        if owner != self.owner:
            raise Rejected('IPC_UNAUTHORIZED')

    def prepare(self, owner, dto):
        self.authorize(owner)
        if not isinstance(dto, dict) or set(dto) != {'schemaVersion', 'report_ref', 'format'}:
            raise Rejected('IPC_INVALID_MESSAGE')
        require_version(dto)
        if self.plan is not None and self.plan['state'] in {'selecting', 'writing'}:
            raise Rejected('IPC_BUSY')
        if dto['format'] not in ('jsonl', 'csv', 'xlsx') or not isinstance(dto['report_ref'], str):
            raise Rejected('IPC_INVALID_MESSAGE')
        report = deepcopy(self.reports.get(dto['report_ref']))
        if report is None:
            raise Rejected('IPC_STALE_CONTEXT')
        validate_ref(report['run'])
        if report['status'] not in {'completed_valid', 'completed_invalid'} or report['truncated']:
            raise Rejected('IPC_INVALID_STATE')
        previous = self.plan
        intent = {'state': 'selecting'}
        self.plan = intent
        try:
            selected = self.choose(dto['format'])
            if selected is None:
                self.plan = None
                return {'schemaVersion': 1, 'state': 'cancelled', 'reason': 'native_selection_cancelled'}
            assessment = self.inspect(selected, dto['format'])
            if (len(assessment['locations']) != 2 or len(assessment['exists']) != 2
                    or any(value not in ('local', 'remote', 'undetermined') for value in assessment['locations'])
                    or any(type(value) is not bool for value in assessment['exists'])):
                raise ValueError('incoherent native destination assessment')
            warning = any(location != 'local' for location in assessment['locations'])
            overwrite = any(assessment['exists'])
            if warning or overwrite:
                accepted = self.confirm(assessment, warning, overwrite)
                if ((warning and accepted.get('shared') is not True) or
                        (overwrite and accepted.get('overwrite') is not True)):
                    self.plan = None
                    return {'schemaVersion': 1, 'state': 'cancelled', 'reason': 'native_confirmation_declined'}
            if self.plan is not intent:
                raise Rejected('IPC_STALE_CONTEXT')
            token = 'e_' + secrets.token_hex(16)
            if previous is not None and token == previous.get('ref'):
                raise ValueError('export handle collision')
            candidate = {'ref': token, 'state': 'ready', 'report_ref': dto['report_ref'],
                         'report': report, 'format': dto['format'], 'selected': selected,
                         'assessment': deepcopy(assessment), 'warning': warning, 'overwrite': overwrite}
            result = {'schemaVersion': 1, 'export_ref': token, 'report_ref': dto['report_ref'],
                      'run': deepcopy(report['run']), 'format': dto['format'], 'state': 'ready',
                      'destination': self.counts(assessment, warning)}
            checked_payload(result)
        except Rejected:
            self.plan = None
            raise
        except (OSError, ValueError, KeyError, TypeError) as error:
            self.plan = None
            raise Rejected('IPC_TECHNICAL_FAILURE') from error
        self.plan = candidate
        return result

    @staticmethod
    def counts(assessment, warning):
        return {'schemaVersion': 1, 'local_files': str(assessment['locations'].count('local')),
                'remote_files': str(assessment['locations'].count('remote')),
                'undetermined_files': str(assessment['locations'].count('undetermined')),
                'warning_acknowledged': warning}

    def matching(self, owner, export_ref):
        self.authorize(owner)
        if self.plan is None or self.plan.get('ref') != export_ref:
            raise Rejected('IPC_STALE_CONTEXT')
        return self.plan

    def execute(self, owner, dto):
        if not isinstance(dto, dict) or set(dto) != {'schemaVersion', 'export_ref'}:
            raise Rejected('IPC_INVALID_MESSAGE')
        require_version(dto)
        if not isinstance(dto['export_ref'], str):
            raise Rejected('IPC_INVALID_MESSAGE')
        plan = self.matching(owner, dto['export_ref'])
        if plan['state'] != 'ready':
            raise Rejected('IPC_INVALID_STATE')
        # Consume before reinspection/writer invocation; no replay of authorization.
        plan['state'] = 'writing'
        try:
            current = self.inspect(plan['selected'], plan['format'])
            if current != plan['assessment']:
                raise Rejected('IPC_STALE_CONTEXT')
            native = self.writer(plan['report'], plan['selected'], plan['format'],
                                 plan['overwrite'], plan['warning'], deepcopy(current))
            # Writer provides per-file facts, including successful rename followed by failed sync.
            published = native['published_roles']
            if (len(published) != len(set(published)) or not set(published) <= {'report', 'summary'}
                    or type(native['cancelled']) is not bool or type(native['cleanup_failed']) is not bool
                    or type(native['success']) is not bool or type(native['durability_confirmed']) is not bool
                    or (native['success'] and (set(published) != {'report', 'summary'} or native['cancelled']
                                               or native['cleanup_failed'] or not native['durability_confirmed']))):
                raise ValueError('incoherent native export receipt')
            result = {'schemaVersion': 1, 'export_ref': plan['ref'], 'report_ref': plan['report_ref'],
                      'run': deepcopy(plan['report']['run']), 'format': plan['format'],
                      'state': 'succeeded' if native['success'] else 'cancelled' if native['cancelled'] else 'failed',
                      'published_roles': published, 'cleanup_failed': native['cleanup_failed'],
                      'durability_confirmed': native['durability_confirmed'],
                      'destination': self.counts(current, plan['warning']),
                      'error': None if native['success'] or native['cancelled'] else
                               {'schemaVersion': 1, 'category': 'technical', 'code': 'IPC_TECHNICAL_FAILURE'}}
            checked_payload(result)
            plan['receipt'] = deepcopy(result)
        except Rejected:
            plan['state'] = 'failed'
            raise
        except (OSError, ValueError, KeyError, TypeError) as error:
            plan['state'] = 'failed'
            # Unexpected adapter failure cannot claim rollback or no publication.
            raise Rejected('IPC_TECHNICAL_FAILURE') from error
        plan['state'] = result['state']
        return result

    def close(self, owner):
        self.authorize(owner)
        if self.plan is not None and self.plan['state'] == 'writing':
            raise Rejected('IPC_BUSY')
        self.plan = None
