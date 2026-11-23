// T-MIG-P5-05: run screen, throttled progress and the D09 soft-budget
// dialog. Acceptance: responsive UI with bounded memory and messages
// coherent with Rust; no frontend budget check, no fabricated warning.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { afterEach, test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { loadComponent } from "./tsx.mjs";
import { IpcError, resourceBudgetState, validationBudget, validationBudgetDecide } from "../src/ipc.ts";
import { INITIAL_SELECTION, reduce } from "../src/analysis.ts";
import {
  BUDGET_INVALID, INITIAL_RUN, POLL_MS, START_FAILED, activeText, budgetText, busy, decide, gateOpen, outcomeText, pollRun,
  reduceRun, startRun,
} from "../src/run.ts";

globalThis.window = { crypto: globalThis.crypto };
afterEach(clearMocks);

const run = { schemaVersion: 1, instance_id: "a".repeat(32), run_id: "1" };
const other = { ...run, run_id: "2" };
const audit = { schemaVersion: 1, input_bytes: "140000", reference_bytes: "0", missing_reference_files: "0",
  input_budget: "65536", reference_budget: null, input_budget_exceeded: true, reference_budget_exceeded: false };
const warned = { schemaVersion: 1, run, state: "confirmation_required", audit, confirmation_ref: "0123456789abcdef0123456789abcdef" };
const reports = { xlsx: true, jsonl: true, csv: false };
const progress = (sequence, records = "10") => ({ schemaVersion: 1, run, sequence, phase: "local_validation", trace_ref: "trace_0", trace_records: records, cross_records: "0" });
const snapshot = (state, outcome = null) => ({ schemaVersion: 1, state, run, plan_ref: "plan_1", mode: "local_only", cancel_requested: false, outcome });
const valid = { schemaVersion: 1, kind: "validation", status: "completed_valid", completeness: "complete", truncated: false, report_available: true };

function mock(response, observe = () => {}) {
  mockIPC((command, bytes) => {
    const request = JSON.parse(new TextDecoder().decode(bytes)); observe(request);
    return new TextEncoder().encode(JSON.stringify({ schemaVersion: 1, request_id: request.request_id, kind: "result", payload: response })).buffer;
  });
}

test("budget DTOs are exact and flags must agree with the counters Rust sent", async () => {
  mock({ schemaVersion: 1, state: "unconfigured", input_bytes: null, reference_bytes: null }, r => assert.deepEqual(r.payload, { schemaVersion: 1 }));
  assert.equal((await resourceBudgetState()).state, "unconfigured");
  mock({ schemaVersion: 1, state: "configured", input_bytes: "18446744073709551615", reference_bytes: null });
  assert.equal((await resourceBudgetState()).input_bytes, "18446744073709551615");
  for (const bad of [{ schemaVersion: 1, state: "invalid", input_bytes: "1", reference_bytes: null },
    { schemaVersion: 1, state: "configured", input_bytes: "0", reference_bytes: null },
    { schemaVersion: 1, state: "configured", input_bytes: 10, reference_bytes: null },
    { schemaVersion: 1, state: "configured", input_bytes: null, reference_bytes: null, path: "/home/x" },
    { schemaVersion: 1, state: "skip", input_bytes: null, reference_bytes: null }]) {
    mock(bad); await assert.rejects(() => resourceBudgetState(), IpcError);
  }
  mock(warned, r => assert.deepEqual(r.payload, { schemaVersion: 1, run }));
  assert.deepEqual(await validationBudget(run), warned);
  for (const bad of [{ ...warned, run: other }, { ...warned, confirmation_ref: null },
    { ...warned, audit: { ...audit, input_budget_exceeded: false } },          // warning without a crossed threshold
    { ...warned, audit: { ...audit, input_bytes: "100" } },                    // flag disagrees with counters
    { ...warned, state: "within_budget", confirmation_ref: null },             // within while exceeded
    { ...warned, state: "proceeded" },                                         // handle outside the pending state
    { ...warned, state: "checking", confirmation_ref: null },                  // audit before the preflight ends
    { ...warned, audit: { ...audit, measurement_source: "x" } }]) {
    mock(bad); await assert.rejects(() => validationBudget(run), IpcError);
  }
  mock({ schemaVersion: 1, run, disposition: "proceeding" }, r => assert.deepEqual(r.payload,
    { schemaVersion: 1, run, confirmation_ref: warned.confirmation_ref, decision: "proceed" }));
  assert.equal(await validationBudgetDecide(run, warned.confirmation_ref, "proceed"), "proceeding");
  mock({ schemaVersion: 1, run, disposition: "proceeding" });
  await assert.rejects(() => validationBudgetDecide(run, warned.confirmation_ref, "cancel"), IpcError);
  await assert.rejects(() => validationBudgetDecide(run, "a/b", "proceed"), IpcError);
});

