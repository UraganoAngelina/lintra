import assert from "node:assert/strict";
import { afterEach, test } from "node:test";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { resultsSelect, resultsText, resultsContext, resultsDomain, resultsDomainState, MAX_TEXT_BYTES, MAX_DETAIL_ENTRIES, IpcError } from "../src/ipc.ts";

globalThis.window = { crypto: globalThis.crypto };
afterEach(clearMocks);
const run = { schemaVersion: 1, instance_id: "a".repeat(32), run_id: "1" };
const q = "q_" + "0".repeat(32);
const d = "d_" + "1".repeat(32);
const s = "s_" + "2".repeat(32);
const preview = (text, complete = true) => ({ schemaVersion: 1, text, complete });
const part = (kind, text, complete = true) => ({ schemaVersion: 1, component_ref: "x_" + kind, kind, preview: preview(text, complete) });
const source = { schemaVersion: 1, source_ref: "o_1", kind: "reference", conditional: false, rule: null,
  dataset: { schemaVersion: 1, id: preview("union-A"), version: preview("1") }, comparison: preview("Confronto sui byte esatti.") };
const selection = { schemaVersion: 1, selection_ref: s, query_ref: q, diagnostic_ref: d, run, line: "18446744073709551615", trace_ref: "trace_0",
  field_position: { schemaVersion: 1, start: "4", end: "6", units: "one_based_inclusive_bytes" }, actual_visibility: "available",
  components: [part("code", "E_REFERENCE"), part("message", "Valore", false), part("actual", "XX")], domain_sources: [source] };
const digest = "b".repeat(64);
const available = { schemaVersion: 1, kind: "reference", conditional: false, expected_sha256: digest, actual_sha256: digest,
  status: "available", entries: [{ schemaVersion: 1, component_ref: "v_1", preview: preview("\"AA\"") }], next_cursor: "c_1" };
const ready = { schemaVersion: 1, selection_ref: s, source_ref: "o_1", state: "ready", read_ref: null, domain: available };
const loading = { schemaVersion: 1, selection_ref: s, source_ref: "o_1", state: "loading", read_ref: "r_1", domain: null };
function mock(response, observe = () => {}) {
  mockIPC((command, bytes) => {
    assert.equal(command, "lintra_ipc");
    const request = JSON.parse(new TextDecoder().decode(bytes)); observe(request);
    return new TextEncoder().encode(JSON.stringify({ schemaVersion: 1, request_id: request.request_id, kind: "result", payload: response })).buffer;
  });
}
function never() { mockIPC(() => assert.fail("invalid request must not reach Rust")); }

test("selection sends exact refs and keeps u64 coordinates as text", async () => {
  mock(selection, r => { assert.equal(r.operation, "results_select"); assert.deepEqual(r.payload, { schemaVersion: 1, query_ref: q, diagnostic_ref: d }); });
  assert.deepEqual(await resultsSelect(q, d), selection);
  const record = { ...selection, field_position: null, actual_visibility: "not_present", components: [part("code", "E_RECORD_LENGTH"), part("message", "Corto")], domain_sources: [] };
  mock(record);
  assert.deepEqual(await resultsSelect(q, d), record);
  for (const bad of ["d/1", "", 1]) { never(); await assert.rejects(() => resultsSelect(q, bad), IpcError); }
});

test("selection rejects raw values, foreign refs and inconsistent visibility", async () => {
  const sources = (extra) => ({ ...selection, domain_sources: [{ ...source, ...extra }] });
  for (const payload of [{ ...selection, query_ref: "q_other" }, { ...selection, diagnostic_ref: "d_other" },
    { ...selection, raw_hex: "00" }, { ...selection, line: "0" }, { ...selection, line: 7 },
    { ...selection, actual_visibility: "local_report_only_record_scope" },
    { ...selection, components: [part("code", "E"), part("message", "M")] },
    { ...selection, components: [...selection.components, part("record", "SYNTHETIC")] },
    { ...selection, components: [...selection.components, part("code", "E")] },
    { ...selection, components: [part("message", "M"), part("actual", "X")] },
    { ...selection, components: [{ ...part("code", "E"), preview: preview("à".repeat(129), false) }, part("message", "M"), part("actual", "X")] },
    { ...selection, field_position: { schemaVersion: 1, start: "5", end: "4", units: "one_based_inclusive_bytes" } },
    { ...selection, field_position: { schemaVersion: 1, start: "1", end: "4", units: "zero_based" } },
    { ...selection, domain_sources: Array(MAX_DETAIL_ENTRIES + 1).fill(source) },
    sources({ kind: "invented" }), sources({ conditional: true }), sources({ rule: preview("R") }),
    sources({ dataset: null }), sources({ kind: "range" }), sources({ path: "A.json" })]) {
    mock(payload); await assert.rejects(() => resultsSelect(q, d), IpcError);
  }
});

