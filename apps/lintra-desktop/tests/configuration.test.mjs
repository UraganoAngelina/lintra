import assert from "node:assert/strict";
import { afterEach, test } from "node:test";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { catalogPage, configurationSelect, configurationPage, IpcError } from "../src/ipc.ts";

globalThis.window = { crypto: globalThis.crypto };
afterEach(clearMocks);
const provenance = { schemaVersion: 1, bundle_source: "embedded", bundle_version: "synthetic-v1", bundle_manifest_status: "present" };
const row = { schemaVersion: 1, schema_ref: "instance_schema_0", flow_id: "SYNTHETIC", schema_version: "v1", trace_count: "1" };
const catalog = { schemaVersion: 1, offset: "0", entries: [row], next_offset: null, provenance };
const selected = { schemaVersion: 1, configuration_ref: "instance_config_1", flow_id: "SYNTHETIC", schema_version: "v1", schema_hash: "a".repeat(64), encoding: "utf8", trace_count: "1", parameter_count: "0", provenance };
const descriptors = { schemaVersion: 1, configuration_ref: selected.configuration_ref, offset: "0",
  descriptors: [{ schemaVersion: 1, kind: "trace", trace_ref: "instance_config_1_trace_0", trace_id: "T1" }], next_offset: null };
function mock(response, observe = () => {}) {
  mockIPC((command, bytes) => {
    assert.equal(command, "lintra_ipc"); assert.ok(bytes instanceof Uint8Array);
    const request = JSON.parse(new TextDecoder().decode(bytes)); observe(request);
    return new TextEncoder().encode(JSON.stringify({ schemaVersion: 1, request_id: request.request_id, kind: "result", payload: response })).buffer;
  });
}

test("catalog, explicit selection and descriptors use raw versioned DTOs and opaque references", async () => {
  mock(catalog, request => assert.deepEqual(request.payload, { schemaVersion: 1, offset: "0" }));
  assert.deepEqual(await catalogPage(), catalog);
  mock(selected, request => assert.deepEqual(request.payload, { schemaVersion: 1, schema_ref: row.schema_ref, encoding: "utf8" }));
  assert.deepEqual(await configurationSelect(row.schema_ref, "utf8"), selected);
  mock(descriptors, request => assert.deepEqual(request.payload, { schemaVersion: 1, configuration_ref: selected.configuration_ref, offset: "0" }));
  assert.deepEqual(await configurationPage(selected.configuration_ref), descriptors);
});

test("metadata page limits, offsets, nested versions and provenance fail explicitly", async () => {
  for (const payload of [
    { ...catalog, entries: Array(65).fill(row) }, { ...catalog, offset: "01" },
    { ...catalog, next_offset: "64" }, { ...catalog, entries: [{ ...row, extra: true }] },
    { ...catalog, entries: [{ ...row, schemaVersion: 2 }] },
    { ...catalog, provenance: { ...provenance, bundle_manifest_status: "absent" } },
    { ...catalog, provenance: { ...provenance, bundle_version: null } },
    { ...catalog, provenance: { ...provenance, path: "SYNTHETIC" } },
  ]) {
    mock(payload); await assert.rejects(catalogPage, IpcError);
  }
  mock({ ...catalog, entries: Array(64).fill(row), next_offset: "64" });
  assert.equal((await catalogPage()).next_offset, "64");
});

test("selected metadata and descriptor references cannot silently drift or gain fields", async () => {
  for (const payload of [{ ...selected, schema_hash: "unknown" }, { ...selected, encoding: "auto" },
    { ...selected, trace_count: 1 }, { ...selected, path: "SYNTHETIC" }]) {
    mock(payload); await assert.rejects(() => configurationSelect(row.schema_ref, "utf8"), IpcError);
  }
  for (const payload of [{ ...descriptors, configuration_ref: "other" },
    { ...descriptors, descriptors: [{ ...descriptors.descriptors[0], raw: "SYNTHETIC" }] },
    { ...descriptors, descriptors: [{ schemaVersion: 1, kind: "parameter", parameter_ref: "param_1", parameter_id: "namespace", value_kind: "float" }] }]) {
    mock(payload); await assert.rejects(() => configurationPage(selected.configuration_ref), IpcError);
  }
});

test("malformed intents fail before transport and u64 values stay canonical strings", async () => {
  let calls = 0; mock(catalog, () => { calls++; });
  for (const value of ["01", "-1", "18446744073709551616", "1.0"]) await assert.rejects(() => catalogPage(value), IpcError);
  await assert.rejects(() => configurationSelect("../file", "utf8"), IpcError);
  await assert.rejects(() => configurationSelect(row.schema_ref, "auto"), IpcError);
  assert.equal(calls, 0);
  mock({ ...catalog, offset: "18446744073709551615", entries: [], next_offset: null });
  assert.equal((await catalogPage("18446744073709551615")).offset, "18446744073709551615");
});
