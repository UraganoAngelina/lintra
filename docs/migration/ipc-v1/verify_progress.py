"""P2-04 slow-consumer, stale-result and bounded-sample design scenarios."""
from copy import deepcopy
import json
import sys

sys.dont_write_bytecode = True
from lifecycle_reference import Lifecycle, MAX_U64
from progress_reference import ProgressSlot, accept_progress
from reference_codec import Rejected, decode


def main():
    checks = []

    def record(name, condition):
        assert condition, name
        checks.append(name)

    def fresh():
        manager = Lifecycle("a" * 32, "native-owner")
        ref = manager.start("native-owner", "request_1", "full_plan", "full")
        slot = ProgressSlot(manager, ref)
        manager.enter_worker(ref)
        slot.phase(ref, "local_validation"); slot.trace_started(ref, "trace_a")
        return manager, ref, slot

    def reject(name, action, code):
        try:
            action()
            raise AssertionError(name + " accepted")
        except Rejected as error:
            record(name, error.code == code)

    manager, ref, slot = fresh()
    lease, first = slot.begin_read("native-owner")
    for count in range(1, 10_001):
        assert slot.records(ref, "trace_a", count)
    record("10k producer updates retain a single latest sample", slot.latest["trace_records"] == 10_000
           and set(slot.latest) == {"phase", "trace_ref", "trace_records", "cross_records"})
    record("slow reader snapshot remains immutable", first["trace_records"] == "0")
    reject("only one data reply owned at once", lambda: slot.begin_read("native-owner"), "IPC_BUSY")
    reject("owner checked before reading", lambda: slot.begin_read("foreign"), "IPC_UNAUTHORIZED")
    record("private lease release", slot.end_read(lease))
    second_lease, newest = slot.begin_read("native-owner")
    record("next read observes latest, no replay backlog", newest["trace_records"] == "10000")
    record("old completion cannot free new reply", not slot.end_read(lease))
    reject("new reply remains owned", lambda: slot.begin_read("native-owner"), "IPC_BUSY")
    slot.end_read(second_lease)

    slot.trace_started(ref, "trace_b"); slot.records(ref, "trace_b", 3)
    lease, trace_b = slot.begin_read("native-owner"); slot.end_read(lease)
    record("new trace count is not a cumulative Full total", trace_b["trace_ref"] == "trace_b" and trace_b["trace_records"] == "3")
    slot.phase(ref, "cross_record_validation"); slot.cross_records(ref, MAX_U64)
    lease, cross = slot.begin_read("native-owner")
    record("PASS2 counter is exact u64 text with no invented trace", cross["cross_records"] == "18446744073709551615"
           and cross["trace_ref"] is None and cross["trace_records"] == "0")
    slot.end_read(lease)
    packet = {"schemaVersion": 1, "request_id": "progress_read", "kind": "result", "payload": cross}
    raw = json.dumps(packet, separators=(",", ":")).encode()
    record("fixed progress DTO fits ratified envelope", decode(raw, "response") == packet and len(raw) < 1024)
    record("no data/error push fields in DTO", set(cross) == {"schemaVersion", "run", "sequence", "phase", "trace_ref", "trace_records", "cross_records"})

    held, old_frame = slot.begin_read("native-owner")
    manager.service_completed(ref)
    record("progress after sealed service is ignored", not slot.cross_records(ref, MAX_U64))
    manager.finish_validation(ref, "completed_invalid", "complete", False, True)
    record("control terminal readable with full data lane", manager.snapshot("native-owner", ref)["outcome"]["status"] == "completed_invalid")
    for _ in range(3):
        record("terminal survives repeated state reads " + str(_), manager.snapshot("native-owner", ref)["state"] == "terminal")
    record("late progress cannot reopen terminal view", accept_progress(old_frame, ref, terminal_seen=True) is None)
    slot.end_read(held)
    manager.acknowledge("native-owner", ref)
    next_ref = manager.start("native-owner", "request_2", "full_plan", "full")
    newer_slot = ProgressSlot(manager, next_ref); manager.enter_worker(next_ref)
    before = deepcopy(newer_slot.latest)
    record("old producer cannot poison new progress", not newer_slot.records(ref, "bad_trace", -1)
           and before == newer_slot.latest and not newer_slot.failed)
    record("old data reply cannot overwrite new run", accept_progress(old_frame, next_ref) is None)
    reject("old progress handle cannot read new state", lambda: slot.begin_read("native-owner"), "IPC_STALE_CONTEXT")

    manager, ref, slot = fresh()
    slot.records(ref, "trace_a", 5)
    lease, frame = slot.begin_read("native-owner"); slot.end_read(lease)
    seq = accept_progress(frame, ref)
    record("first current sequence accepted", seq == int(frame["sequence"]))
    record("duplicate/out-of-order sample ignored", accept_progress(frame, ref, last_sequence=seq) is None)
    for key, value in [("sequence", "01"), ("trace_records", str(MAX_U64 + 1)),
                       ("cross_records", 3), ("phase", "unknown"), ("trace_ref", "../file")]:
        reject("bad typed progress " + key, lambda key=key, value=value: accept_progress({**frame, key: value}, ref), "IPC_INVALID_MESSAGE")
    reject("no bulk diagnostic field", lambda: accept_progress({**frame, "diagnostics": []}, ref), "IPC_INVALID_MESSAGE")
    reject("version mismatch", lambda: accept_progress({**frame, "schemaVersion": 2}, ref), "IPC_UNSUPPORTED_VERSION")
    reject("malformed RunRef", lambda: accept_progress({**frame, "run": {**ref, "run_id": 1}}, ref), "IPC_INVALID_MESSAGE")

    for name, broken in [
        ("counter regression", lambda s, r: s.records(r, "trace_a", 4)),
        ("wrong trace", lambda s, r: s.records(r, "trace_b", 6)),
        ("counter overflow", lambda s, r: s.records(r, "trace_a", MAX_U64 + 1)),
        ("boolean counter", lambda s, r: s.records(r, "trace_a", True)),
        ("phase regression", lambda s, r: s.phase(r, "preflight")),
    ]:
        manager, ref, slot = fresh(); slot.records(ref, "trace_a", 5)
        before = deepcopy(slot.latest)
        record(name + " fails closed without corrupting sample", not broken(slot, ref) and slot.failed and slot.latest == before)
        reject(name + " classified technical", lambda: slot.begin_read("native-owner"), "IPC_TECHNICAL_FAILURE")
        manager.fail(ref)
        record(name + " does not hide actual worker failure", manager.snapshot("native-owner", ref)["outcome"]["kind"] == "technical_failure")
    manager, ref, slot = fresh(); slot.sequence = MAX_U64
    record("progress sequence cannot wrap", not slot.records(ref, "trace_a", 1) and slot.failed and slot.sequence == MAX_U64)
    manager.service_completed(ref); manager.finish_validation(ref, "completed_valid", "complete", False, True)
    record("progress RPC error is not a fabricated job verdict", manager.current.outcome["kind"] == "validation"
           and manager.current.outcome["status"] == "completed_valid")
    reject("progress error still returned as technical, separately", lambda: slot.begin_read("native-owner"), "IPC_TECHNICAL_FAILURE")
    print(json.dumps({"passed": True, "cases": len(checks), "checks": checks,
                      "scope": "P2-04 deterministic application buffers; no native IPC/thread qualification"}, indent=2))


if __name__ == "__main__":
    main()