/** Scripted API: each call returns the next reply of its queue. */
function api(script, calls = []) {
  const take = name => (...args) => {
    calls.push([name, ...args]);
    const queue = script[name];
    if (!queue || queue.length === 0) return Promise.reject(new Error(`unexpected ${name}`));
    const reply = queue.shift();
    return reply instanceof Error ? Promise.reject(reply) : Promise.resolve(reply);
  };
  return Object.fromEntries(["resourceBudgetState", "planPrepare", "validationStart", "validationState", "validationProgress",
    "validationBudget", "validationBudgetDecide", "validationAcknowledge"].map(name => [name, take(name)]));
}
const unconfigured = { schemaVersion: 1, state: "unconfigured", input_bytes: null, reference_bytes: null };

test("start freezes the plan in Rust, then starts it; an invalid budget never starts", async () => {
  let state = reduceRun(INITIAL_RUN, { type: "start", reports });
  assert.ok(busy(state));
  assert.equal(reduceRun(state, { type: "start", reports }), state, "one start at a time");
  const events = [];
  const calls = [];
  await startRun(api({ resourceBudgetState: [unconfigured], planPrepare: [{ plan_ref: "plan_1", mode: "local_only" }], validationStart: [run] }, calls),
    e => events.push(e), state.view.token, "plan_1");
  assert.deepEqual(calls.map(c => c[0]), ["resourceBudgetState", "planPrepare", "validationStart"]);
  for (const event of events) state = reduceRun(state, event);
  assert.equal(state.view.status, "active");
  assert.deepEqual(state.view.reports, reports, "formats frozen with the run");

  let invalid = reduceRun(INITIAL_RUN, { type: "start", reports });
  const later = [];
  const invalidCalls = [];
  await startRun(api({ resourceBudgetState: [{ ...unconfigured, state: "invalid" }] }, invalidCalls), e => later.push(e), invalid.view.token, "plan_1");
  assert.deepEqual(invalidCalls.map(c => c[0]), ["resourceBudgetState"], "no prepare or start with an invalid budget");
  for (const event of later) invalid = reduceRun(invalid, event);
  assert.deepEqual(invalid.view, { status: "failed", message: BUDGET_INVALID });

  let refused = reduceRun(INITIAL_RUN, { type: "start", reports });
  const failed = [];
  await startRun(api({ resourceBudgetState: [unconfigured, unconfigured], planPrepare: [{ plan_ref: "plan_1", mode: "local_only" }],
    validationStart: [new IpcError("IPC_BUSY")] }), e => failed.push(e), refused.view.token, "plan_1");
  await new Promise(resolve => setTimeout(resolve, 0));
  for (const event of failed) refused = reduceRun(refused, event);
  assert.deepEqual(refused.view, { status: "failed", message: START_FAILED });
  assert.ok(!busy(refused));
});

function active() {
  const started = reduceRun(INITIAL_RUN, { type: "start", reports });
  return reduceRun(started, { type: "started", token: started.view.token, run });
}

test("polling reads state first, keeps one sample, ignores late and foreign replies", async () => {
  let state = active();
  const events = [];
  const again = await pollRun(api({ validationState: [snapshot("running")], validationProgress: [progress("5")], validationBudget: [{ ...warned, state: "unconfigured", audit: null, confirmation_ref: null }] }),
    e => events.push(e), run, true);
  assert.equal(again, true);
  for (const event of events) state = reduceRun(state, event);
  assert.equal(state.view.progress.sequence, "5");
  assert.equal(gateOpen(state.view.gate), false, "an unconfigured gate is not read again");
  // Older or repeated sequence and another run's data change nothing.
  for (const event of [{ type: "progress", progress: progress("4", "999") }, { type: "progress", progress: progress("5", "999") },
    { type: "progress", progress: { ...progress("9"), run: other } }, { type: "gate", gate: { ...warned, run: other } },
    { type: "snapshot", snapshot: { ...snapshot("running"), run: other } }]) {
    assert.equal(reduceRun(state, event), state, event.type);
  }
  // The terminal is acknowledged and final: late samples cannot reopen it.
  const end = [];
  const calls = [];
  assert.equal(await pollRun(api({ validationState: [snapshot("terminal", valid)], validationProgress: [progress("6", "4000000")],
    validationAcknowledge: [undefined] }, calls), e => end.push(e), run, false), false);
  assert.deepEqual(calls.map(c => c[0]), ["validationState", "validationProgress", "validationAcknowledge"]);
  for (const event of end) state = reduceRun(state, event);
  assert.equal(state.view.status, "terminal");
  assert.equal(state.view.progress.trace_records, "4000000", "the terminal shows the final count, not an older sample");
  // A failing progress channel never hides the terminal.
  let sealed = active();
  await pollRun(api({ validationState: [snapshot("terminal", valid)], validationProgress: [new IpcError("IPC_TECHNICAL_FAILURE")],
    validationAcknowledge: [undefined] }), e => { sealed = reduceRun(sealed, e); }, run, false);
  assert.equal(sealed.view.status, "terminal");
  assert.equal(reduceRun(state, { type: "progress", progress: progress("99") }), state);
  assert.ok(!busy(state));
  assert.ok(POLL_MS >= 100, "throttled: never a request per record or per frame");
});

