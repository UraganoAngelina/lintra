// T-MIG-P5-06: operator cancel, terminal state and the previous report.
// Acceptance: no false conformity and no new report after cancel; state
// machine tests. Rust decides: a request proves nothing, the terminal does.
import assert from "node:assert/strict";
import { test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { loadComponent } from "./tsx.mjs";
import { INITIAL_SELECTION } from "../src/analysis.ts";
import { CANCEL_FAILED, CANCEL_TOO_LATE, INITIAL_RUN, activeText, busy, canCancel, canStart, cancelRun, outcomeText, reduceRun } from "../src/run.ts";

globalThis.window = { crypto: globalThis.crypto };

const reports = { xlsx: true, jsonl: true, csv: false };
const first = { schemaVersion: 1, instance_id: "a".repeat(32), run_id: "1" };
const second = { ...first, run_id: "2" };
const snapshot = (run, state, extra = {}) => ({ schemaVersion: 1, state, run, plan_ref: "plan_1", mode: "local_only", cancel_requested: false, outcome: null, ...extra });
const valid = { schemaVersion: 1, kind: "validation", status: "completed_valid", completeness: "complete", truncated: false, report_available: true };
const cancelled = { schemaVersion: 1, kind: "validation", status: "cancelled", completeness: "partial", truncated: false, report_available: false };
const notStarted = { schemaVersion: 1, kind: "not_started", reason: "cancelled" };
const technical = { schemaVersion: 1, kind: "technical_failure", error: { schemaVersion: 1, category: "technical", code: "IPC_TECHNICAL_FAILURE" } };

function run(state, ref) {
  const started = reduceRun(state, { type: "start", reports });
  return reduceRun(started, { type: "started", token: started.view.token, run: ref });
}
function finish(state, ref, outcome) {
  return reduceRun(state, { type: "terminal", run: ref, outcome, acknowledged: true });
}
function running(state, ref) {
  return reduceRun(state, { type: "snapshot", snapshot: snapshot(ref, "running") });
}

test("cancel is offered only while Rust can stop the work, once", () => {
  let state = run(INITIAL_RUN, first);
  assert.ok(canCancel(state.view), "preparing");
  state = running(state, first);
  assert.ok(canCancel(state.view), "running");
  for (const stage of ["finalizing", "cancelling"]) {
    assert.ok(!canCancel(reduceRun(state, { type: "snapshot", snapshot: snapshot(first, stage) }).view), stage);
  }
  assert.ok(!canCancel(reduceRun(state, { type: "snapshot", snapshot: snapshot(first, "running", { cancel_requested: true }) }).view));
  const warned = { schemaVersion: 1, run: first, state: "confirmation_required", confirmation_ref: "h",
    audit: { schemaVersion: 1, input_bytes: "2", reference_bytes: "0", missing_reference_files: "0", input_budget: "1", reference_budget: null,
      input_budget_exceeded: true, reference_budget_exceeded: false } };
  assert.ok(!canCancel(reduceRun(state, { type: "gate", gate: warned }).view), "the D09 dialog's Annulla is the cancel there");
  state = reduceRun(state, { type: "cancel" });
  assert.equal(state.view.cancel, "requesting");
  assert.equal(reduceRun(state, { type: "cancel" }), state, "no second request");
});

test("each Rust disposition is shown as such; a request is not a cancelled outcome", async () => {
  for (const [disposition, expected] of [["requested", "requested"], ["already_requested", "requested"], ["too_late", "too_late"], ["already_terminal", "requested"]]) {
    let state = running(run(INITIAL_RUN, first), first);
    const calls = [];
    await cancelRun({ validationCancel: async ref => { calls.push(ref); return disposition; } }, e => { state = reduceRun(state, e); }, first);
    assert.deepEqual(calls, [first]);
    assert.equal(state.view.status, "active", `${disposition}: still waiting for the Rust terminal`);
    assert.equal(state.view.cancel, expected);
    assert.ok(busy(state), "the form stays frozen until the terminal");
  }
  let late = running(run(INITIAL_RUN, first), first);
  await cancelRun({ validationCancel: async () => "too_late" }, e => { late = reduceRun(late, e); }, first);
  assert.ok(!canCancel(late.view));
  assert.equal(CANCEL_TOO_LATE, "L'analisi è già in conclusione: non è più possibile interromperla. Attendi l'esito.");
  // A failed request can be retried; nothing else changes.
  let failed = running(run(INITIAL_RUN, first), first);
  await cancelRun({ validationCancel: async () => { throw new Error("lost"); } }, e => { failed = reduceRun(failed, e); }, first);
  assert.equal(failed.view.cancel, "failed");
  assert.ok(canCancel(failed.view), "retry allowed");
  // Answers about another run are ignored.
  const other = reduceRun(failed, { type: "cancel_answer", run: second, disposition: "requested" });
  assert.equal(other, failed);
});

test("after a requested cancel the words wait for Rust and the outcome decides", () => {
  let state = reduceRun(running(run(INITIAL_RUN, first), first), { type: "cancel" });
  state = reduceRun(state, { type: "cancel_answer", run: first, disposition: "requested" });
  assert.equal(activeText(state.view), "Interruzione richiesta: attendi la conclusione comunicata dal servizio.");
  // Rust may still complete the run before the boundary: its outcome wins.
  const completed = finish(state, first, valid);
  assert.equal(completed.view.outcome.status, "completed_valid");
  assert.deepEqual(completed.report, first, "a published report is the run's own");
  const stopped = finish(state, first, cancelled);
  assert.equal(stopped.report, null, "no report after cancel");
  assert.equal(outcomeText(stopped.view.outcome).title, "Analisi annullata");
  assert.match(outcomeText(stopped.view.outcome).detail, /Nessun report completo pubblicato per questa analisi\./);
  assert.ok(!busy(stopped) && canStart(stopped));
});

test("a cancelled or failed newer run never replaces nor impersonates the previous report", () => {
  let state = finish(run(INITIAL_RUN, first), first, valid);
  assert.deepEqual(state.report, first);
  for (const outcome of [cancelled, notStarted, technical]) {
    const next = finish(run(state, second), second, outcome);
    assert.deepEqual(next.report, first, `${outcome.kind}: the previous report keeps its identity`);
    const { title, detail } = outcomeText(next.view.outcome, true);
    assert.doesNotMatch(title, /Nessuna violazione|Violazioni/, "never a conformity for the cancelled run");
    assert.match(detail, /Il report dell'analisi precedente resta in Risultati: riguarda quell'analisi, non questa\./);
    assert.doesNotMatch(detail, /Il report è disponibile in Risultati\./);
  }
  // A newer published report becomes the current one.
  const replaced = finish(run(state, second), second, { ...valid, status: "completed_invalid" });
  assert.deepEqual(replaced.report, second);
});

test("truncated and partial outcomes are never presented as conformity", () => {
  for (const status of ["completed_valid", "completed_invalid"]) {
    const text = outcomeText({ ...valid, status, completeness: "partial", truncated: true, report_available: false });
    assert.equal(text.title, "Analisi parziale");
    assert.match(text.detail, /Raggiunto il limite operativo\. Nessun report completo pubblicato/);
  }
  assert.equal(outcomeText({ ...valid, completeness: "partial" }).title, "Controlli incompleti");
});

const { RunPanel } = await loadComponent("RunPanel");
const selection = { ...INITIAL_SELECTION, plan: { status: "ready", draft: { schemaVersion: 1, plan_ref: "plan_1", configuration_ref: "c", mode: "local_only", assigned_input_count: "1" },
  input: { chosen: { schemaVersion: 1, file_size: "9", sampled_bytes: "9", lf: "2", crlf: "0", lone_cr: "0", reached_eof: true }, activity: { status: "idle" } } } };
const panel = state => renderToStaticMarkup(createElement(RunPanel, { state, selection, onEvent: () => {}, onStart: () => {}, onDecide: () => {}, onCancel: () => {} }));

test("the panel shows Interrompi analisi while it applies and keeps it in place afterwards", () => {
  const budget = { type: "budget_loaded", budget: { schemaVersion: 1, state: "unconfigured", input_bytes: null, reference_bytes: null } };
  let state = running(run(reduceRun(INITIAL_RUN, budget), first), first);
  assert.match(panel(state), /<button type="button" class="secondary" aria-disabled="false">Interrompi analisi<\/button>/);
  state = reduceRun(reduceRun(state, { type: "cancel" }), { type: "cancel_answer", run: first, disposition: "requested" });
  assert.match(panel(state), /<button type="button" class="secondary" aria-disabled="true">Interruzione richiesta<\/button>/);
  const late = reduceRun(reduceRun(running(run(reduceRun(INITIAL_RUN, budget), first), first), { type: "cancel" }),
    { type: "cancel_answer", run: first, disposition: "too_late" });
  assert.match(panel(late), /non è più possibile interromperla/);
  const failed = reduceRun(reduceRun(running(run(reduceRun(INITIAL_RUN, budget), first), first), { type: "cancel" }), { type: "cancel_failed", run: first });
  assert.match(panel(failed), new RegExp(`role="alert">${CANCEL_FAILED.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}`));
  const done = finish(state, first, cancelled);
  assert.doesNotMatch(panel(done), /Interrompi analisi|Interruzione richiesta/, "no cancel once terminal");
  assert.match(panel(done), /<h3>Analisi annullata<\/h3>/);
});
