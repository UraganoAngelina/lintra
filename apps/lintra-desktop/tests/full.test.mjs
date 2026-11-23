// T-MIG-P5-09: Full selection with an explicit input per trace.
// Acceptance: the UI sends only legitimate typed requests, blocks incoherent
// input, has no implicit latest and makes no normative inference: which traces
// are required and which values are allowed is answered by Rust (fake replies).
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { afterEach, test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { loadComponent } from "./tsx.mjs";
import {
  INITIAL_SELECTION, PARAMETER_FAILED, PARAMETER_NOT_INTEGER, integerSyntax, missingChoice, mutating, reduce, setParameter, startablePlan, versions,
} from "../src/analysis.ts";
import { INITIAL_RUN, START_FAILED_FULL, reduceRun, startRun } from "../src/run.ts";
import { IpcError, integerText, planParameterSet } from "../src/ipc.ts";

globalThis.window = { crypto: globalThis.crypto };
afterEach(clearMocks);

const provenance = { schemaVersion: 1, bundle_source: "external", bundle_version: null, bundle_manifest_status: "absent" };
const catalog = [{ schemaVersion: 1, schema_ref: "f_v1", flow_id: "FULLSYN", schema_version: "v1", trace_count: "2" },
  { schemaVersion: 1, schema_ref: "f_v2", flow_id: "FULLSYN", schema_version: "v2", trace_count: "2" }];
const configuration = { schemaVersion: 1, configuration_ref: "config_1", flow_id: "FULLSYN", schema_version: "v1", schema_hash: "b".repeat(64),
  encoding: "utf8", trace_count: "2", parameter_count: "2", provenance };
const traces = [{ trace_ref: "config_1_trace_0", trace_id: "S" }, { trace_ref: "config_1_trace_1", trace_id: "T" }];
const parameters = [{ parameter_ref: "config_1_parameter_0", parameter_id: "unit", value_kind: "text" },
  { parameter_ref: "config_1_parameter_1", parameter_id: "year", value_kind: "integer" }];
const draft = { schemaVersion: 1, plan_ref: "plan_1", configuration_ref: "config_1", mode: "full", assigned_input_count: "0" };
const preview = { schemaVersion: 1, file_size: "7", sampled_bytes: "7", lf: "1", crlf: "0", lone_cr: "0", reached_eof: true };
const [S, T] = traces.map(trace => trace.trace_ref);
const [UNIT, YEAR] = parameters.map(parameter => parameter.parameter_ref);

const run = (state, ...events) => events.reduce(reduce, state);
/** Flow and version chosen, schema captured by Rust, in the given mode. */
function captured(mode = "full") {
  let state = run(INITIAL_SELECTION, { type: "catalog_loaded", entries: catalog, provenance }, { type: "select_flow", flow: "FULLSYN" });
  if (mode === "full") state = reduce(state, { type: "select_mode", mode: "full" });
  state = reduce(state, { type: "select_version", schemaRef: "f_v1" });
  return reduce(state, { type: "configuration_loaded", token: state.configuration.token, selection: configuration, traces, parameters });
}
function planned() {
  const state = captured();
  return reduce(state, { type: "plan_created", token: state.plan.token, draft });
}
function chosen(state, trace, planRef) {
  state = reduce(state, { type: "choose_input", traceRef: trace });
  const token = state.plan.inputs[trace].activity.token;
  const count = String(Object.values(state.plan.inputs).filter(input => input.chosen !== null).length + 1);
  return reduce(state, { type: "input_chosen", token, planRef, assignedInputCount: count, preview });
}

test("the mode is an explicit choice, never a default version, and changing it drops the plan", () => {
  let state = run(INITIAL_SELECTION, { type: "catalog_loaded", entries: catalog, provenance }, { type: "select_flow", flow: "FULLSYN" });
  assert.equal(state.mode, "local_only", "the existing LocalOnly form is unchanged by default");
  state = reduce(state, { type: "select_mode", mode: "full" });
  assert.equal(state.mode, "full");
  assert.equal(state.schemaRef, null, "Full does not pick a version either");
  assert.deepEqual(versions(state).map(entry => entry.schema_version), ["v1", "v2"]);
  assert.equal(state.plan.status, "idle", "no plan before the schema is captured");
  assert.deepEqual(reduce(state, { type: "select_mode", mode: "latest" }), state, "unknown modes are ignored");
  assert.deepEqual(reduce(state, { type: "select_mode", mode: "full" }), state, "same mode: nothing changes");
  // Captured schema in Full: the plan is requested at once, without a trace.
  state = captured();
  assert.equal(state.plan.status, "creating");
  assert.equal(state.traceRef, null);
  assert.deepEqual(reduce(state, { type: "select_trace", traceRef: S }), state, "Full has no single trace");
  // LocalOnly keeps the trace step; switching mode drops trace, plan and files.
  let local = captured("local_only");
  assert.equal(local.plan.status, "idle");
  local = reduce(local, { type: "select_trace", traceRef: S });
  local = reduce(local, { type: "plan_created", token: local.plan.token, draft: { ...draft, mode: "local_only" } });
  const switched = reduce(local, { type: "select_mode", mode: "full" });
  assert.equal(switched.traceRef, null);
  assert.equal(switched.plan.status, "creating");
  assert.notEqual(switched.plan.token, local.plan.token);
  const back = reduce(planned(), { type: "select_mode", mode: "local_only" });
  assert.equal(back.plan.status, "idle", "LocalOnly waits for its trace");
  assert.equal(back.configuration.status, "ready", "the captured schema is kept");
});

test("a Full plan lists every trace of the captured schema with its own input, none marked required", () => {
  let state = planned();
  assert.deepEqual(Object.keys(state.plan.inputs), [S, T]);
  assert.deepEqual(Object.keys(state.plan.parameters), [UNIT, YEAR]);
  assert.deepEqual(state.plan.parameters[UNIT], { text: "", applied: null, activity: { status: "idle" } });
  assert.equal(missingChoice(state), "Scegli i file dei tracciati del flusso.");
  assert.equal(startablePlan(state), null, "no file, no start");
  state = chosen(state, T, "plan_2");
  assert.equal(state.plan.draft.plan_ref, "plan_2");
  assert.equal(state.plan.draft.assigned_input_count, "1");
  assert.equal(state.plan.inputs[S].chosen, null);
  assert.deepEqual(state.plan.inputs[T], { chosen: preview, activity: { status: "idle" } });
  // One file is enough for React: whether S is required is Rust's answer at prepare.
  assert.equal(missingChoice(state), null);
  assert.equal(startablePlan(state).plan_ref, "plan_2");
});

test("one native dialog at a time across traces, only for listed traces, and late replies are ignored", () => {
  let state = reduce(planned(), { type: "choose_input", traceRef: S });
  assert.ok(mutating(state.plan));
  assert.equal(startablePlan(state), null, "no start while a dialog is open");
  assert.deepEqual(reduce(state, { type: "choose_input", traceRef: T }), state, "a second dialog for another trace");
  assert.deepEqual(reduce(planned(), { type: "choose_input", traceRef: "config_9_trace_0" }), planned(), "unknown trace");
  assert.deepEqual(reduce(planned(), { type: "choose_input" }), planned(), "Full needs the trace named");
  const token = state.plan.inputs[S].activity.token;
  assert.deepEqual(reduce(state, { type: "input_chosen", token: token + 1, planRef: "x", assignedInputCount: "1", preview }), state);
  state = reduce(state, { type: "input_cancelled", token });
  assert.deepEqual(state.plan.inputs[S], { chosen: null, activity: { status: "cancelled" } });
  assert.equal(state.plan.draft.plan_ref, "plan_1", "a cancelled dialog keeps the plan");
});

test("parameter values: typed, sent once, integer syntax blocked locally, Rust's refusal keeps the held value", () => {
  let state = planned();
  // An untouched empty field is not a value: nothing is sent.
  assert.deepEqual(reduce(state, { type: "commit_parameter", parameterRef: UNIT }), state);
  // Integer text that is not digits is refused here and never sent.
  for (const text of ["", "12a", "+5", "1.0", "1e3", " 7", "-"]) {
    const edited = reduce(state, { type: "edit_parameter", parameterRef: YEAR, text });
    const blocked = reduce(edited, { type: "commit_parameter", parameterRef: YEAR });
    if (text === "") { assert.deepEqual(blocked, edited, "empty untouched integer"); continue; }
    assert.deepEqual(blocked.plan.parameters[YEAR].activity, { status: "failed", message: PARAMETER_NOT_INTEGER }, text);
    assert.equal(blocked.nextToken, edited.nextToken, `no request for ${JSON.stringify(text)}`);
    assert.equal(startablePlan(chosen(blocked, S, "plan_x")), null, "an unapplied value blocks the start");
  }
  state = reduce(state, { type: "edit_parameter", parameterRef: YEAR, text: "-0042" });
  state = reduce(state, { type: "commit_parameter", parameterRef: YEAR });
  const sent = state.plan.parameters[YEAR].activity;
  assert.equal(sent.status, "setting");
  assert.equal(sent.text, "-0042", "exact text, never a JS Number");
  assert.equal(startablePlan(state), null);
  // While Rust answers, the field is frozen and no other mutation starts.
  assert.deepEqual(reduce(state, { type: "edit_parameter", parameterRef: YEAR, text: "1" }), state);
  assert.deepEqual(reduce(state, { type: "choose_input", traceRef: S }), state);
  const other = reduce(reduce(state, { type: "edit_parameter", parameterRef: UNIT, text: "A" }), { type: "commit_parameter", parameterRef: UNIT });
  assert.equal(other.plan.parameters[UNIT].activity.status, "idle", "queued as typed text, not sent concurrently");
  assert.deepEqual(reduce(state, { type: "parameter_set", token: sent.token + 1, planRef: "late" }), state);
  state = reduce(state, { type: "parameter_set", token: sent.token, planRef: "plan_2" });
  assert.deepEqual(state.plan.parameters[YEAR], { text: "-0042", applied: "-0042", activity: { status: "idle" } });
  assert.equal(state.plan.draft.plan_ref, "plan_2");
  // A value Rust refuses: the failure is shown, the previous value stays held.
  state = reduce(reduce(state, { type: "edit_parameter", parameterRef: YEAR, text: "99999" }), { type: "commit_parameter", parameterRef: YEAR });
  const refused = state.plan.parameters[YEAR].activity.token;
  state = reduce(state, { type: "parameter_failed", token: refused, message: PARAMETER_FAILED });
  assert.deepEqual(state.plan.parameters[YEAR], { text: "99999", applied: "-0042", activity: { status: "failed", message: PARAMETER_FAILED } });
  assert.equal(state.plan.draft.plan_ref, "plan_2");
  state = chosen(state, S, "plan_3");
  assert.equal(startablePlan(state), null, "the typed value differs from the one Rust holds");
  assert.match(missingChoice(state), /non sono applicati/);
  state = reduce(state, { type: "edit_parameter", parameterRef: YEAR, text: "-0042" });
  assert.equal(startablePlan(state).plan_ref, "plan_3", "back to the held value");
});

test("integer syntax in the model and in the IPC client agree", () => {
  for (const text of ["0", "-0", "007", "-12", "18446744073709551616", "", "-", "+1", "1.5", "1e3", " 1", "1 ", "١", "0x1", "--1"]) {
    assert.equal(integerSyntax(text), integerText(text), JSON.stringify(text));
  }
});

test("setParameter sends the declared kind and trusts only a new Full plan", async () => {
  const calls = [];
  const events = [];
  const reply = async (...args) => { calls.push(args); return { ...draft, plan_ref: "plan_2" }; };
  await setParameter({ planParameterSet: reply }, event => events.push(event), 4, "plan_1", parameters[1], "2026");
  assert.deepEqual(calls, [["plan_1", YEAR, { kind: "integer", text: "2026" }]]);
  assert.deepEqual(events, [{ type: "parameter_set", token: 4, planRef: "plan_2" }]);
  for (const bad of [{ ...draft }, { ...draft, plan_ref: "plan_2", mode: "local_only" }, new IpcError("IPC_INVALID_STATE")]) {
    const failed = [];
    await setParameter({ planParameterSet: async () => { if (bad instanceof Error) throw bad; return bad; } }, event => failed.push(event), 4, "plan_1", parameters[0], "A");
    assert.deepEqual(failed, [{ type: "parameter_failed", token: 4, message: PARAMETER_FAILED }], JSON.stringify(bad));
  }
});

test("the IPC request is exact and an incoherent integer never leaves the renderer", async () => {
  let sent = [];
  mockIPC((_, bytes) => {
    const request = JSON.parse(new TextDecoder().decode(bytes));
    sent.push(request);
    return new TextEncoder().encode(JSON.stringify({ schemaVersion: 1, request_id: request.request_id, kind: "result", payload: { ...draft, plan_ref: "plan_2" } })).buffer;
  });
  await planParameterSet("plan_1", YEAR, { kind: "integer", text: "-0042" });
  assert.equal(sent[0].operation, "plan_parameter_set");
  assert.deepEqual(sent[0].payload, { schemaVersion: 1, plan_ref: "plan_1", parameter_ref: YEAR, value: { schemaVersion: 1, kind: "integer", text: "-0042" } });
  sent = [];
  for (const value of [{ kind: "integer", text: "4.2" }, { kind: "integer", text: "" }, { kind: "number", text: "1" }, { kind: "integer", text: 1 }]) {
    await assert.rejects(planParameterSet("plan_1", YEAR, value), /IPC_INVALID_MESSAGE/);
  }
  assert.deepEqual(sent, [], "nothing sent");
});

const prepared = mode => ({ plan_ref: "plan_3", mode });
const fakeRunApi = (replies, calls) => Object.fromEntries(Object.entries(replies).map(([name, values]) => [name, async (...args) => {
  calls.push([name, ...args]);
  const value = values.shift();
  if (value instanceof Error) throw value;
  return value;
}]));
const unconfigured = { schemaVersion: 1, state: "unconfigured", input_bytes: null, reference_bytes: null };
const runRef = { schemaVersion: 1, instance_id: "i", run_id: "r1" };

test("a Full plan is prepared and started as Full, never downgraded", async () => {
  let state = reduceRun(INITIAL_RUN, { type: "start", reports: {} });
  let calls = [];
  const events = [];
  await startRun(fakeRunApi({ resourceBudgetState: [unconfigured], planPrepare: [prepared("full")], validationStart: [runRef] }, calls),
    event => events.push(event), state.view.token, "plan_3", null, "full");
  assert.deepEqual(calls, [["resourceBudgetState"], ["planPrepare", "plan_3"], ["validationStart", "plan_3", "full"]]);
  assert.equal(events.at(-1).type, "started");
  // Rust refuses the plan (a required trace or parameter missing): Full words, no start.
  for (const reply of [new IpcError("IPC_INVALID_STATE"), prepared("local_only")]) {
    calls = [];
    const failed = [];
    state = reduceRun(INITIAL_RUN, { type: "start", reports: {} });
    await startRun(fakeRunApi({ resourceBudgetState: [unconfigured, unconfigured], planPrepare: [reply],
      validationState: [{ schemaVersion: 1, state: "idle", run: null, plan_ref: null, mode: null, cancel_requested: false, outcome: null }] }, calls),
      event => failed.push(event), state.view.token, "plan_3", null, "full");
    await new Promise(resolve => setTimeout(resolve, 0));
    assert.ok(!calls.some(call => call[0] === "validationStart"), "never started");
    for (const event of failed) state = reduceRun(state, event);
    assert.deepEqual(state.view, { status: "failed", message: START_FAILED_FULL });
  }
  // A lost reply: attach only to a Full run of this plan.
  for (const [mode, expected] of [["full", "active"], ["local_only", "failed"]]) {
    state = reduceRun(INITIAL_RUN, { type: "start", reports: {} });
    const resync = [];
    await startRun(fakeRunApi({ resourceBudgetState: [unconfigured, unconfigured], planPrepare: [prepared("full")], validationStart: [new IpcError("IPC_BUSY")],
      validationState: [{ schemaVersion: 1, state: "running", run: runRef, plan_ref: "plan_3", mode, cancel_requested: false, outcome: null }] }, []),
      event => resync.push(event), state.view.token, "plan_3", null, "full");
    await new Promise(resolve => setTimeout(resolve, 0));
    for (const event of resync) state = reduceRun(state, event);
    assert.equal(state.view.status, expected, mode);
  }
});

const { AnalysisForm } = await loadComponent("AnalysisForm");
const { RunPanel } = await loadComponent("RunPanel");
const render = (state, busy = false) => renderToStaticMarkup(createElement(AnalysisForm, { state, onEvent: () => {}, busy }));

test("the Full form: explicit mode, one labelled file choice per trace, typed parameter fields", () => {
  let html = render(captured("local_only"));
  assert.match(html, /<fieldset class="analysis-mode"><legend>Tipo di analisi<\/legend><label><input type="radio" name="analysis-mode" checked="" value="local_only"\/>Singolo tracciato<\/label><label><input type="radio" name="analysis-mode" value="full"\/>Flusso completo \(Full\)<\/label><\/fieldset>/);
  assert.match(html, /<select id="selection-trace">/);
  let state = chosen(planned(), T, "plan_2");
  html = render(state);
  assert.match(html, /checked="" value="full"/);
  assert.doesNotMatch(html, /selection-trace/, "no single trace in Full");
  assert.match(html, /<h3 id="full-inputs-title">File dei tracciati<\/h3>/);
  assert.match(html, /<h3>Tracciato S<\/h3><button type="button" aria-disabled="false" aria-label="Scegli il file per il tracciato S">Scegli il file<\/button><p class="selection-status" role="status">Nessun file assegnato\.<\/p>/);
  assert.match(html, /<h3>Tracciato T<\/h3><button type="button" aria-disabled="false" aria-label="Scegli un altro file per il tracciato T">Scegli un altro file<\/button><p class="selection-status" role="status">File assegnato al tracciato\.<\/p>/);
  assert.doesNotMatch(html, /obbligatori|richiesto\b|facoltativ|opzional/i, "no required/optional inferred in React");
  assert.match(html, /<label for="parameter-config_1_parameter_0">unit \(testo\)<\/label><input id="parameter-config_1_parameter_0" type="text" spellCheck="false" autoComplete="off" inputMode="text" aria-invalid="false" aria-describedby="parameter-config_1_parameter_0-status" value=""\/><p id="parameter-config_1_parameter_0-status" class="selection-status" role="status">Nessun valore applicato\.<\/p>/);
  assert.match(html, /<label for="parameter-config_1_parameter_1">year \(numero intero\)<\/label><input [^>]*inputMode="numeric"/);
  assert.match(html, /<dt>Analisi<\/dt><dd>Flusso completo \(Full\)<\/dd><dt>File assegnati<\/dt><dd>1 di 2<\/dd>/);
  state = reduce(reduce(state, { type: "edit_parameter", parameterRef: YEAR, text: "20x6" }), { type: "commit_parameter", parameterRef: YEAR });
  html = render(state);
  assert.match(html, /aria-invalid="true" aria-describedby="parameter-config_1_parameter_1-status" value="20x6"\/><p id="parameter-config_1_parameter_1-status" class="selection-status" role="alert">Inserisci un numero intero, solo cifre con un eventuale segno meno\. Per il servizio nessun valore applicato\.<\/p>/);
  state = reduce(state, { type: "edit_parameter", parameterRef: YEAR, text: "2026" });
  assert.match(render(state), /role="status">Valore non ancora applicato: premi Invio o esci dal campo\. Per il servizio nessun valore applicato\./);
  // While a run is active everything is frozen, mode included.
  html = render(state, true);
  assert.match(html, /<fieldset class="analysis-mode" disabled="">/);
  assert.match(html, /<fieldset class="full-parameters" disabled="">/);
  assert.match(html, /aria-label="Scegli il file per il tracciato S" aria-disabled="true"|aria-disabled="true" aria-label="Scegli il file per il tracciato S"/);
});

test("the form dispatches typed events: mode radios, per-trace dialogs, edit then commit on blur or Invio", () => {
  const events = [];
  const nodes = [];
  const walk = node => {
    if (Array.isArray(node)) return node.forEach(walk);
    if (!node || typeof node !== "object") return;
    if (typeof node.type === "function") return walk(node.type(node.props));
    if (node.type === "input" || node.type === "button") nodes.push(node);
    walk(node.props?.children);
  };
  walk(AnalysisForm({ state: planned(), onEvent: event => events.push(event) }));
  const radios = nodes.filter(node => node.props.type === "radio");
  radios[0].props.onChange();
  radios[1].props.onChange();
  const buttons = nodes.filter(node => node.type === "button" && node.props["aria-label"]);
  buttons[1].props.onClick();
  const year = nodes.find(node => node.props.id === "parameter-config_1_parameter_1");
  year.props.onChange({ target: { value: "2026" } });
  year.props.onKeyDown({ key: "Tab" });
  year.props.onKeyDown({ key: "Enter" });
  year.props.onBlur();
  assert.deepEqual(events, [
    { type: "select_mode", mode: "local_only" }, { type: "select_mode", mode: "full" },
    { type: "choose_input", traceRef: T },
    { type: "edit_parameter", parameterRef: YEAR, text: "2026" },
    { type: "commit_parameter", parameterRef: YEAR }, { type: "commit_parameter", parameterRef: YEAR },
  ]);
});

test("Avvia analisi is offered for the Full plan only once React has nothing pending", () => {
  const panel = selection => renderToStaticMarkup(createElement(RunPanel, { state: INITIAL_RUN, selection, onEvent: () => {}, onStart: () => {}, onDecide: () => {}, onCancel: () => {} }));
  assert.match(panel(planned()), /aria-disabled="true">Avvia analisi/, "no file");
  assert.match(panel(planned()), /Completa la selezione, scegli i file dei tracciati e applica i parametri per avviare l&#x27;analisi\./);
  const ready = chosen(planned(), S, "plan_2");
  assert.match(panel(ready), /aria-disabled="false">Avvia analisi/);
  // What the click sends: the plan Rust returned last, in its own mode.
  assert.deepEqual({ planRef: startablePlan(ready).plan_ref, mode: startablePlan(ready).mode }, { planRef: "plan_2", mode: "full" });
  assert.match(readFileSync(new URL("../src/RunPanel.tsx", import.meta.url), "utf8"), /onStart\(\{ planRef: plan\.plan_ref, mode: plan\.mode \}\)/);
  assert.match(panel(reduce(ready, { type: "choose_input", traceRef: T })), /aria-disabled="true">Avvia analisi/, "dialog open");
  const pending = reduce(reduce(ready, { type: "edit_parameter", parameterRef: UNIT, text: "A" }), { type: "commit_parameter", parameterRef: UNIT });
  assert.match(panel(pending), /aria-disabled="true">Avvia analisi/, "value being checked by Rust");
});

test("Full adds no normative logic, storage or file access to the renderer", () => {
  for (const name of ["analysis.ts", "AnalysisForm.tsx", "RunPanel.tsx", "run.ts"]) {
    const text = readFileSync(new URL(`../src/${name}`, import.meta.url), "utf8");
    assert.doesNotMatch(text, /allowed_values|min:|max:|required_traces|availability|localStorage|sessionStorage|readFile|fetch\(|Number\(|parseInt|parseFloat/, name);
  }
});