test("Annulla and Procedi comunque send the exact handle once; a refused decision changes nothing", async () => {
  let state = reduceRun(active(), { type: "gate", gate: warned });
  assert.equal(activeText(state.view), "I file superano la soglia di avviso configurata: scegli come procedere.");
  const events = [];
  const calls = [];
  await decide(api({ validationBudgetDecide: ["proceeding"], validationBudget: [{ ...warned, state: "proceeded", confirmation_ref: null }] }, calls),
    e => { events.push(e); state = reduceRun(state, e); }, warned, "proceed");
  assert.deepEqual(calls[0], ["validationBudgetDecide", run, warned.confirmation_ref, "proceed"]);
  assert.equal(state.view.gate.state, "proceeded");
  assert.equal(state.view.deciding, false);
  // A second click while deciding is ignored by the reducer.
  let pending = reduceRun(reduceRun(active(), { type: "gate", gate: warned }), { type: "decide" });
  assert.equal(reduceRun(pending, { type: "decide" }), pending);
  // Stale handle: an explicit message, the state stays the one Rust reports.
  await decide(api({ validationBudgetDecide: [new IpcError("IPC_STALE_CONTEXT")] }), e => { pending = reduceRun(pending, e); }, warned, "cancel");
  assert.equal(pending.view.decisionFailed, true);
  assert.equal(pending.view.gate.state, "confirmation_required");
});

test("words follow the Rust outcome without merging partial, cancelled or technical", () => {
  assert.equal(outcomeText(valid).title, "Nessuna violazione nei controlli eseguiti");
  assert.equal(outcomeText({ ...valid, completeness: "partial" }).title, "Controlli incompleti");
  assert.equal(outcomeText({ ...valid, status: "completed_invalid" }).title, "Violazioni rilevate");
  assert.match(outcomeText({ ...valid, status: "completed_invalid", completeness: "partial" }).detail, /incompleti/);
  assert.match(outcomeText({ ...valid, status: "cancelled", completeness: "partial", report_available: false }).detail, /Nessun report/);
  assert.equal(outcomeText({ schemaVersion: 1, kind: "not_started", reason: "cancelled" }).title, "Analisi annullata");
  assert.equal(outcomeText({ schemaVersion: 1, kind: "technical_failure", error: {} }).title, "Errore tecnico");
  assert.match(budgetText(unconfigured), /non configurato/);
  assert.equal(budgetText({ ...unconfigured, state: "invalid" }), BUDGET_INVALID);
  assert.equal(budgetText({ schemaVersion: 1, state: "configured", input_bytes: "1048576", reference_bytes: null }),
    "Budget di risorse configurato: avviso oltre 1.048.576 byte di input.");
});

const { RunPanel, BudgetDialog, BudgetChoice } = await loadComponent("RunPanel");
const { AnalysisForm } = await loadComponent("AnalysisForm");
const selection = { ...INITIAL_SELECTION, plan: { status: "ready", draft: { schemaVersion: 1, plan_ref: "plan_2", configuration_ref: "c", mode: "local_only", assigned_input_count: "1" },
  input: { chosen: { schemaVersion: 1, file_size: "9", sampled_bytes: "9", lf: "2", crlf: "0", lone_cr: "0", reached_eof: true }, activity: { status: "idle" } } } };
const panel = (state, sel = selection) => renderToStaticMarkup(createElement(RunPanel, { state, selection: sel, onEvent: () => {}, onStart: () => {}, onDecide: () => {} }));

