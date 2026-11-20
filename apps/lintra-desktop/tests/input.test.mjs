// T-MIG-P5-03: input file chosen through the native dialog owned by Rust.
// Acceptance: the frontend never reads fixed-width bytes and never presents
// the bounded sample as the whole file.
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { afterEach, test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { loadComponent } from "./tsx.mjs";
import { INITIAL_SELECTION, INPUT_FAILED, chooseInput, missingChoice, reduce } from "../src/analysis.ts";
import { planInputChoose } from "../src/ipc.ts";

globalThis.window = { crypto: globalThis.crypto };
afterEach(clearMocks);

const provenance = { schemaVersion: 1, bundle_source: "embedded", bundle_version: "synthetic-v1", bundle_manifest_status: "present" };
const catalog = [{ schemaVersion: 1, schema_ref: "s_v1", flow_id: "SYNTHETIC", schema_version: "v1", trace_count: "2" }];
const configuration = { schemaVersion: 1, configuration_ref: "config_1", flow_id: "SYNTHETIC", schema_version: "v1", schema_hash: "a".repeat(64),
  encoding: "utf8", trace_count: "2", parameter_count: "0", provenance };
const traces = [{ trace_ref: "config_1_trace_0", trace_id: "T1" }, { trace_ref: "config_1_trace_1", trace_id: "T2" }];
const draft = { schemaVersion: 1, plan_ref: "plan_1", configuration_ref: "config_1", mode: "local_only", assigned_input_count: "0" };
const partial = { schemaVersion: 1, file_size: "1048576", sampled_bytes: "65536", lf: "8192", crlf: "0", lone_cr: "0", reached_eof: false };
const whole = { schemaVersion: 1, file_size: "14", sampled_bytes: "14", lf: "1", crlf: "1", lone_cr: "0", reached_eof: true };

function planned() {
  let state = [
    { type: "catalog_loaded", entries: catalog, provenance }, { type: "select_flow", flow: "SYNTHETIC" },
    { type: "select_version", schemaRef: "s_v1" }, { type: "configuration_loaded", token: 1, selection: configuration, traces },
    { type: "select_trace", traceRef: "config_1_trace_0" },
  ].reduce(reduce, INITIAL_SELECTION);
  return reduce(state, { type: "plan_created", token: state.plan.token, draft });
}

test("the file is chosen once per dialog and only for a plan accepted by Rust", () => {
  assert.deepEqual(reduce(INITIAL_SELECTION, { type: "choose_input" }), INITIAL_SELECTION);
  let state = planned();
  assert.equal(missingChoice(state), "Scegli il file del tracciato.");
  state = reduce(state, { type: "choose_input" });
  const token = state.plan.input.activity.token;
  assert.equal(state.plan.input.activity.status, "choosing");
  assert.deepEqual(reduce(state, { type: "choose_input" }), state, "a second dialog while one is open");
  state = reduce(state, { type: "input_chosen", token, planRef: "plan_2", assignedInputCount: "1", preview: partial });
  assert.equal(state.plan.draft.plan_ref, "plan_2");
  assert.equal(state.plan.draft.assigned_input_count, "1");
  assert.deepEqual(state.plan.input, { chosen: partial, activity: { status: "idle" } });
  assert.equal(missingChoice(state), null);
});

test("cancelled dialogs and failed choices keep the previous binding, like Rust", () => {
  let state = reduce(planned(), { type: "choose_input" });
  state = reduce(state, { type: "input_cancelled", token: state.plan.input.activity.token });
  assert.deepEqual(state.plan.input, { chosen: null, activity: { status: "cancelled" } });
  assert.equal(state.plan.draft.plan_ref, "plan_1");
  state = reduce(state, { type: "choose_input" });
  state = reduce(state, { type: "input_chosen", token: state.plan.input.activity.token, planRef: "plan_2", assignedInputCount: "1", preview: whole });
  state = reduce(state, { type: "choose_input" });
  state = reduce(state, { type: "input_failed", token: state.plan.input.activity.token, message: INPUT_FAILED });
  assert.deepEqual(state.plan.input, { chosen: whole, activity: { status: "failed", message: INPUT_FAILED } });
  assert.equal(state.plan.draft.plan_ref, "plan_2");
  state = reduce(state, { type: "choose_input" });
  state = reduce(state, { type: "input_cancelled", token: state.plan.input.activity.token });
  assert.deepEqual(state.plan.input, { chosen: whole, activity: { status: "cancelled" } });
});

test("late dialog replies are ignored and changing the selection drops the input", () => {
  let state = reduce(planned(), { type: "choose_input" });
  const token = state.plan.input.activity.token;
  for (const event of [
    { type: "input_chosen", token: token + 1, planRef: "plan_x", assignedInputCount: "1", preview: whole },
    { type: "input_cancelled", token: token - 1 }, { type: "input_failed", token: 999, message: "x" },
  ]) assert.deepEqual(reduce(state, event), state, JSON.stringify(event));
  state = reduce(state, { type: "input_chosen", token, planRef: "plan_2", assignedInputCount: "1", preview: whole });
  assert.deepEqual(reduce(state, { type: "input_chosen", token, planRef: "plan_3", assignedInputCount: "1", preview: partial }), state);
  // Another trace, version or flow means another plan: the input is gone.
  const other = reduce(state, { type: "select_trace", traceRef: "config_1_trace_1" });
  assert.equal(other.plan.status, "creating");
  assert.equal(reduce(state, { type: "select_version", schemaRef: "s_v1" }).plan.status, "idle");
  assert.equal(reduce(state, { type: "select_flow", flow: null }).plan.status, "idle");
});

test("the controller trusts only Rust's answer for the plan's own trace", async () => {
  const run = async reply => {
    const events = [];
    const calls = [];
    await chooseInput({ planInputChoose: async (...args) => { calls.push(args); if (reply instanceof Error) throw reply; return reply; } },
      event => events.push(event), 4, "plan_1", "config_1_trace_0");
    return { events, calls };
  };
  const base = { plan_ref: "plan_2", trace_ref: "config_1_trace_0", disposition: "selected", assigned_input_count: "1", preview: whole };
  let { events, calls } = await run(base);
  assert.deepEqual(calls, [["plan_1", "config_1_trace_0"]]);
  assert.deepEqual(events, [{ type: "input_chosen", token: 4, planRef: "plan_2", assignedInputCount: "1", preview: whole }]);
  ({ events } = await run({ ...base, plan_ref: "plan_1", disposition: "cancelled", preview: null }));
  assert.deepEqual(events, [{ type: "input_cancelled", token: 4 }]);
  for (const reply of [{ ...base, trace_ref: "config_1_trace_1" }, { ...base, plan_ref: "plan_1" }, { ...base, preview: null }, new Error("IPC_TECHNICAL_FAILURE")]) {
    ({ events } = await run(reply));
    assert.deepEqual(events, [{ type: "input_failed", token: 4, message: INPUT_FAILED }], String(reply?.message ?? JSON.stringify(reply)));
  }
});

test("the IPC request carries only references and a reply exposing a path or name is refused", async () => {
  const reply = { schemaVersion: 1, plan_ref: "plan_2", trace_ref: "config_1_trace_0", disposition: "selected", assigned_input_count: "1", preview: partial };
  let sent;
  const mock = payload => mockIPC((_, bytes) => {
    const request = JSON.parse(new TextDecoder().decode(bytes));
    sent = request;
    return new TextEncoder().encode(JSON.stringify({ schemaVersion: 1, request_id: request.request_id, kind: "result", payload })).buffer;
  });
  mock(reply);
  const events = [];
  await chooseInput({ planInputChoose }, event => events.push(event), 1, "plan_1", "config_1_trace_0");
  assert.equal(sent.operation, "plan_input_choose");
  assert.deepEqual(sent.payload, { schemaVersion: 1, plan_ref: "plan_1", trace_ref: "config_1_trace_0" });
  assert.equal(events[0].type, "input_chosen");
  for (const payload of [{ ...reply, path: "/tmp/x" }, { ...reply, name: "x.txt" }, { ...reply, preview: { ...partial, first_line: "001A" } }]) {
    mock(payload);
    const failed = [];
    await chooseInput({ planInputChoose }, event => failed.push(event), 1, "plan_1", "config_1_trace_0");
    assert.deepEqual(failed, [{ type: "input_failed", token: 1, message: INPUT_FAILED }]);
  }
});

const { AnalysisForm } = await loadComponent("AnalysisForm");
const render = state => renderToStaticMarkup(createElement(AnalysisForm, { state, onEvent: () => {} }));

test("a partial sample is never presented as the whole file", () => {
  let state = reduce(planned(), { type: "choose_input" });
  let html = render(state);
  assert.match(html, /<button type="button" disabled="">Scegli il file<\/button><p class="selection-status" role="status">Finestra di scelta del file aperta…<\/p>/);
  state = reduce(state, { type: "input_chosen", token: state.plan.input.activity.token, planRef: "plan_2", assignedInputCount: "1", preview: partial });
  html = render(state);
  assert.match(html, /<button type="button">Scegli un altro file<\/button>/);
  assert.match(html, /<dl class="selection-summary" aria-label="Anteprima del campione"><dt>Dimensione del file<\/dt><dd>1\.048\.576 byte<\/dd><dt>Campione letto<\/dt><dd>65\.536 byte<\/dd><dt>Fine riga nel campione<\/dt><dd>LF 8192 · CRLF 0 · CR isolati 0<\/dd><\/dl>/);
  assert.match(html, /Il campione comprende solo i primi 65\.536 byte: i conteggi non descrivono l’intero file\./);
  assert.doesNotMatch(html, /intero file\.<\/p>.*intero file/);
  assert.doesNotMatch(html, /record|righe del file|righe totali/i);
  state = reduce(reduce(state, { type: "choose_input" }), { type: "input_chosen", token: state.nextToken, planRef: "plan_3", assignedInputCount: "1", preview: whole });
  html = render(state);
  assert.match(html, /<p class="sample-note">Il campione comprende l’intero file\.<\/p>/);
  assert.doesNotMatch(html, /solo i primi/);
});

test("cancel and failure messages keep the previous file visible", () => {
  let state = reduce(planned(), { type: "choose_input" });
  state = reduce(state, { type: "input_cancelled", token: state.plan.input.activity.token });
  assert.match(render(state), /role="status">Scelta annullata: nessun file assegnato\.<\/p>/);
  state = reduce(state, { type: "choose_input" });
  state = reduce(state, { type: "input_chosen", token: state.plan.input.activity.token, planRef: "plan_2", assignedInputCount: "1", preview: whole });
  state = reduce(state, { type: "choose_input" });
  state = reduce(state, { type: "input_failed", token: state.plan.input.activity.token, message: INPUT_FAILED });
  const html = render(state);
  assert.match(html, /role="alert">Non è stato possibile usare il file scelto/);
  assert.match(html, /aria-label="Anteprima del campione"/);
});

test("no frontend code reads files or bytes", () => {
  const source = new URL("../src/", import.meta.url);
  for (const name of readdirSync(source).filter(file => /\.(ts|tsx)$/.test(file))) {
    const text = readFileSync(new URL(name, source), "utf8");
    assert.doesNotMatch(text, /type="file"|FileReader|\.arrayBuffer\(|new Blob|readFile|readTextFile|plugin-fs|plugin-dialog|showOpenFilePicker/, name);
  }
});
