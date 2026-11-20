// T-MIG-P5-02: selection state machine. Parity with the Slint contract in
// ui/tests/analysis_configuration.rs (explicit version, dependent choices
// cleared, invalid selections never prepared) and ui/tests/schema_binding.rs
// (the schema shown is the one Rust captured), plus stale replies, paging
// and the rendered form. No normative logic: values come from fake Rust replies.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { afterEach, test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { loadComponent } from "./tsx.mjs";
import {
  CATALOG_FAILED, INITIAL_SELECTION, PLAN_FAILED, SELECTION_FAILED, createPlan, flows, loadCatalog,
  loadConfiguration, missingChoice, reduce, versions,
} from "../src/analysis.ts";
import { catalogPage, configurationPage, configurationSelect, planCreate } from "../src/ipc.ts";

globalThis.window = { crypto: globalThis.crypto };
afterEach(clearMocks);

const provenance = { schemaVersion: 1, bundle_source: "embedded", bundle_version: "synthetic-v1", bundle_manifest_status: "present" };
const entry = (flow, version, ref) => ({ schemaVersion: 1, schema_ref: ref, flow_id: flow, schema_version: version, trace_count: "1" });
const catalog = [entry("SYNTHETIC", "v1", "s_v1"), entry("SYNTHETIC", "v2", "s_v2"), entry("OTHER", "v1", "o_v1")];
const configuration = (flow = "SYNTHETIC", version = "v2", ref = "config_1", traceCount = "1") => ({
  schemaVersion: 1, configuration_ref: ref, flow_id: flow, schema_version: version, schema_hash: "a".repeat(64),
  encoding: "utf8", trace_count: traceCount, parameter_count: "0", provenance,
});
const traces = [{ trace_ref: "config_1_trace_0", trace_id: "T1" }];
const draft = { schemaVersion: 1, plan_ref: "plan_1", configuration_ref: "config_1", mode: "local_only", assigned_input_count: "0" };

const run = (...events) => events.reduce(reduce, INITIAL_SELECTION);
const loaded = () => run({ type: "catalog_loaded", entries: catalog, provenance });

test("catalog requires an explicit version and clears dependent choices (analysis_configuration parity)", () => {
  let state = loaded();
  assert.deepEqual(flows(state), ["SYNTHETIC", "OTHER"]);
  assert.equal(state.flow, null);
  assert.deepEqual(versions(state), []);
  assert.equal(missingChoice(state), "Scegli il flusso da analizzare.");
  state = reduce(state, { type: "select_flow", flow: "SYNTHETIC" });
  assert.equal(state.schemaRef, null);
  assert.equal(state.traceRef, null);
  assert.deepEqual(versions(state).map(item => item.schema_version), ["v1", "v2"]);
  assert.match(missingChoice(state), /nessuna versione viene scelta in automatico/);
  state = reduce(state, { type: "select_version", schemaRef: "s_v1" });
  assert.deepEqual(state.configuration, { status: "loading", token: 1 });
  state = reduce(state, { type: "configuration_loaded", token: 1, selection: configuration("SYNTHETIC", "v1"), traces });
  state = reduce(state, { type: "select_trace", traceRef: "config_1_trace_0" });
  assert.equal(state.traceRef, "config_1_trace_0");
  assert.equal(state.plan.status, "creating");
  // Choosing another version clears the trace and the plan.
  state = reduce(state, { type: "select_version", schemaRef: "s_v2" });
  assert.equal(state.traceRef, null);
  assert.deepEqual(state.plan, { status: "idle" });
  assert.equal(state.configuration.status, "loading");
  // Choosing another flow clears version, configuration, trace and plan.
  state = reduce(state, { type: "select_flow", flow: "OTHER" });
  assert.equal(state.schemaRef, null);
  assert.deepEqual(state.configuration, { status: "idle" });
  assert.equal(state.traceRef, null);
  assert.deepEqual(versions(state).map(item => item.schema_ref), ["o_v1"]);
  // Clearing a choice is allowed and clears its dependants too.
  state = reduce(state, { type: "select_flow", flow: null });
  assert.equal(state.flow, null);
  assert.deepEqual(versions(state), []);
});

