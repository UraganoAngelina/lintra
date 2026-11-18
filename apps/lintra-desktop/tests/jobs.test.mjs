import assert from "node:assert/strict";
import { afterEach, test } from "node:test";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { validationStart, validationState, validationProgress, validationCancel, validationAcknowledge, acceptRunProgress, IpcError } from "../src/ipc.ts";

globalThis.window = { crypto: globalThis.crypto };
afterEach(clearMocks);
const run = { schemaVersion: 1, instance_id: "a".repeat(32), run_id: "1" };
const other = { ...run, run_id: "2" };
const progress = { schemaVersion: 1, run, sequence: "18446744073709551615", phase: "local_validation", trace_ref: "trace_0", trace_records: "18446744073709551615", cross_records: "0" };
const snapshot = { schemaVersion: 1, run, plan_ref: "plan_1", mode: "local_only", state: "terminal", cancel_requested: false,
  outcome: { schemaVersion: 1, kind: "validation", status: "completed_invalid", completeness: "complete", truncated: false, report_available: true } };
function mock(response, observe = () => {}) {
  mockIPC((command, bytes) => {
    assert.equal(command, "lintra_ipc");
    const request = JSON.parse(new TextDecoder().decode(bytes)); observe(request);
    return new TextEncoder().encode(JSON.stringify({ schemaVersion: 1, request_id: request.request_id, kind: "result", payload: response })).buffer;
  });
}
test("job operations send exact DTOs, resynchronize and keep u64 counters as text", async () => {
  mock(run, r => assert.deepEqual(r.payload, { schemaVersion: 1, plan_ref: "plan_1", mode: "local_only" }));
  assert.deepEqual(await validationStart("plan_1", "local_only"), run);
  mock(snapshot, r => assert.equal(r.payload.run, null));
  assert.deepEqual(await validationState(), snapshot);
  mock(progress, r => assert.deepEqual(r.payload, { schemaVersion: 1, run }));
  assert.deepEqual(await validationProgress(run), progress);
  for (const disposition of ["requested", "already_requested", "too_late", "already_terminal"]) {
    mock({ schemaVersion: 1, run, disposition }); assert.equal(await validationCancel(run), disposition);
  }
  mock({ schemaVersion: 1, run, acknowledged: true }); await validationAcknowledge(run);
});
test("terminal outcomes preserve partial, cancelled and technical distinctions", async () => {
  for (const outcome of [snapshot.outcome,
    { ...snapshot.outcome, status: "completed_valid", completeness: "partial" },
    { ...snapshot.outcome, status: "cancelled", completeness: "partial", report_available: false },
    { schemaVersion: 1, kind: "technical_failure", error: { schemaVersion: 1, category: "technical", code: "IPC_TECHNICAL_FAILURE" } },
    { schemaVersion: 1, kind: "not_started", reason: "cancelled" }]) {
    mock({ ...snapshot, cancel_requested: true, outcome });
    assert.deepEqual((await validationState(run)).outcome, outcome);
  }
  for (const payload of [{ ...snapshot, state: "running" }, { ...snapshot, outcome: null },
    { ...snapshot, outcome: { ...snapshot.outcome, status: "cancelled" } },
    { ...snapshot, outcome: { ...snapshot.outcome, truncated: true } },
    { ...snapshot, outcome: { schemaVersion: 1, kind: "not_started", reason: "cancelled" } },
    { ...snapshot, outcome: { ...snapshot.outcome, path: "SYNTHETIC" } }, { ...snapshot, run: other }]) {
    mock(payload); await assert.rejects(() => validationState(run), IpcError);
  }
});
test("progress rejects malformed, stale and bulk data; late samples cannot reopen terminal", async () => {
  for (const payload of [{ ...progress, run: other }, { ...progress, sequence: "01" },
    { ...progress, sequence: "18446744073709551616" }, { ...progress, trace_records: 1 },
    { ...progress, phase: "unknown" }, { ...progress, trace_ref: null },
    { ...progress, phase: "cross_record_validation" }, { ...progress, trace_ref: "a".repeat(65) },
    { ...progress, errors: ["SYNTHETIC"] }]) {
    mock(payload); await assert.rejects(() => validationProgress(run), IpcError);
  }
  assert.equal(acceptRunProgress(progress, run, "18446744073709551614", false), true);
  assert.equal(acceptRunProgress(progress, run, progress.sequence, false), false);
  assert.equal(acceptRunProgress(progress, run, "0", true), false);
  assert.equal(acceptRunProgress(progress, other, null, false), false);
});
test("cancel and ack require the same run; bad handles fail before transport", async () => {
  mock({ schemaVersion: 1, run: other, disposition: "requested" });
  await assert.rejects(() => validationCancel(run), IpcError);
  mock({ schemaVersion: 1, run, disposition: "cancelled" });
  await assert.rejects(() => validationCancel(run), IpcError);
  mock({ schemaVersion: 1, run: other, acknowledged: true });
  await assert.rejects(() => validationAcknowledge(run), IpcError);
  let calls = 0; mock(run, () => calls++);
  for (const invalid of [{ ...run, run_id: "0" }, { ...run, run_id: "01" }, { ...run, instance_id: "A".repeat(32) }, { ...run, path: "SYNTHETIC" }]) {
    await assert.rejects(() => validationState(invalid), IpcError);
    await assert.rejects(() => validationCancel(invalid), IpcError);
  }
  assert.equal(calls, 0);
});
test("Full start sends the exact mode and keeps Full snapshots and partial outcomes distinct", async () => {
  mock(run, r => assert.deepEqual(r.payload, { schemaVersion: 1, plan_ref: "plan_1", mode: "full" }));
  assert.deepEqual(await validationStart("plan_1", "full"), run);
  const full = { ...snapshot, mode: "full", outcome: { ...snapshot.outcome, completeness: "partial" } };
  mock(full);
  assert.deepEqual(await validationState(run), full);
  for (const mode of ["FULL", "flow", "", null]) {
    await assert.rejects(validationStart("plan_1", mode), IpcError);
    mock({ ...full, mode }); await assert.rejects(validationState(run), IpcError);
  }
});
