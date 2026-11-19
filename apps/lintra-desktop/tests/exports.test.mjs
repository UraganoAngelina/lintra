import assert from "node:assert/strict";
import { afterEach, test } from "node:test";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { reportAbout, exportPrepare, exportExecute, exportState, exportCancel, IpcError } from "../src/ipc.ts";

globalThis.window = { crypto: globalThis.crypto };
afterEach(clearMocks);
const run = { schemaVersion: 1, instance_id: "a".repeat(32), run_id: "18446744073709551615" };
const preview = text => ({ schemaVersion: 1, text, complete: true });
const bundle = { schemaVersion: 1, bundle_source: "embedded", bundle_version: null, bundle_manifest_status: "absent" };
const about = { schemaVersion: 1, report_ref: "p_1", run, app_version: "0.1.1-rc.3", validator_version: preview("0.1.1"),
  flow_id: preview("SYNTHETIC"), schema_version: preview("v1"), schema_hash: preview("b".repeat(64)), mode: "full",
  status: "completed_invalid", completeness: "partial", truncated: false, exportable: true, trace_count: "2",
  reference_dataset_count: "0", bundle };
const local = { schemaVersion: 1, local_files: "2", remote_files: "0", undetermined_files: "0", warning_acknowledged: false };
const ready = { schemaVersion: 1, export_ref: "e_1", report_ref: "p_1", run, format: "csv", state: "ready", destination: local };
const done = { schemaVersion: 1, export_ref: "e_1", report_ref: "p_1", run, format: "csv", state: "succeeded",
  published_roles: ["report", "summary"], cleanup_failed: false, durability_confirmed: true, destination: local, error: null };
const technical = { schemaVersion: 1, category: "technical", code: "IPC_TECHNICAL_FAILURE" };
function mock(response, observe = () => {}) {
  mockIPC((command, bytes) => {
    assert.equal(command, "lintra_ipc");
    const request = JSON.parse(new TextDecoder().decode(bytes)); observe(request);
    return new TextEncoder().encode(JSON.stringify({ schemaVersion: 1, request_id: request.request_id, kind: "result", payload: response })).buffer;
  });
}
function never() { mockIPC(() => assert.fail("invalid request must not reach Rust")); }

test("about returns bounded provenance of the published run", async () => {
  mock(about, r => { assert.equal(r.operation, "report_about"); assert.deepEqual(r.payload, { schemaVersion: 1, run }); });
  assert.deepEqual(await reportAbout(run), about);
  for (const payload of [{ ...about, run: { ...run, run_id: "2" } }, { ...about, exportable: true, truncated: true },
    { ...about, status: "cancelled" }, { ...about, trace_count: "0" }, { ...about, trace_count: 2 },
    { ...about, mode: "trace" }, { ...about, bundle: { ...bundle, bundle_source: "cloud" } },
    { ...about, input_path: "/tmp/S.txt" }, { ...about, app_version: "1 0" }]) {
    mock(payload); await assert.rejects(() => reportAbout(run), IpcError);
  }
});

test("prepare sends only a handle and a format; outcomes are explicit", async () => {
  mock(ready, r => assert.deepEqual(r.payload, { schemaVersion: 1, report_ref: "p_1", format: "csv" }));
  assert.deepEqual(await exportPrepare("p_1", "csv"), ready);
  for (const reason of ["native_selection_cancelled", "native_confirmation_declined"]) {
    mock({ schemaVersion: 1, state: "cancelled", reason });
    assert.equal((await exportPrepare("p_1", "csv")).reason, reason);
  }
  mock({ schemaVersion: 1, state: "rejected", reason: "destination_not_allowed" });
  assert.equal((await exportPrepare("p_1", "csv")).state, "rejected");
  never(); await assert.rejects(() => exportPrepare("p_1", "pdf"), IpcError);
  never(); await assert.rejects(() => exportPrepare("p/1", "csv"), IpcError);
  const shared = { ...local, local_files: "0", remote_files: "1", undetermined_files: "1", warning_acknowledged: true };
  mock({ ...ready, destination: shared });
  assert.equal((await exportPrepare("p_1", "csv")).destination.warning_acknowledged, true);
  for (const payload of [{ ...ready, format: "xlsx" }, { ...ready, report_ref: "p_2" }, { ...ready, path: "/tmp/r.csv" },
    { ...ready, destination: { ...local, local_files: "3" } }, { ...ready, destination: { ...local, warning_acknowledged: true } },
    { ...ready, destination: { ...local, local_files: 2 } }, { schemaVersion: 1, state: "cancelled", reason: "other" },
    { schemaVersion: 1, state: "rejected", reason: "protected_source", path: "/tmp/S.txt" }]) {
    mock(payload); await assert.rejects(() => exportPrepare("p_1", "csv"), IpcError);
  }
});

test("receipts keep partial publication explicit and success strict", async () => {
  mock({ schemaVersion: 1, export_ref: "e_1", state: "writing" }, r => { assert.equal(r.operation, "export_execute"); assert.deepEqual(r.payload, { schemaVersion: 1, export_ref: "e_1" }); });
  assert.equal((await exportExecute("e_1")).state, "writing");
  mock(done, r => assert.equal(r.operation, "export_state"));
  assert.deepEqual(await exportState("e_1"), done);
  const partial = { ...done, state: "failed", published_roles: ["report"], durability_confirmed: false, error: technical };
  mock(partial);
  assert.deepEqual((await exportState("e_1")).published_roles, ["report"]);
  mock({ ...done, state: "cancelled", published_roles: [], durability_confirmed: false });
  assert.equal((await exportState("e_1")).state, "cancelled");
  for (const payload of [{ ...done, published_roles: ["report"] }, { ...done, durability_confirmed: false },
    { ...done, cleanup_failed: true }, { ...done, error: technical }, { ...partial, error: null },
    { ...done, published_roles: ["report", "report"] }, { ...done, published_roles: ["report", "input"] },
    { ...done, export_ref: "e_2" }, { ...done, files: ["/tmp/r.csv"] }]) {
    mock(payload); await assert.rejects(() => exportState("e_1"), IpcError);
  }
  mock(done);
  await assert.rejects(() => exportExecute("e_1"), IpcError);
  mock({ schemaVersion: 1, export_ref: "e_1", disposition: "requested" }, r => assert.equal(r.operation, "export_cancel"));
  assert.equal(await exportCancel("e_1"), "requested");
  mock({ schemaVersion: 1, export_ref: "e_1", disposition: "too_late" });
  await assert.rejects(() => exportCancel("e_1"), IpcError);
});