test("unknown or out-of-order choices never change the selection (invalid selection parity)", () => {
  const ready = run({ type: "catalog_loaded", entries: catalog, provenance }, { type: "select_flow", flow: "SYNTHETIC" },
    { type: "select_version", schemaRef: "s_v2" }, { type: "configuration_loaded", token: 1, selection: configuration(), traces });
  for (const event of [
    { type: "select_flow", flow: "MISSING" }, { type: "select_flow", flow: "synthetic" },
    { type: "select_version", schemaRef: "o_v1" }, { type: "select_version", schemaRef: "missing" },
    { type: "select_trace", traceRef: "UNKNOWN" }, { type: "select_trace", traceRef: "T1" },
  ]) assert.deepEqual(reduce(ready, event), ready, JSON.stringify(event));
  // A trace cannot be chosen before Rust has returned the configuration.
  const loading = run({ type: "catalog_loaded", entries: catalog, provenance }, { type: "select_flow", flow: "SYNTHETIC" },
    { type: "select_version", schemaRef: "s_v2" });
  assert.deepEqual(reduce(loading, { type: "select_trace", traceRef: "config_1_trace_0" }), loading);
  // Nothing can be chosen before the catalog is loaded.
  assert.deepEqual(reduce(INITIAL_SELECTION, { type: "select_flow", flow: "SYNTHETIC" }), INITIAL_SELECTION);
});

test("late replies for a replaced choice are ignored", () => {
  let state = run({ type: "catalog_loaded", entries: catalog, provenance }, { type: "select_flow", flow: "SYNTHETIC" },
    { type: "select_version", schemaRef: "s_v1" }, { type: "select_version", schemaRef: "s_v2" });
  assert.deepEqual(state.configuration, { status: "loading", token: 2 });
  const stale = reduce(state, { type: "configuration_loaded", token: 1, selection: configuration("SYNTHETIC", "v1"), traces });
  assert.deepEqual(stale, state);
  assert.deepEqual(reduce(state, { type: "configuration_failed", token: 1, message: "x" }), state);
  state = reduce(state, { type: "configuration_loaded", token: 2, selection: configuration(), traces });
  state = reduce(state, { type: "select_trace", traceRef: "config_1_trace_0" });
  const token = state.plan.token;
  state = reduce(state, { type: "select_trace", traceRef: null });
  assert.deepEqual(reduce(state, { type: "plan_created", token, draft }), state);
  state = reduce(state, { type: "select_trace", traceRef: "config_1_trace_0" });
  assert.notEqual(state.plan.token, token);
  state = reduce(state, { type: "plan_created", token: state.plan.token, draft });
  assert.deepEqual(state.plan, { status: "ready", draft, input: { chosen: null, activity: { status: "idle" } } });
  // T-MIG-P5-03: once Rust accepts the plan, the file is the next choice.
  assert.equal(missingChoice(state), "Scegli il file del tracciato.");
  // A reply after the plan is ready, or a failure for an old token, changes nothing.
  assert.deepEqual(reduce(state, { type: "plan_failed", token, message: "x" }), state);
});

function fakeApi(overrides = {}) {
  const calls = [];
  const api = {
    catalogPage: async offset => { calls.push(["catalog", offset]); return { entries: catalog, next_offset: null, provenance }; },
    configurationSelect: async (ref, encoding) => { calls.push(["select", ref, encoding]); return configuration(); },
    configurationPage: async (ref, offset) => { calls.push(["page", ref, offset]); return { descriptors: [{ kind: "trace", ...traces[0] }, { kind: "parameter" }], next_offset: null }; },
    planCreate: async (ref, mode, trace) => { calls.push(["plan", ref, mode, trace]); return draft; },
    ...overrides,
  };
  return { api, calls };
}
const collect = () => { const events = []; return { events, dispatch: event => events.push(event) }; };

test("catalog and trace lists are read page by page and must stay consistent", async () => {
  const many = Array.from({ length: 70 }, (_, i) => entry("F", `v${i}`, `ref_${i}`));
  const { api, calls } = fakeApi({ catalogPage: async offset => {
    calls.push(["catalog", offset]);
    return offset === "0" ? { entries: many.slice(0, 64), next_offset: "64", provenance } : { entries: many.slice(64), next_offset: null, provenance };
  } });
  let sink = collect();
  await loadCatalog(api, sink.dispatch);
  assert.deepEqual(calls, [["catalog", "0"], ["catalog", "64"]]);
  assert.equal(sink.events[0].type, "catalog_loaded");
  assert.equal(sink.events[0].entries.length, 70);
  // A provenance change between pages means a different catalog: refuse it.
  sink = collect();
  let page = 0;
  await loadCatalog(fakeApi({ catalogPage: async () => (page++ === 0
    ? { entries: many.slice(0, 64), next_offset: "64", provenance }
    : { entries: [], next_offset: null, provenance: { ...provenance, bundle_version: "other" } }) }).api, sink.dispatch);
  assert.deepEqual(sink.events, [{ type: "catalog_failed", message: CATALOG_FAILED }]);
  sink = collect();
  await loadCatalog(fakeApi({ catalogPage: async () => { throw new Error("IPC_BUSY"); } }).api, sink.dispatch);
  assert.deepEqual(sink.events, [{ type: "catalog_failed", message: CATALOG_FAILED }]);
});