test("the panel shows the budget status and enables Avvia analisi only when Rust allows it", () => {
  const ready = reduceRun(INITIAL_RUN, { type: "budget_loaded", budget: unconfigured });
  assert.match(panel(ready), /<button type="button" class="primary" aria-disabled="false">Avvia analisi<\/button>/);
  assert.match(panel(ready), /Budget di risorse non configurato/);
  assert.match(panel(INITIAL_RUN), /aria-disabled="true">Avvia analisi/, "budget state not yet read");
  const invalid = reduceRun(INITIAL_RUN, { type: "budget_loaded", budget: { ...unconfigured, state: "invalid" } });
  assert.match(panel(invalid), /role="alert">Il file di configurazione dei budget di risorse non è valido/);
  assert.match(panel(invalid), /aria-disabled="true">Avvia analisi/);
  assert.match(panel(ready, INITIAL_SELECTION), /aria-disabled="true">Avvia analisi/, "no file, no start");
  const running = reduceRun(reduceRun(ready, { type: "start", reports }), { type: "started", token: 1, run });
  const html = panel(reduceRun(running, { type: "progress", progress: progress("3", "1234567") }));
  assert.match(html, /aria-disabled="true">Avvia analisi/);
  assert.match(html, /<progress aria-label="Avanzamento dell&#x27;analisi"><\/progress>/, "indeterminate, no invented percentage");
  assert.doesNotMatch(html, /%|ETA|stimat/);
  assert.match(html, /<dt>Record del tracciato<\/dt><dd>1\.234\.567<\/dd>/);
  assert.match(html, /<dt>Formati del report<\/dt><dd>XLSX, JSONL<\/dd>/);
  assert.match(html, /<p>Validazione dei record<\/p>/);
});

test("the D09 dialog shows only Rust measures, with Annulla first and Procedi comunque", () => {
  const html = renderToStaticMarkup(createElement(BudgetDialog, { gate: warned, deciding: false, onDecide: () => {} }));
  assert.match(html, /<dialog class="budget-dialog" aria-labelledby="budget-title" aria-describedby="budget-text">/);
  assert.match(html, /I file superano la soglia di avviso configurata\. L’analisi non ridurrà l’input per rientrare nella soglia\./);
  assert.match(html, /<dt>Input<\/dt><dd>140\.000 byte \(soglia 65\.536 byte\), superata<\/dd>/);
  assert.doesNotMatch(html, /Reference/, "no row for an unconfigured threshold");
  assert.match(html, /<button type="button" autofocus="" aria-disabled="false">Annulla<\/button><button type="button" class="primary" aria-disabled="false">Procedi comunque<\/button>/);
  const missing = renderToStaticMarkup(createElement(BudgetDialog, { gate: { ...warned, audit: { ...audit, missing_reference_files: "2", reference_budget: "10" } }, deciding: true, onDecide: () => {} }));
  assert.match(missing, /<dt>Reference mancanti<\/dt><dd>2 file \(non sono dataset vuoti\)<\/dd>/);
  assert.match(missing, /aria-disabled="true">Annulla/);
  // Panel opens the dialog only for a pending decision.
  const state = reduceRun(active(), { type: "gate", gate: warned });
  assert.match(panel(state), /<dialog class="budget-dialog"/);
  assert.doesNotMatch(panel(reduceRun(state, { type: "gate", gate: { ...warned, state: "declined", confirmation_ref: null } })), /<dialog/);
});

test("the dialog buttons dispatch Annulla and Procedi comunque, never while deciding", () => {
  const chosen = [];
  const buttons = node => {
    if (Array.isArray(node)) return node.flatMap(buttons);
    if (!node || typeof node !== "object") return [];
    return [...(node.type === "button" ? [node.props] : []), ...buttons(node.props?.children)];
  };
  const [annulla, procedi] = buttons(BudgetChoice({ gate: warned, deciding: false, onDecide: d => chosen.push(d) }));
  assert.equal(annulla.autoFocus, true, "the safe choice has the initial focus");
  annulla.onClick(); procedi.onClick();
  assert.deepEqual(chosen, ["cancel", "proceed"]);
  buttons(BudgetChoice({ gate: warned, deciding: true, onDecide: d => chosen.push(d) })).forEach(button => button.onClick());
  assert.deepEqual(chosen, ["cancel", "proceed"], "no decision while one is in flight");
});

test("the form is frozen while a run is active, as Slint busy", () => {
  const state = reduce(selection, { type: "set_report", format: "csv", requested: true });
  const html = renderToStaticMarkup(createElement(AnalysisForm, { state, onEvent: () => {}, busy: true }));
  assert.match(html, /<fieldset class="report-formats" aria-describedby="report-formats-note" disabled="">/);
  assert.match(html, /<button type="button" aria-disabled="true">Scegli un altro file<\/button>/);
  assert.match(html, /<select id="selection-flow" disabled="">/);
});

test("no budget threshold or check lives in the frontend", () => {
  for (const file of ["src/run.ts", "src/RunPanel.tsx"]) {
    const text = readFileSync(new URL(`../${file}`, import.meta.url), "utf8");
    assert.doesNotMatch(text, /input_bytes\s*[<>]=?|file_size\s*[<>]|>\s*BigInt\(.*budget/, file);
  }
});