test("text chunks are bounded, contiguous and exact", async () => {
  assert.equal(MAX_TEXT_BYTES, 16384);
  mock({ schemaVersion: 1, offset: "0", text: "😀", next_offset: "4", has_more: true },
    r => assert.deepEqual(r.payload, { schemaVersion: 1, selection_ref: s, component_ref: "x_message", offset: "0", limit: 16384 }));
  assert.equal((await resultsText(s, "x_message")).text, "😀");
  mock({ schemaVersion: 1, offset: "5", text: "", next_offset: "5", has_more: false });
  assert.equal((await resultsText(s, "x_message", "5", 4)).has_more, false);
  for (const [offset, limit] of [["01", 4], ["-1", 4], ["0", 0], ["0", 16385], ["0", 1.5], ["0", "4"]]) {
    never(); await assert.rejects(() => resultsText(s, "x_message", offset, limit), IpcError);
  }
  for (const payload of [{ schemaVersion: 1, offset: "0", text: "😀", next_offset: "3", has_more: true },
    { schemaVersion: 1, offset: "1", text: "x", next_offset: "2", has_more: true },
    { schemaVersion: 1, offset: "0", text: "", next_offset: "0", has_more: true },
    { schemaVersion: 1, offset: "0", text: "xxxxx", next_offset: "5", has_more: false },
    { schemaVersion: 1, offset: "0", text: "x", next_offset: "1", has_more: false, path: "/tmp" }]) {
    mock(payload); await assert.rejects(() => resultsText(s, "x_message", "0", 4), IpcError);
  }
});

test("context pages are bounded and withheld values carry no handle", async () => {
  const shown = { schemaVersion: 1, trace: preview("T1"), field: preview("F2"), component_ref: "k_1", value: preview("01"), visibility: "available" };
  const hidden = { ...shown, component_ref: null, value: null, visibility: "local_report_only_record_scope" };
  const page = { schemaVersion: 1, selection_ref: s, offset: "64", entries: [shown, hidden], next_offset: "66", has_more: true };
  mock(page, r => assert.deepEqual(r.payload, { schemaVersion: 1, selection_ref: s, offset: "64", limit: 64 }));
  assert.deepEqual(await resultsContext(s, "64"), page);
  never(); await assert.rejects(() => resultsContext(s, "0", 65), IpcError);
  for (const payload of [{ ...page, next_offset: "65" }, { ...page, offset: "0" }, { ...page, selection_ref: "s_other" },
    { ...page, entries: [{ ...hidden, value: preview("WHOLE") }] }, { ...page, entries: [{ ...shown, component_ref: null }] },
    { ...page, entries: [{ ...shown, raw_hex: "00" }] }, { ...page, entries: [], next_offset: "64", has_more: true }]) {
    mock(payload); await assert.rejects(() => resultsContext(s, "64"), IpcError);
  }
  mock({ ...page, entries: [shown, hidden, shown] });
  await assert.rejects(() => resultsContext(s, "64", 2), IpcError);
});

test("domains distinguish loading, verified pages and every non-listing status", async () => {
  mock(loading, r => { assert.equal(r.operation, "results_domain"); assert.deepEqual(r.payload, { schemaVersion: 1, selection_ref: s, source_ref: "o_1", cursor_ref: null }); });
  assert.deepEqual(await resultsDomain(s, "o_1"), loading);
  mock(ready, r => assert.deepEqual(r.payload, { schemaVersion: 1, selection_ref: s, read_ref: "r_1" }));
  assert.deepEqual(await resultsDomainState(s, "r_1"), ready);
  mock({ ...ready, domain: { ...available, status: "available_empty", entries: [], next_cursor: null } });
  assert.deepEqual((await resultsDomain(s, "o_1", "c_1")).domain.entries, []);
  for (const status of ["degraded", "unavailable", "not_captured", "unsupported", "cancelled", "not_enumerable"]) {
    mock({ ...ready, domain: { ...available, status, entries: null, next_cursor: null } });
    assert.equal((await resultsDomain(s, "o_1")).domain.entries, null);
  }
  for (const domain of [{ ...available, status: "degraded" }, { ...available, entries: null },
    { ...available, status: "available_empty" }, { ...available, entries: [] },
    { ...available, status: "not_captured", entries: null, next_cursor: "c_1" },
    { ...available, expected_sha256: "B".repeat(64) }, { ...available, actual_sha256: "b" },
    { ...available, entries: Array(MAX_DETAIL_ENTRIES + 1).fill(available.entries[0]) },
    { ...available, status: "empty" }, { ...available, values: ["AA"] }]) {
    mock({ ...ready, domain }); await assert.rejects(() => resultsDomain(s, "o_1"), IpcError);
  }
  for (const payload of [{ ...loading, domain: available }, { ...ready, read_ref: "r_1" }, { ...ready, source_ref: "o_2" },
    { ...ready, selection_ref: "s_other" }, { ...ready, state: "done" }]) {
    mock(payload); await assert.rejects(() => resultsDomain(s, "o_1"), IpcError);
  }
  mock({ ...loading, read_ref: "r_2" });
  await assert.rejects(() => resultsDomainState(s, "r_1"), IpcError);
  never(); await assert.rejects(() => resultsDomain(s, "o_1", "c/1"), IpcError);
});
