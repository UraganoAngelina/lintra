"""P2-03 interleavings with explicit expected outcomes, without a fake engine."""
from copy import deepcopy
import json
import sys

sys.dont_write_bytecode = True
from lifecycle_reference import Lifecycle, MAX_U64
from reference_codec import Rejected, decode


def main():
    checks = []

    def record(name, condition):
        assert condition, name
        checks.append(name)

    def fresh():
        manager = Lifecycle("a" * 32, "native-owner")
        ref = manager.start("native-owner", "request_1", "bound_plan", "local_only")
        return manager, ref

    def reject(name, manager, action, code):
        before = manager.snapshot("native-owner")
        try:
            action()
            raise AssertionError(name + " accepted")
        except Rejected as error:
            record(name, error.code == code and before == manager.snapshot("native-owner"))

    def finish(manager, ref, status="completed_valid", completeness="complete", truncated=False, report=True):
        manager.service_completed(ref)
        manager.finish_validation(ref, status, completeness, truncated, report)

    manager, ref = fresh()
    reject("concurrent second start", manager,
           lambda: manager.start("native-owner", "request_2", "other_plan", "full"), "IPC_BUSY")
    reject("ack before terminal", manager, lambda: manager.acknowledge("native-owner", ref), "IPC_INVALID_STATE")
    reject("other native owner cannot read", manager, lambda: manager.snapshot("other", ref), "IPC_UNAUTHORIZED")
    reject("other native owner cannot cancel", manager, lambda: manager.cancel("other", ref), "IPC_UNAUTHORIZED")
    copy = deepcopy(ref); copy["extra"] = True
    reject("unknown RunRef field", manager, lambda: manager.snapshot("native-owner", copy), "IPC_INVALID_MESSAGE")
    for identifier in ["0", "01", "-1", "1.0", str(MAX_U64 + 1), 1, True]:
        copy = {**ref, "run_id": identifier}
        reject("invalid run_id " + repr(identifier), manager,
               lambda copy=copy: manager.snapshot("native-owner", copy), "IPC_INVALID_MESSAGE")
    returned = manager.snapshot("native-owner", ref); returned["run"]["run_id"] = "100"
    record("snapshot does not expose mutable internal identity", manager.snapshot("native-owner", ref)["run"] == ref)

    manager, ref = fresh()
    record("pre-start cancel is a request", manager.cancel("native-owner", ref) == "requested")
    manager.confirm_not_started(ref)
    record("pre-start cancellation fabricates no core RunStatus",
           manager.snapshot("native-owner", ref)["outcome"] == {"schemaVersion": 1, "kind": "not_started", "reason": "cancelled"})
    reject("worker cannot enter after confirmed no-start", manager, lambda: manager.enter_worker(ref), "IPC_INVALID_STATE")

    manager, ref = fresh()
    manager.cancel("native-owner", ref); manager.enter_worker(ref)
    reject("entered worker must return actual cancellation outcome", manager,
           lambda: manager.confirm_not_started(ref), "IPC_INVALID_STATE")
    finish(manager, ref, "cancelled", "partial", False, False)
    record("cooperative cancellation remains a validation outcome",
           manager.snapshot("native-owner", ref)["outcome"]["status"] == "cancelled")

    for status, completeness, truncated, report in [
        ("completed_valid", "complete", False, True),
        ("completed_invalid", "complete", False, True),
        ("completed_valid", "partial", False, True),
        ("completed_invalid", "partial", True, False),
    ]:
        manager, ref = fresh(); manager.enter_worker(ref)
        manager.cancel("native-owner", ref)
        record("duplicate cancel " + status + completeness, manager.cancel("native-owner", ref) == "already_requested")
        finish(manager, ref, status, completeness, truncated, report)
        outcome = manager.snapshot("native-owner", ref)["outcome"]
        record("cancel request cannot overwrite actual outcome " + status + completeness,
               outcome["kind"] == "validation" and outcome["status"] == status and outcome["completeness"] == completeness
               and outcome["report_available"] == report)

    manager, ref = fresh(); manager.enter_worker(ref); manager.finalizing(ref)
    record("cancel before final core checkpoint can still be requested", manager.cancel("native-owner", ref) == "requested")
    manager.service_completed(ref)
    record("RunCompleted is not an adapter terminal", manager.snapshot("native-owner", ref)["outcome"] is None)
    record("cancel after service completion is too late", manager.cancel("native-owner", ref) == "too_late")
    manager.fail(ref)
    record("publication failure after core completion is technical",
           manager.snapshot("native-owner", ref)["outcome"]["kind"] == "technical_failure")
    reject("duplicate terminal cannot invent CompletedValid", manager,
           lambda: manager.finish_validation(ref, "completed_valid", "complete", False, True), "IPC_INVALID_STATE")

    for phase in ["preparing", "running", "cancelling", "finalizing"]:
        manager, ref = fresh()
        if phase != "preparing": manager.enter_worker(ref)
        if phase == "cancelling": manager.cancel("native-owner", ref)
        if phase == "finalizing": manager.finalizing(ref)
        manager.fail(ref)
        outcome = manager.snapshot("native-owner", ref)["outcome"]
        record("technical failure from " + phase, outcome["kind"] == "technical_failure" and "status" not in outcome
               and outcome["error"]["category"] == "technical")

    manager, ref = fresh(); manager.enter_worker(ref)
    manager.finish_validation(ref, "completed_valid", "complete", False, True)
    record("missing service boundary fails closed", manager.current.outcome["kind"] == "technical_failure")
    for status, completeness, truncated, report in [
        ("not_a_status", "complete", False, True),
        ("cancelled", "complete", False, False),
        ("completed_valid", "complete", True, False),
        ("completed_invalid", "complete", False, False),
        ("completed_valid", "complete", 0, True),
    ]:
        manager, ref = fresh(); manager.enter_worker(ref)
        finish(manager, ref, status, completeness, truncated, report)
        record("invalid trusted producer shape " + repr((status, truncated, report)), manager.current.outcome["kind"] == "technical_failure")

    manager, first = fresh(); manager.enter_worker(first); finish(manager, first)
    reject("unobserved terminal blocks replacement", manager,
           lambda: manager.start("native-owner", "request_2", "bound_plan", "full"), "IPC_INVALID_STATE")
    manager.acknowledge("native-owner", first)
    reject("immediate start replay is not a new intent", manager,
           lambda: manager.start("native-owner", "request_1", "bound_plan", "full"), "IPC_INVALID_STATE")
    second = manager.start("native-owner", "request_2", "bound_full_plan", "full")
    record("one Full run identity for the whole bundle", second["run_id"] == "2" and manager.current.mode == "full")
    reject("late producer result cannot mutate new run", manager, lambda: manager.fail(first), "IPC_STALE_CONTEXT")
    reject("old client cannot cancel new run", manager, lambda: manager.cancel("native-owner", first), "IPC_STALE_CONTEXT")
    manager.fail(second)
    record("old successful report is not current success",
           manager.current.outcome["kind"] == "technical_failure" and manager.last_report["run"] == first)

    restarted = Lifecycle("b" * 32, "native-owner")
    restarted.start("native-owner", "fresh_request", "bound_plan", "local_only")
    reject("restart invalidates previous instance handles", restarted,
           lambda: restarted.snapshot("native-owner", first), "IPC_STALE_CONTEXT")
    manager = Lifecycle("a" * 32, "native-owner"); manager.next_id = MAX_U64
    ref = manager.start("native-owner", "last_id", "bound_plan", "local_only")
    record("u64 last run id stays exact text", ref["run_id"] == "18446744073709551615")
    manager.fail(ref); manager.acknowledge("native-owner", ref)
    reject("run counter cannot wrap or reuse", manager,
           lambda: manager.start("native-owner", "overflow", "bound_plan", "local_only"), "IPC_TECHNICAL_FAILURE")

    manager, ref = fresh(); manager.enter_worker(ref)
    manager.close("native-owner")
    reject("window close prevents new validation", manager,
           lambda: manager.start("native-owner", "after_close", "bound_plan", "local_only"), "IPC_INVALID_STATE")
    finish(manager, ref, "cancelled", "partial", False, False)
    record("close still waits for actual owned-worker terminal", manager.current.outcome["status"] == "cancelled")
    packet = {"schemaVersion": 1, "request_id": "state_query", "kind": "result",
              "payload": manager.snapshot("native-owner", ref)}
    raw = json.dumps(packet, separators=(",", ":")).encode()
    record("state DTO integrates with ratified byte envelope", decode(raw, "response") == packet and len(raw) < 1024)
    print(json.dumps({"passed": True, "cases": len(checks), "checks": checks,
                      "scope": "P2-03 deterministic design model; no native engine/thread test"}, indent=2))


if __name__ == "__main__":
    main()
