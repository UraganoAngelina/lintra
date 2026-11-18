import assert from "node:assert/strict";
import { afterEach, test } from "node:test";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { resultsQuery, resultsQueryState, resultsPage, DEFAULT_PAGE_ROWS, MAX_PAGE_ROWS, IpcError } from "../src/ipc.ts";

globalThis.window = { crypto: globalThis.crypto };
afterEach(clearMocks);
const run = { schemaVersion: 1, instance_id: "a".repeat(32), run_id: "1" };
const other = { ...run, run_id: "2" };
const q = "q_" + "0".repeat(32);
const ready = { schemaVersion: 1, run, query_ref: q, state: "ready", total_matches: "18446744073709551615", total_diagnostics: "18446744073709551615" };
const building = { ...ready, state: "building", total_matches: null, total_diagnostics: null };
const preview = text => ({ schemaVersion: 1, text, complete: true });
const row = i => ({ schemaVersion: 1, diagnostic_ref: "d_" + String(i).padStart(32, "0"), ordinal: String(i), line: String(i + 1),
  trace_ref: "trace_0", code: preview("E_SYNTHETIC_RULE"), field: i % 2 ? null : preview("Campo à") });
const page = { schemaVersion: 1, query_ref: q, run, page_generation: "1", offset: "0", rows: [row(0), row(1)],
  next_offset: "2", has_more: true, total_matches: "3", total_diagnostics: "5" };
function mock(response, observe = () => {}) {
  mockIPC((command, bytes) => {
    assert.equal(command, "lintra_ipc");
    const request = JSON.parse(new TextDecoder().decode(bytes)); observe(request);
    return new TextEncoder().encode(JSON.stringify({ schemaVersion: 1, request_id: request.request_id, kind: "result", payload: response })).buffer;
  });
}

test("queries send exact filter DTOs and keep u64 totals as text", async () => {
  const filters = { schemaVersion: 1, code: " E ", line: "007", trace_ref: "trace_1" };
  mock(building, r => { assert.equal(r.operation, "results_query"); assert.deepEqual(r.payload, { schemaVersion: 1, run, filters }); });
  assert.deepEqual(await resultsQuery(run, filters), building);
  mock(ready, r => assert.deepEqual(r.payload, { schemaVersion: 1, query_ref: q }));
  assert.deepEqual(await resultsQueryState(q), ready);
  mock({ ...building, state: "failed" });
  assert.equal((await resultsQueryState(q)).state, "failed");
  for (const bad of [{ schemaVersion: 1, path: "SYNTHETIC" }, { schemaVersion: 1, code: 1 }, { schemaVersion: 1, line: 7 },
    { schemaVersion: 1, line: "-1" }, { schemaVersion: 1, trace_ref: "a b" }, { schemaVersion: 2 }]) {
    mock(building, () => assert.fail("invalid filters must not reach Rust"));
    await assert.rejects(() => resultsQuery(run, bad), IpcError);
  }
});

test("query status rejects stale runs, foreign refs and inconsistent totals", async () => {
  for (const payload of [{ ...ready, run: other }, { ...ready, total_matches: "6", total_diagnostics: "5" },
    { ...ready, total_matches: 3 }, { ...ready, total_matches: null }, { ...building, total_matches: "0" },
    { ...ready, state: "done" }, { ...ready, offsets: ["0"] }, { ...ready, query_ref: "q/1" }]) {
    mock(payload); await assert.rejects(() => resultsQuery(run, { schemaVersion: 1 }), IpcError);
  }
  mock({ ...building, state: "failed" });
  await assert.rejects(() => resultsQuery(run, { schemaVersion: 1 }), IpcError);
  mock({ ...ready, query_ref: "q_other" });
  await assert.rejects(() => resultsQueryState(q), IpcError);
});

test("pages are bounded, ordered and contain only references and previews", async () => {
  assert.equal(DEFAULT_PAGE_ROWS, 64); assert.equal(MAX_PAGE_ROWS, 128);
  mock(page, r => assert.deepEqual(r.payload, { schemaVersion: 1, query_ref: q, offset: "0", limit: 64 }));
  assert.deepEqual(await resultsPage(q), page);
  mock({ ...page, offset: "3", rows: [], next_offset: "3", has_more: false });
  assert.equal((await resultsPage(q, "3", 128)).rows.length, 0);
  for (const [offset, limit] of [["01", 1], ["-1", 1], ["0", 0], ["0", 129], ["0", 1.5]]) {
    mock(page, () => assert.fail("invalid paging must not reach Rust"));
    await assert.rejects(() => resultsPage(q, offset, limit), IpcError);
  }
  const long = "à".repeat(129);
  for (const payload of [{ ...page, rows: [row(1), row(0)] }, { ...page, next_offset: "3" },
    { ...page, has_more: false }, { ...page, offset: "1" }, { ...page, query_ref: "q_other" },
    { ...page, total_matches: "1" }, { ...page, total_diagnostics: "2" },
    { ...page, rows: [{ ...row(0), actual: { raw_hex: "00" } }, row(1)] },
    { ...page, rows: [{ ...row(0), record_identity: {} }, row(1)] },
    { ...page, rows: [{ ...row(0), line: "0" }, row(1)] },
    { ...page, rows: [{ ...row(0), field: { schemaVersion: 1, text: long, complete: false } }, row(1)] },
    { ...page, rows: [{ ...row(0), code: { schemaVersion: 1, text: "E" } }, row(1)] },
    { ...page, index: ["0"] }]) {
    mock(payload); await assert.rejects(() => resultsPage(q, "0", 64), IpcError);
  }
  mock({ ...page, rows: [row(0), row(1), row(2)], next_offset: "3", has_more: false });
  await assert.rejects(() => resultsPage(q, "0", 2), IpcError);
});
