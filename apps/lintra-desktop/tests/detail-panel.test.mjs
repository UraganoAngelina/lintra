// T-MIG-P6-03: selected diagnostic and «Dettaglio controllo». Rust pins the
// selection and sends every value; these tests check that the renderer shows
// those values as plain text (never HTML), keeps spaces, says when Rust does
// not expose a value, marks the selected row, and drops late detail replies.
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { afterEach, test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { loadComponent } from "./tsx.mjs";
import { NO_FILTERS } from "../src/pages.ts";

globalThis.window = { crypto: globalThis.crypto };
afterEach(clearMocks);
const { DetailPanel, Results, ResultsGrid, openDetail } = await loadComponent("Results");

const RUN = { schemaVersion: 1, instance_id: "a".repeat(32), run_id: "1" };
const Q = "q_" + "0".repeat(32);
const D = "d_" + "1".repeat(32);
const S = "s_" + "2".repeat(32);
const preview = (text, complete = true) => ({ schemaVersion: 1, text, complete });
const part = (kind, text, complete = true) => ({ schemaVersion: 1, component_ref: "x_" + kind, kind, preview: preview(text, complete) });
const SELECTION = { query_ref: Q, ordinal: "41", diagnostic_ref: D };
const detail = (components, visibility = "available") => ({ schemaVersion: 1, selection_ref: S, query_ref: Q, diagnostic_ref: D, run: RUN,
  line: "1234567", trace_ref: "trace_0", field_position: { schemaVersion: 1, start: "4", end: "6", units: "one_based_inclusive_bytes" },
  actual_visibility: visibility, components, source_count: "2" });
const TRACES = new Map([["trace_0", preview("T1")]]);
const panel = state => renderToStaticMarkup(createElement(DetailPanel, { state, traces: TRACES, onRetry() {} }));
const settle = () => new Promise(resolve => setImmediate(resolve));
async function drain() { for (let i = 0; i < 10; i++) await settle(); }

/** Answers results_select with `payload`, or holds it until released. */
function select(payload, { hold = false, fail = false } = {}) {
  let release = () => {};
  const gate = hold ? new Promise(resolve => { release = resolve; }) : Promise.resolve();
  const seen = [];
  mockIPC(async (_command, bytes) => {
    const request = JSON.parse(new TextDecoder().decode(bytes));
    seen.push(request);
    await gate;
    const reply = fail ? { schemaVersion: 1, request_id: request.request_id, kind: "error", error: { schemaVersion: 1, code: "IPC_STALE_CONTEXT", category: "technical" } }
      : { schemaVersion: 1, request_id: request.request_id, kind: "result", payload };
    return new TextEncoder().encode(JSON.stringify(reply)).buffer;
  });
  return { seen, release: () => release() };
}

test("a selection asks Rust for the row's exact refs and shows its detail", async () => {
  const ready = detail([part("code", "E_X"), part("message", "Valore non ammesso"), part("actual", "XX")]);
  const { seen } = select(ready);
  const states = [];
  openDetail(SELECTION, () => true, state => states.push(state));
  await drain();
  assert.deepEqual(seen.map(request => [request.operation, request.payload]),
    [["results_select", { schemaVersion: 1, query_ref: Q, diagnostic_ref: D }]]);
  assert.deepEqual(states.map(state => state.status), ["loading", "ready"]);
  assert.equal(states[1].detail.selection_ref, S);
  assert.deepEqual(states[1].selection, SELECTION);
});

test("a detail that cannot be read is a failure, and a late one is dropped", async () => {
  select(null, { fail: true });
  const failed = [];
  openDetail(SELECTION, () => true, state => failed.push(state));
  await drain();
  assert.deepEqual(failed.map(state => state.status), ["loading", "failed"]);

  // The user chose another row while this reply was in flight.
  const gate = select(detail([part("code", "E_OLD"), part("message", "vecchia"), part("actual", "1")]), { hold: true });
  let live = true;
  const late = [];
  openDetail(SELECTION, () => live, state => late.push(state));
  await drain();
  live = false; gate.release();
  await drain();
  assert.deepEqual(late.map(state => state.status), ["loading"]);

  const failing = select(null, { fail: true, hold: true });
  live = true;
  const lateFailure = [];
  openDetail(SELECTION, () => live, state => lateFailure.push(state));
  await drain();
  live = false; failing.release();
  await drain();
  assert.deepEqual(lateFailure.map(state => state.status), ["loading"]);
});

test("the panel shows Rust's values as plain text, spaces kept", () => {
  const hostile = "<img src=x onerror=alert(1)>";
  const html = panel({ status: "ready", selection: SELECTION, detail: detail([
    part("code", "E_<b>X</b>"), part("message", hostile), part("field_name", "COGNOME"),
    part("sioc_record_id", "R-77"), part("expected", "valore in <i>dominio</i>"), part("actual", "  A  B "),
    part("fiscal_code", "RSSMRA80A01H501U"), part("episode_id", "EP-9")]) });
  assert.match(html, /Dettaglio controllo/);
  assert.match(html, /Codice anomalia/);
  assert.match(html, /E_&lt;b&gt;X&lt;\/b&gt;/);
  assert.match(html, /&lt;img src=x onerror=alert\(1\)&gt;/);
  assert.doesNotMatch(html, /<img|<b>|<i>/);
  assert.match(html, /valore in &lt;i&gt;dominio&lt;\/i&gt;/);
  // The value read keeps every space, inside a box that shows its edges.
  assert.match(html, /class="detail-text detail-value">  A  B <\/span>/);
  assert.match(html, /<dt>Riga<\/dt><dd>1\.234\.567<\/dd>/);
  assert.match(html, /<dt>Tracciato<\/dt><dd>T1<\/dd>/);
  assert.match(html, /COGNOME/);
  assert.match(html, /R-77/);
  assert.match(html, /aria-label="Diagnostica 42"/);
  // Data minimisation: fiscal code and episode are not shown.
  assert.doesNotMatch(html, /RSSMRA80A01H501U|EP-9/);
  // Domains belong to P6-05/06 (guide and byte position: P6-04, field-guide.test.mjs).
  assert.doesNotMatch(html, /dominio ammesso|Fonti/i);
  // Context is read only on request.
  assert.match(html, />Mostra il contesto</);
});

test("an incomplete preview is marked and offers the full text from Rust", () => {
  const html = panel({ status: "ready", selection: SELECTION, detail: detail([
    part("code", "E_X"), part("message", "Inizio del messaggio", false), part("actual", "1")]) });
  assert.match(html, /Inizio del messaggio<span class="text-muted">…<\/span>/);
  assert.match(html, />Leggi il testo completo</);
  const complete = panel({ status: "ready", selection: SELECTION, detail: detail([part("code", "E_X"), part("message", "Intero")]) });
  assert.doesNotMatch(complete, /Leggi il testo completo/);
});

test("a value Rust does not expose is said so, not shown as empty", () => {
  const scope = panel({ status: "ready", selection: SELECTION,
    detail: detail([part("code", "E_RECORD"), part("message", "Record")], "local_report_only_record_scope") });
  assert.match(scope, /Il valore copre l&#x27;intero record: si consulta solo nel report locale\./);
  const absent = panel({ status: "ready", selection: SELECTION,
    detail: detail([part("code", "E_MISSING"), part("message", "Campo assente")], "not_present") });
  assert.match(absent, /Nessun valore letto\./);
  assert.doesNotMatch(absent, /Valore atteso/);
});

test("panel states: nothing selected, loading, failed", () => {
  assert.match(panel({ status: "none" }), /Seleziona una riga per consultare causa e valori\./);
  assert.match(panel({ status: "loading", selection: SELECTION }), /role="status">Lettura del dettaglio della diagnostica 42…/);
  const failed = panel({ status: "failed", selection: SELECTION });
  assert.match(failed, /role="alert"><p>Non è stato possibile leggere il dettaglio della diagnostica 42\.<\/p>/);
  assert.match(failed, />Riprova</);
});

const row = (ordinal, line) => ({ schemaVersion: 1, diagnostic_ref: `d_${ordinal}`, ordinal, line, trace_ref: "trace_0",
  code: preview("E_X"), field: null, message: preview("Errore"), expected: null, actual: null });
const page = { schemaVersion: 1, query_ref: Q, run: RUN, page_generation: "1", offset: "0", next_offset: "3", has_more: false,
  total_matches: "3", total_diagnostics: "3", rows: [row("0", "1"), row("1", "2"), row("2", "3")] };

test("the grid marks the selected row by its ordinal in the query", () => {
  const html = renderToStaticMarkup(createElement(ResultsGrid, { page, traces: TRACES, selected: "1", onSelect() {} }));
  assert.match(html, /<table class="grid-table" role="grid" aria-readonly="true"/);
  assert.equal(html.match(/aria-selected="true"/g)?.length, 1);
  assert.equal(html.match(/aria-selected="false"/g)?.length, 2);
  assert.match(html, /aria-rowindex="3" aria-selected="true" class="selectable is-selected"/);
  assert.match(html, /frecce per scegliere una riga/);
  // Without a selection callback the grid stays a plain table (P6-01 use).
  const plain = renderToStaticMarkup(createElement(ResultsGrid, { page, traces: TRACES }));
  assert.doesNotMatch(plain, /role="grid"|aria-selected/);
});

test("Risultati puts the panel next to the rows only when there are rows", () => {
  const about = { status: "none" }, overview = { status: "none" };
  const view = { status: "ready", run: RUN, query_ref: Q, total_matches: "3", total_diagnostics: "3", page: 0n, loading: false, pageFailed: false, current: page };
  const props = { about, overview, reportAvailable: true, filters: NO_FILTERS, onFilter() {}, onPage() {}, onRetry() {} };
  const html = renderToStaticMarkup(createElement(Results, { ...props, view,
    detail: { status: "loading", selection: { ...SELECTION, ordinal: "2" } }, onSelect() {} }));
  assert.match(html, /class="results-split with-detail"/);
  assert.match(html, /aria-rowindex="4" aria-selected="true"/);
  assert.match(html, /Dettaglio controllo/);
  const empty = renderToStaticMarkup(createElement(Results, { ...props,
    view: { ...view, total_matches: "0", current: { ...page, rows: [], total_matches: "0" } } }));
  assert.doesNotMatch(empty, /Dettaglio controllo/);
  assert.match(empty, /class="results-split"/);
});

test("no renderer source injects HTML", () => {
  const sources = new URL("../src/", import.meta.url);
  for (const name of readdirSync(sources).filter(file => /\.(ts|tsx)$/.test(file))) {
    const text = readFileSync(new URL(name, sources), "utf8");
    assert.doesNotMatch(text, /dangerouslySetInnerHTML|innerHTML|outerHTML|insertAdjacentHTML/, name);
  }
});