test("the schema shown is the schema Rust captured (schema_binding parity)", async () => {
  const chosen = catalog[1];
  let { api, calls } = fakeApi();
  let sink = collect();
  await loadConfiguration(api, sink.dispatch, 7, chosen);
  assert.deepEqual(calls, [["select", "s_v2", "utf8"], ["page", "config_1", "0"]]);
  assert.deepEqual(sink.events, [{ type: "configuration_loaded", token: 7, selection: configuration(), traces }]);
  for (const reply of [configuration("SYNTHETIC", "v1"), configuration("OTHER", "v2"), configuration("SYNTHETIC", "v2", "config_1", "2")]) {
    sink = collect();
    await loadConfiguration(fakeApi({ configurationSelect: async () => reply }).api, sink.dispatch, 7, chosen);
    assert.deepEqual(sink.events, [{ type: "configuration_failed", token: 7, message: SELECTION_FAILED }], JSON.stringify(reply));
  }
  sink = collect();
  await loadConfiguration(fakeApi({ configurationPage: async () => { throw new Error("IPC_STALE_CONTEXT"); } }).api, sink.dispatch, 7, chosen);
  assert.deepEqual(sink.events, [{ type: "configuration_failed", token: 7, message: SELECTION_FAILED }]);
});

test("the LocalOnly plan is created by Rust for the captured configuration and trace only", async () => {
  let { api, calls } = fakeApi();
  let sink = collect();
  await createPlan(api, sink.dispatch, 3, "config_1", "config_1_trace_0");
  assert.deepEqual(calls, [["plan", "config_1", "local_only", "config_1_trace_0"]]);
  assert.deepEqual(sink.events, [{ type: "plan_created", token: 3, draft }]);
  for (const reply of [{ ...draft, configuration_ref: "config_2" }, { ...draft, mode: "full" }]) {
    sink = collect();
    await createPlan(fakeApi({ planCreate: async () => reply }).api, sink.dispatch, 3, "config_1", "config_1_trace_0");
    assert.deepEqual(sink.events, [{ type: "plan_failed", token: 3, message: PLAN_FAILED }]);
  }
});

test("the real IPC client wires the controller to the exact v1 operations", async () => {
  const api = { catalogPage, configurationSelect, configurationPage, planCreate };
  const replies = {
    catalog_page: { schemaVersion: 1, offset: "0", entries: [catalog[1]], next_offset: null, provenance },
    configuration_select: configuration(),
    configuration_page: { schemaVersion: 1, configuration_ref: "config_1", offset: "0",
      descriptors: [{ schemaVersion: 1, kind: "trace", trace_ref: "config_1_trace_0", trace_id: "T1" }], next_offset: null },
    plan_create: draft,
  };
  const seen = [];
  mockIPC((_, bytes) => {
    const request = JSON.parse(new TextDecoder().decode(bytes));
    seen.push(request.operation);
    return new TextEncoder().encode(JSON.stringify({ schemaVersion: 1, request_id: request.request_id, kind: "result", payload: replies[request.operation] })).buffer;
  });
  const sink = collect();
  await loadCatalog(api, sink.dispatch);
  await loadConfiguration(api, sink.dispatch, 1, catalog[1]);
  await createPlan(api, sink.dispatch, 2, "config_1", "config_1_trace_0");
  assert.deepEqual(seen, ["catalog_page", "configuration_select", "configuration_page", "plan_create"]);
  assert.deepEqual(sink.events.map(event => event.type), ["catalog_loaded", "configuration_loaded", "plan_created"]);
});

