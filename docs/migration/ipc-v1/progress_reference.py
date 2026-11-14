"""P2-04 single latest sample and one owned response, no producer push queue."""
from copy import deepcopy
import re

from lifecycle_reference import MAX_U64, validate_ref
from reference_codec import IDENTIFIER, Rejected, require_version

PHASES = {"preflight", "local_validation", "cross_record_validation", "finalizing_report"}
PHASE_ORDER = {"preflight": 0, "local_validation": 1, "cross_record_validation": 2, "finalizing_report": 3}
DECIMAL = re.compile(r"0|[1-9][0-9]{0,19}", re.ASCII)


def u64(value):
    return type(value) is int and 0 <= value <= MAX_U64


class ProgressSlot:
    def __init__(self, manager, ref):
        manager.live(ref)
        self.manager = manager
        self.ref = deepcopy(ref)
        self.sequence = 0
        self.latest = {"phase": None, "trace_ref": None, "trace_records": 0, "cross_records": 0}
        self.failed = False
        self.lease = None

    def ready(self, ref):
        # A late old producer cannot poison either a new sample or its error bit.
        return (not self.failed and ref == self.ref and self.manager.current is not None
                and self.manager.current.ref == ref and self.manager.current.started
                and not self.manager.current.sealed and self.manager.current.state != "terminal")

    def replace(self, values):
        if self.sequence == MAX_U64:
            self.failed = True
            return False
        self.latest = values
        self.sequence += 1
        return True

    def phase(self, ref, phase):
        if not self.ready(ref):
            return False
        if not isinstance(phase, str) or phase not in PHASES:
            self.failed = True
            return False
        previous = self.latest["phase"]
        if previous is not None and PHASE_ORDER[phase] < PHASE_ORDER[previous]:
            self.failed = True
            return False
        values = {**self.latest, "phase": phase}
        if phase == "cross_record_validation":
            values.update(trace_ref=None, trace_records=0)
        return self.replace(values)

    def trace_started(self, ref, trace_ref):
        if not self.ready(ref):
            return False
        if (self.latest["phase"] != "local_validation" or not isinstance(trace_ref, str)
                or not IDENTIFIER.fullmatch(trace_ref)):
            self.failed = True
            return False
        return self.replace({**self.latest, "trace_ref": trace_ref, "trace_records": 0})

    def records(self, ref, trace_ref, count):
        if not self.ready(ref):
            return False
        if (self.latest["phase"] != "local_validation" or self.latest["trace_ref"] is None
                or trace_ref != self.latest["trace_ref"] or not u64(count)
                or count < self.latest["trace_records"]):
            self.failed = True
            return False
        return self.replace({**self.latest, "trace_records": count})

    def cross_records(self, ref, count):
        if not self.ready(ref):
            return False
        if (self.latest["phase"] != "cross_record_validation" or not u64(count)
                or count < self.latest["cross_records"]):
            self.failed = True
            return False
        return self.replace({**self.latest, "cross_records": count})

    def begin_read(self, owner):
        self.manager.authorize(owner)
        self.manager.matching(self.ref)
        if self.failed:
            raise Rejected("IPC_TECHNICAL_FAILURE")
        if self.lease is not None:
            raise Rejected("IPC_BUSY")
        packet = {"schemaVersion": 1, "run": deepcopy(self.ref), "sequence": str(self.sequence),
                  "phase": self.latest["phase"], "trace_ref": self.latest["trace_ref"],
                  "trace_records": str(self.latest["trace_records"]),
                  "cross_records": str(self.latest["cross_records"])}
        self.lease = object()  # Native private response ownership, never a wire/C1 token.
        return self.lease, packet

    def end_read(self, lease):
        if lease is not self.lease or lease is None:
            return False
        self.lease = None
        return True


def accept_progress(packet, current_ref, last_sequence=None, terminal_seen=False):
    """UI reference reducer: exact DTO validation before a monotonic update."""
    keys = {"schemaVersion", "run", "sequence", "phase", "trace_ref", "trace_records", "cross_records"}
    if not isinstance(packet, dict) or set(packet) != keys:
        raise Rejected("IPC_INVALID_MESSAGE")
    require_version(packet)
    for key in ["sequence", "trace_records", "cross_records"]:
        value = packet[key]
        if not isinstance(value, str) or not DECIMAL.fullmatch(value) or int(value) > MAX_U64:
            raise Rejected("IPC_INVALID_MESSAGE")
    ref = packet["run"]
    validate_ref(ref)
    if packet["phase"] is not None and (not isinstance(packet["phase"], str) or packet["phase"] not in PHASES):
        raise Rejected("IPC_INVALID_MESSAGE")
    trace = packet["trace_ref"]
    if trace is not None and (not isinstance(trace, str) or not IDENTIFIER.fullmatch(trace)):
        raise Rejected("IPC_INVALID_MESSAGE")
    if (trace is None and packet["trace_records"] != "0") or (
            packet["phase"] == "cross_record_validation" and trace is not None):
        raise Rejected("IPC_INVALID_MESSAGE")
    sequence = int(packet["sequence"])
    if ref != current_ref or terminal_seen or (last_sequence is not None and sequence <= last_sequence):
        return None
    return sequence
