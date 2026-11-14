"""P2-03 executable state contract. Trusted worker events, no product bridge."""
from copy import deepcopy
from dataclasses import dataclass
import re

from reference_codec import IDENTIFIER, Rejected, require_version

MAX_U64 = 18_446_744_073_709_551_615
INSTANCE = re.compile(r"[a-f0-9]{32}", re.ASCII)
RUN_ID = re.compile(r"[1-9][0-9]{0,19}", re.ASCII)


def validate_ref(ref):
    if not isinstance(ref, dict) or set(ref) != {"schemaVersion", "instance_id", "run_id"}:
        raise Rejected("IPC_INVALID_MESSAGE")
    require_version(ref)
    instance, run_id = ref["instance_id"], ref["run_id"]
    if (not isinstance(instance, str) or not INSTANCE.fullmatch(instance)
            or not isinstance(run_id, str) or not RUN_ID.fullmatch(run_id)
            or int(run_id) > MAX_U64):
        raise Rejected("IPC_INVALID_MESSAGE")


@dataclass
class Run:
    ref: dict
    request_id: str
    plan_ref: str
    mode: str
    state: str = "preparing"
    started: bool = False
    sealed: bool = False
    cancel_requested: bool = False
    acknowledged: bool = False
    outcome: dict | None = None


class Lifecycle:
    def __init__(self, instance_id, owner):
        # Fixtures inject an instance ID. Production must mint a fresh OS-random
        # nonce once per application instance, never once per validation/trace.
        if not isinstance(instance_id, str) or not INSTANCE.fullmatch(instance_id):
            raise ValueError("invalid fixture instance")
        self.instance_id = instance_id
        self.owner = owner
        self.next_id = 1
        self.current = None
        self.last_report = None
        self.closed = False

    def authorize(self, owner):
        if owner != self.owner:
            raise Rejected("IPC_UNAUTHORIZED")

    def matching(self, ref):
        validate_ref(ref)
        if self.current is None or ref != self.current.ref:
            raise Rejected("IPC_STALE_CONTEXT")
        return self.current

    def live(self, ref):
        run = self.matching(ref)
        if run.state == "terminal":
            raise Rejected("IPC_INVALID_STATE")
        return run

    def start(self, owner, request_id, plan_ref, mode):
        self.authorize(owner)
        if self.closed:
            raise Rejected("IPC_INVALID_STATE")
        for value in [request_id, plan_ref]:
            if not isinstance(value, str) or not IDENTIFIER.fullmatch(value):
                raise Rejected("IPC_INVALID_MESSAGE")
        if not isinstance(mode, str) or mode not in {"local_only", "full"}:
            raise Rejected("IPC_INVALID_MESSAGE")
        if self.current is not None:
            if self.current.state != "terminal":
                raise Rejected("IPC_BUSY")
            if not self.current.acknowledged or request_id == self.current.request_id:
                raise Rejected("IPC_INVALID_STATE")
        if self.next_id > MAX_U64:
            raise Rejected("IPC_TECHNICAL_FAILURE")
        ref = {"schemaVersion": 1, "instance_id": self.instance_id, "run_id": str(self.next_id)}
        self.current = Run(ref, request_id, plan_ref, mode)
        self.next_id += 1
        return deepcopy(ref)

    def enter_worker(self, ref):
        run = self.live(ref)
        if run.started or run.state not in {"preparing", "cancelling"}:
            raise Rejected("IPC_INVALID_STATE")
        run.started = True
        if not run.cancel_requested:
            run.state = "running"

    def cancel(self, owner, ref):
        self.authorize(owner)
        run = self.matching(ref)
        if run.state == "terminal":
            return "already_terminal"
        if run.sealed:
            return "too_late"
        if run.cancel_requested:
            return "already_requested"
        run.cancel_requested = True
        if run.state != "finalizing":
            run.state = "cancelling"
        return "requested"

    def finalizing(self, ref):
        run = self.live(ref)
        if not run.started:
            raise Rejected("IPC_INVALID_STATE")
        run.state = "finalizing"

    def service_completed(self, ref):
        # RunCompleted fixes the service result, but flush/publish/join can
        # still fail. This event never publishes a successful terminal itself.
        self.finalizing(ref)
        self.current.sealed = True

    def fail(self, ref):
        run = self.live(ref)
        run.outcome = {"schemaVersion": 1, "kind": "technical_failure", "error": {
            "schemaVersion": 1, "category": "technical", "code": "IPC_TECHNICAL_FAILURE"}}
        run.state = "terminal"

    def finish_validation(self, ref, status, completeness, truncated, report_available):
        # Trusted producer calls only after its actual owned work is joined.
        run = self.live(ref)
        valid = (run.started and run.sealed
                 and isinstance(status, str) and status in {"completed_valid", "completed_invalid", "cancelled"}
                 and isinstance(completeness, str) and completeness in {"complete", "partial"}
                 and type(truncated) is bool and type(report_available) is bool)
        if valid:
            valid = ((status != "cancelled" and not truncated) or completeness == "partial")
            valid &= report_available == (status != "cancelled" and not truncated)
        if not valid:
            self.fail(ref)
            return
        run.outcome = {"schemaVersion": 1, "kind": "validation", "status": status,
                       "completeness": completeness, "truncated": truncated,
                       "report_available": report_available}
        run.state = "terminal"
        if report_available:
            self.last_report = {"schemaVersion": 1, "run": deepcopy(ref),
                                "plan_ref": run.plan_ref, "mode": run.mode}

    def confirm_not_started(self, ref):
        run = self.live(ref)
        if run.started or not run.cancel_requested:
            raise Rejected("IPC_INVALID_STATE")
        run.outcome = {"schemaVersion": 1, "kind": "not_started", "reason": "cancelled"}
        run.state = "terminal"

    def acknowledge(self, owner, ref):
        self.authorize(owner)
        run = self.matching(ref)
        if run.state != "terminal":
            raise Rejected("IPC_INVALID_STATE")
        run.acknowledged = True

    def snapshot(self, owner, ref=None):
        self.authorize(owner)
        run = self.matching(ref) if ref is not None else self.current
        if run is None:
            return {"schemaVersion": 1, "state": "idle", "run": None, "outcome": None}
        return {"schemaVersion": 1, "run": deepcopy(run.ref), "state": run.state,
                "plan_ref": run.plan_ref, "mode": run.mode,
                "cancel_requested": run.cancel_requested, "outcome": deepcopy(run.outcome)}

    def close(self, owner):
        self.authorize(owner)
        self.closed = True
        if self.current is not None:
            return self.cancel(owner, self.current.ref)
        return "idle"