test("the selection modules hold no normative logic and import only the IPC client", () => {
  const model = readFileSync(new URL("../src/analysis.ts", import.meta.url), "utf8");
  assert.deepEqual([...model.matchAll(/^import .* from "(.+)";$/gm)].map(match => match[1]), ["./ipc"]);
  assert.match(model, /^import type /m);
  const form = readFileSync(new URL("../src/AnalysisForm.tsx", import.meta.url), "utf8");
  assert.deepEqual([...form.matchAll(/from "(.+)";$/gm)].map(match => match[1]), ["react", "./ipc", "./analysis"]);
  for (const text of [model, form]) assert.doesNotMatch(text, /RegExp|\.test\(|constraint|allowed_values|readFile|localStorage|fetch\(/);
});

const { AnalysisForm } = await loadComponent("AnalysisForm");
const render = state => renderToStaticMarkup(createElement(AnalysisForm, { state, onEvent: () => {} }));

test("the form labels every choice, preselects nothing and enables a field only after its parent", () => {
  let html = render(INITIAL_SELECTION);
  assert.equal((html.match(/<label for="selection-(flow|version|trace)">/g) ?? []).length, 3);
  assert.match(html, /<select id="selection-flow" disabled="">/);
  assert.match(html, /<p class="selection-status" role="status">Lettura del catalogo degli schemi…<\/p>/);
  html = render(loaded());
  assert.match(html, /<select id="selection-flow"><option value="" selected="">Scegli il flusso<\/option><option value="SYNTHETIC">SYNTHETIC<\/option><option value="OTHER">OTHER<\/option><\/select>/);
  assert.match(html, /<select id="selection-version" disabled="">/);
  assert.match(html, /<select id="selection-trace" disabled="">/);
  html = render(reduce(loaded(), { type: "select_flow", flow: "SYNTHETIC" }));
  assert.match(html, /<select id="selection-version"><option value="" selected="">Scegli la versione<\/option><option value="s_v1">v1<\/option><option value="s_v2">v2<\/option><\/select>/);
  assert.match(html, /nessuna versione viene scelta in automatico/);
  assert.equal((html.match(/<h2\b/g) ?? []).length, 1);
  assert.doesNotMatch(html, /\sstyle=|<img\b|onclick/i);
});

test("the summary shows the schema captured by Rust and failures are announced", () => {
  let state = run({ type: "catalog_loaded", entries: catalog, provenance }, { type: "select_flow", flow: "SYNTHETIC" },
    { type: "select_version", schemaRef: "s_v2" });
  assert.match(render(state), /role="status">Il servizio acquisisce lo schema scelto…/);
  state = reduce(state, { type: "configuration_loaded", token: 1, selection: { ...configuration(), schema_hash: "0123456789abcdef".repeat(4) }, traces });
  let html = render(state);
  assert.match(html, /<dl class="selection-summary" aria-label="Schema acquisito dal servizio"><dt>Schema<\/dt><dd>SYNTHETIC · v2<\/dd><dt>Codifica<\/dt><dd>UTF-8<\/dd><dt>Impronta<\/dt><dd>0123456789abcdef<\/dd><dt>Origine<\/dt><dd>schemi incorporati, versione synthetic-v1<\/dd><\/dl>/);
  assert.match(html, /<select id="selection-trace"><option value="" selected="">Scegli il tracciato<\/option><option value="config_1_trace_0">T1<\/option><\/select>/);
  state = reduce(state, { type: "select_trace", traceRef: "config_1_trace_0" });
  state = reduce(state, { type: "plan_created", token: state.plan.token, draft });
  html = render(state);
  assert.match(html, /role="status">Scegli il file del tracciato\./);
  assert.match(html, /<dt>Tracciato<\/dt><dd>T1<\/dd>/);
  const failed = reduce(reduce(state, { type: "select_version", schemaRef: "s_v1" }), { type: "configuration_failed", token: 3, message: SELECTION_FAILED });
  html = render(failed);
  assert.match(html, /<p class="selection-status" role="alert">Non è stato possibile caricare lo schema scelto\./);
  assert.doesNotMatch(html, /selection-summary/);
  const external = { ...configuration(), provenance: { schemaVersion: 1, bundle_source: "external", bundle_version: null, bundle_manifest_status: "absent" } };
  const ext = reduce(run({ type: "catalog_loaded", entries: catalog, provenance }, { type: "select_flow", flow: "SYNTHETIC" },
    { type: "select_version", schemaRef: "s_v2" }), { type: "configuration_loaded", token: 1, selection: external, traces });
  assert.match(render(ext), /<dd>schemi esterni, versione non dichiarata<\/dd>/);
});
