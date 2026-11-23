// T-MIG-P5-10: Full monitor and multi-pass terminal states.
// Acceptance: semantic comparison with the Full service; no false conformity
// and no invented percentage. Samples follow PROGRESS.md: trace_records is
// per named trace (handle of the plan), PASS2 has trace_ref null and its own
// cross_records; the terminal outcome is Rust's, never recomputed.
import assert from "node:assert/strict";
import { afterEach, test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { loadComponent } from "./tsx.mjs";
import { INITIAL_SELECTION, reduce, runTraces } from "../src/analysis.ts";
import { INITIAL_RUN, counterRows, lastPhaseText, outcomeText, phaseText, reduceRun, sampleTrace } from "../src/run.ts";
import { validationProgress } from "../src/ipc.ts";

globalThis.window = { crypto: globalThis.crypto };
afterEach(clearMocks);

const instance = "a".repeat(32);
const run = { schemaVersion: 1, instance_id: instance, run_id: "4" };
const S = `${instance}_config_1_trace_0`, T = `${instance}_config_1_trace_2`;
// Bundle of S and T only: the schema trace in between (index 1) had no file.
const context = { mode: "full", traces: [{ trace_ref: S, trace_id: "S" }, { trace_ref: T, trace_id: "T" }] };
const sample = (sequence, phase, trace_ref, trace_records, cross_records = "0") =>
  ({ schemaVersion: 1, run, sequence, phase, trace_ref, trace_records, cross_records });
const reports = { xlsx: true, jsonl: true, csv: false };

test("Full samples are labelled as the service counts them: per named trace, then across traces", () => {
  assert.equal(phaseText(sample("1", "local_validation", S, "0"), context), "Controlli sui singoli tracciati");
  assert.equal(phaseText(sample("9", "cross_record_validation", null, "0", "5"), context), "Controlli tra tracciati");
  assert.equal(phaseText(sample("1", "local_validation", S, "0")), "Validazione dei record", "LocalOnly words unchanged");
  assert.equal(sampleTrace(sample("2", "local_validation", T, "10"), context), "T");
  assert.deepEqual(counterRows(sample("2", "local_validation", T, "1234567"), context), [
    { label: "Tracciati nell'analisi", value: "S, T" }, { label: "Record del tracciato T", value: "1.234.567" }]);
  // PASS2: no trace, so its zero trace_records is not shown as a count.
  assert.deepEqual(counterRows(sample("9", "cross_record_validation", null, "0", "0"), context), [
    { label: "Tracciati nell'analisi", value: "S, T" }, { label: "Record nei controlli tra tracciati", value: "0" }]);
  assert.deepEqual(counterRows(sample("12", "finalizing_report", null, "0", "18446744073709551615"), context).at(-1),
    { label: "Record nei controlli tra tracciati", value: "18.446.744.073.709.551.615" }, "exact u64");
  // A handle outside the run's bundle is never given a name.
  for (const ref of [`${instance}_config_1_trace_1`, "trace_0", `${instance}_config_2_trace_0`]) {
    assert.equal(sampleTrace(sample("3", "local_validation", ref, "7"), context), null, ref);
    assert.equal(counterRows(sample("3", "local_validation", ref, "7"), context)[1].label, "Record del tracciato in corso");
  }
  assert.equal(sampleTrace(sample("3", "local_validation", S, "7"), null), null);
  // LocalOnly rows are the P5-05 ones.
  assert.deepEqual(counterRows(sample("2", "local_validation", S, "3")), [{ label: "Record del tracciato", value: "3" }]);
  assert.deepEqual(counterRows(null), [{ label: "Record del tracciato", value: "—" }]);
});

test("the run context is frozen at Avvia and kept through active and terminal", () => {
  let state = reduceRun(INITIAL_RUN, { type: "start", reports, context });
  assert.deepEqual(state.view.context, context);
  state = reduceRun(state, { type: "started", token: state.view.token, run });
  assert.deepEqual(state.view.context, context);
  state = reduceRun(state, { type: "progress", progress: sample("5", "cross_record_validation", null, "0", "2") });
  const outcome = { schemaVersion: 1, kind: "validation", status: "completed_valid", completeness: "complete", truncated: false, report_available: true };
  state = reduceRun(state, { type: "terminal", run, outcome, acknowledged: true });
  assert.deepEqual(state.view.context, context);
  assert.equal(reduceRun(INITIAL_RUN, { type: "start", reports }).view.context, null, "no context: no names");
});

test("runTraces lists the bundle in schema order: Full only traces with a file", () => {
  const traces = [{ trace_ref: S, trace_id: "S" }, { trace_ref: `${instance}_config_1_trace_1`, trace_id: "U" }, { trace_ref: T, trace_id: "T" }];
  const chosen = { chosen: { schemaVersion: 1 }, activity: { status: "idle" } };
  const empty = { chosen: null, activity: { status: "idle" } };
  const state = { ...INITIAL_SELECTION, mode: "full", configuration: { status: "ready", selection: {}, traces, parameters: [] },
    plan: { status: "ready", draft: {}, inputs: { [T]: chosen, [S]: chosen, [traces[1].trace_ref]: empty }, parameters: {} } };
  assert.deepEqual(runTraces(state), [traces[0], traces[2]]);
  assert.deepEqual(runTraces(INITIAL_SELECTION), []);
  assert.equal(reduce(INITIAL_SELECTION, { type: "select_mode", mode: "full" }).plan.status, "idle");
});

const validation = (status, completeness, truncated, report_available) =>
  ({ schemaVersion: 1, kind: "validation", status, completeness, truncated, report_available });

test("every Full terminal says what Rust said: conformity is never claimed, cancel and partial never merged into success", () => {
  for (const status of ["completed_valid", "completed_invalid", "cancelled"]) {
    for (const completeness of ["complete", "partial"]) {
      for (const truncated of [false, true]) {
        for (const report of [false, true]) {
          const { title, detail } = outcomeText(validation(status, completeness, truncated, report));
          const text = `${title} ${detail}`;
          const label = `${status}/${completeness}/${truncated}/${report}`;
          assert.doesNotMatch(text, /conform|valid[oa]\b|%|percentual|tutti i controlli superati/i, label);
          const clean = status === "completed_valid" && completeness === "complete" && !truncated;
          assert.equal(title === "Nessuna violazione nei controlli eseguiti", clean, label);
          if (status === "cancelled") assert.equal(title, "Analisi annullata", label);
          else if (truncated) assert.equal(title, "Analisi parziale", label);
          else if (status === "completed_invalid") assert.equal(title, "Violazioni rilevate", label);
          if (completeness === "partial" && status === "completed_valid" && !truncated) assert.equal(title, "Controlli incompleti", label);
          assert.equal(/Il report è disponibile/.test(detail), report, label);
        }
      }
    }
  }
});

test("a Full run stopped without a report names the last phase Rust reported, as an observation", () => {
  const pass2 = sample("8", "cross_record_validation", null, "0", "3");
  const cancelled = validation("cancelled", "partial", false, false);
  assert.equal(lastPhaseText(cancelled, pass2, context), "Ultima fase comunicata dal servizio: Controlli tra tracciati.");
  assert.equal(lastPhaseText(cancelled, sample("3", "local_validation", T, "9"), context),
    "Ultima fase comunicata dal servizio: Controlli sui singoli tracciati, tracciato T.");
  const failure = { schemaVersion: 1, kind: "technical_failure", error: {} };
  assert.match(lastPhaseText(failure, pass2, context), /Controlli tra tracciati/);
  // Native finding: cancelled in PASS1, Rust reports finalizing_report and
  // keeps S in the sample. The report writing is not named after S.
  const finalizing = sample("9", "finalizing_report", S, "690554");
  assert.equal(lastPhaseText(cancelled, finalizing, context), "Ultima fase comunicata dal servizio: Scrittura del report.");
  assert.deepEqual(counterRows(finalizing, context)[1], { label: "Record del tracciato S", value: "690.554" }, "S's last real count stays");
  assert.equal(lastPhaseText(validation("completed_valid", "complete", false, true), pass2, context), null, "a published report speaks for itself");
  assert.equal(lastPhaseText(cancelled, null, context), null, "no sample, no claim");
  assert.equal(lastPhaseText(cancelled, pass2, null), null, "LocalOnly unchanged");
  assert.equal(lastPhaseText(cancelled, pass2, { ...context, mode: "local_only" }), null);
});

test("the progress decoder accepts plan handles and still refuses bulk or malformed ones", async () => {
  const reply = payload => mockIPC((_, bytes) => {
    const request = JSON.parse(new TextDecoder().decode(bytes));
    return new TextEncoder().encode(JSON.stringify({ schemaVersion: 1, request_id: request.request_id, kind: "result", payload })).buffer;
  });
  reply(sample("2", "local_validation", T, "10"));
  assert.equal((await validationProgress(run)).trace_ref, T);
  for (const bad of [{ ...sample("2", "local_validation", "a/b", "1") }, { ...sample("2", "local_validation", "x".repeat(129), "1") },
    { ...sample("2", "local_validation", T, "1"), trace_id: "S" }]) {
    reply(bad);
    await assert.rejects(validationProgress(run), /IPC_INVALID_MESSAGE/);
  }
});

const { RunPanel } = await loadComponent("RunPanel");
const panel = state => renderToStaticMarkup(createElement(RunPanel, { state, selection: INITIAL_SELECTION, onEvent: () => {}, onStart: () => {}, onDecide: () => {}, onCancel: () => {} }));
function active(...samples) {
  let state = reduceRun(INITIAL_RUN, { type: "start", reports, context });
  state = reduceRun(state, { type: "started", token: state.view.token, run });
  state = reduceRun(state, { type: "snapshot", snapshot: { schemaVersion: 1, state: "running", run, plan_ref: "p", mode: "full", cancel_requested: false, outcome: null } });
  return samples.reduce((current, progress) => reduceRun(current, { type: "progress", progress }), state);
}

test("the Full monitor: pass and trace from the sample, indeterminate bar, no total or percentage", () => {
  let html = panel(active(sample("3", "local_validation", T, "2000")));
  assert.match(html, /<p>Controlli sui singoli tracciati: tracciato T<\/p>/);
  assert.match(html, /<dt>Tracciati nell&#x27;analisi<\/dt><dd>S, T<\/dd><dt>Record del tracciato T<\/dt><dd>2000<\/dd>/);
  assert.match(html, /<progress aria-label="Avanzamento dell&#x27;analisi"><\/progress>/);
  html = panel(active(sample("3", "local_validation", T, "2000"), sample("4", "cross_record_validation", null, "0", "150")));
  assert.match(html, /<p>Controlli tra tracciati<\/p>/);
  assert.doesNotMatch(html, /Record del tracciato/, "PASS2 has no running trace");
  assert.match(html, /<dt>Record nei controlli tra tracciati<\/dt><dd>150<\/dd>/);
  // An older sample never moves the monitor back to PASS1.
  html = panel(active(sample("4", "cross_record_validation", null, "0", "150"), sample("3", "local_validation", T, "2000")));
  assert.match(html, /<p>Controlli tra tracciati<\/p>/);
  assert.match(panel(active(sample("9", "finalizing_report", S, "5"))), /<p>Scrittura del report<\/p>/, "no trace beside the report writing");
  for (const text of [html, panel(active(sample("1", "preflight", null, "0")))]) {
    assert.doesNotMatch(text, /\d\s?%|percentual|stimat|ETA|rimanent|di 2\b|totale/i);
  }
});

test("Full terminals: Rust's words, last phase only without a report, counters of the last sample", () => {
  const done = (outcome, ...samples) => reduceRun(active(...samples), { type: "terminal", run, outcome, acknowledged: true });
  let html = panel(done(validation("cancelled", "partial", false, false), sample("6", "cross_record_validation", null, "0", "40")));
  assert.match(html, /<h3>Analisi annullata<\/h3>/);
  assert.match(html, /<p>Ultima fase comunicata dal servizio: Controlli tra tracciati\.<\/p>/);
  assert.match(html, /<dt>Record nei controlli tra tracciati<\/dt><dd>40<\/dd>/);
  html = panel(done(validation("completed_invalid", "complete", false, true), sample("9", "finalizing_report", null, "0", "4")));
  assert.match(html, /<h3>Violazioni rilevate<\/h3><p>Il report è disponibile in Risultati\.<\/p>/);
  assert.doesNotMatch(html, /Ultima fase/);
  html = panel(done(validation("completed_valid", "partial", false, true), sample("9", "finalizing_report", null, "0", "4")));
  assert.match(html, /<h3>Controlli incompleti<\/h3>/);
  assert.doesNotMatch(html, /Nessuna violazione nei controlli eseguiti/);
});
