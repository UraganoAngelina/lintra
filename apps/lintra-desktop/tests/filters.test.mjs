// T-MIG-P6-02: the five Slint filters and stale-safe queries. Rust applies
// the filters (exact, AND, no trim or case folding); these tests check that
// the renderer sends them as typed, flags a line Rust would reject, says when
// a result is filtered, and drops every reply of a query left behind.
import assert from "node:assert/strict";
import { afterEach, mock, test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { loadComponent } from "./tsx.mjs";
import { NO_FILTERS, filtering, validLine } from "../src/pages.ts";

globalThis.window = { crypto: globalThis.crypto };
afterEach(() => { clearMocks(); mock.timers.reset(); });
const { FilterBar, Results, openQuery } = await loadComponent("Results");

const RUN = { schemaVersion: 1, instance_id: "a".repeat(32), run_id: "1" };
const Q = "q_" + "1".repeat(32);
const preview = text => ({ schemaVersion: 1, text, complete: true });
const status = (state, matches = null, total = null) =>
  ({ schemaVersion: 1, run: RUN, query_ref: Q, state, total_matches: matches, total_diagnostics: total });
const firstPage = { schemaVersion: 1, query_ref: Q, run: RUN, page_generation: "1", offset: "0", next_offset: "1", has_more: false,
  total_matches: "1", total_diagnostics: "5",
  rows: [{ schemaVersion: 1, diagnostic_ref: "d_" + "0".repeat(32), ordinal: "0", line: "7", trace_ref: "trace_0",
    code: preview("E_X"), field: null, message: preview("Errore sintetico"), expected: null, actual: null }] };

/** Answers each operation from `replies`, recording the requests. */
function ipc(replies, seen = []) {
  mockIPC((_command, bytes) => {
    const request = JSON.parse(new TextDecoder().decode(bytes));
    seen.push(request);
    const payload = replies[request.operation];
    if (payload === undefined) throw new Error(`unexpected ${request.operation}`);
    return new TextEncoder().encode(JSON.stringify({ schemaVersion: 1, request_id: request.request_id, kind: "result", payload })).buffer;
  });
  return seen;
}
const settle = () => new Promise(resolve => setImmediate(resolve));
async function drain() { for (let i = 0; i < 10; i++) await settle(); }

test("the line filter follows ResultsFilter::parse_line; other filters are free text", () => {
  for (const ok of ["", "1", "007", "18446744073709551615"]) assert.equal(validLine(ok), true, ok);
  for (const bad of ["0", "000", " 7", "7 ", "-1", "+1", "1e3", "٣", "18446744073709551616"]) assert.equal(validLine(bad), false, bad);
  assert.equal(filtering(NO_FILTERS), false);
  assert.equal(filtering({ ...NO_FILTERS, record: " " }), true, "a space is a restriction, never trimmed away");
  assert.ok(Object.isFrozen(NO_FILTERS));
});

test("the filter bar has the five Slint filters with visible labels, as typed", () => {
  const applied = { code: " E_X ", field: "F1", record: "", fiscal_code: "abc", line: "007" };
  const html = renderToStaticMarkup(createElement(FilterBar, { applied, onApply: () => {} }));
  const labels = [...html.matchAll(/<label for="filter-([a-z_]+)">([^<]+)<\/label>/g)].map(m => [m[1], m[2]]);
  assert.deepEqual(labels, [["code", "Codice"], ["field", "ID campo"], ["record", "ID record"], ["fiscal_code", "Codice fiscale"], ["line", "Riga"]]);
  assert.match(html, /id="filter-code"[^>]*value=" E_X "/);
  assert.match(html, /id="filter-fiscal_code"[^>]*value="abc"/);
  assert.match(html, /id="filter-line"[^>]*inputMode="numeric"[^>]*aria-invalid="false"[^>]*value="007"/);
  assert.match(html, /<button[^>]*type="submit"[^>]*>Applica filtri<\/button>/);
  assert.match(html, /<button[^>]*type="button"[^>]*aria-disabled="false"[^>]*>Azzera<\/button>/);
  assert.match(html, /maiuscole e spazi contano/);
  // No free-text search, no sort and no severity chips of the reference.
  assert.doesNotMatch(html, /Cerca|Ordina|Tutti|Avvisi|Validi/);
  const empty = renderToStaticMarkup(createElement(FilterBar, { applied: NO_FILTERS, onApply: () => {} }));
  assert.match(empty, /aria-disabled="true"[^>]*>Azzera</);
});

const about = { status: "none" };
const overview = { status: "none" };
const results = (view, filters) => renderToStaticMarkup(createElement(Results,
  { view, about, overview, reportAvailable: true, filters, onFilter: () => {}, onPage: () => {}, onRetry: () => {} }));
const ready = extra => ({ status: "ready", run: RUN, query_ref: Q, total_matches: "1", total_diagnostics: "5",
  page: 0n, loading: false, pageFailed: false, current: firstPage, ...extra });
const CODE = { ...NO_FILTERS, code: "E_X" };

test("Risultati says when the rows are filtered, and never shows old rows while filters apply", () => {
  const building = results({ status: "building", run: RUN }, CODE);
  assert.match(building, /role="status">Applicazione dei filtri…</);
  assert.doesNotMatch(building, /<table/);
  assert.match(results({ status: "building", run: RUN }, NO_FILTERS), /Preparazione dei risultati…/);
  const failed = results({ status: "failed", run: RUN }, CODE);
  assert.match(failed, /role="alert"><p>Non è stato possibile applicare i filtri: nessuna riga è mostrata\.<\/p>/);
  assert.doesNotMatch(failed, /<table/);
  assert.match(results(ready({ total_matches: "0", current: null }), CODE), /Nessuna diagnostica corrisponde ai filtri\./);
  assert.match(results(ready({ total_matches: "0", total_diagnostics: "0", current: null }), NO_FILTERS), /Il report non contiene diagnostiche\./);
  assert.match(results(ready({ total_diagnostics: "1000000" }), CODE), /<p class="grid-range">1 corrispondenza su 1\.000\.000 diagnostiche<\/p>/);
  assert.match(results(ready(), NO_FILTERS), /<p class="grid-range">Righe 1–1 di 1<\/p>/);
  assert.match(results(ready(), CODE), /<form class="grid-filters"/);
  // Without a query there is nothing to filter.
  assert.doesNotMatch(results({ status: "none" }, NO_FILTERS), /grid-filters/);
});

test("a query sends the five filters exactly as typed and shows its first page", async () => {
  const filters = { code: " E_X", field: "", record: "Ab", fiscal_code: "", line: "007" };
  const seen = ipc({ results_query: status("ready", "1", "5"), results_page: firstPage });
  const views = [];
  let view = { status: "none" };
  openQuery(RUN, filters, () => true, update => { view = typeof update === "function" ? update(view) : update; views.push(view.status); }, () => {});
  await drain();
  assert.deepEqual(seen[0].payload, { schemaVersion: 1, run: RUN, filters: { schemaVersion: 1, ...filters } });
  assert.equal(seen[1].operation, "results_page");
  assert.deepEqual(views, ["building", "ready", "ready"]);
  assert.equal(view.current.rows[0].line, "7");
});

/** IPC whose replies to `held` operations wait for `release()`; `failing`
 * operations get an error reply, so the client call rejects. */
function gated(replies, held, failing = []) {
  let release;
  const gate = new Promise(resolve => { release = resolve; });
  mockIPC(async (_command, bytes) => {
    const request = JSON.parse(new TextDecoder().decode(bytes));
    if (held.includes(request.operation)) await gate;
    const reply = failing.includes(request.operation)
      ? { schemaVersion: 1, request_id: request.request_id, kind: "error", error: { schemaVersion: 1, code: "technical_failure", message: "synthetic", retryable: false } }
      : { schemaVersion: 1, request_id: request.request_id, kind: "result", payload: replies[request.operation] };
    return new TextEncoder().encode(JSON.stringify(reply)).buffer;
  });
  return () => release();
}
const statuses = updates => updates.map(update => typeof update === "function" ? "page" : update.status);

test("replies of a query left behind never reach the view", async () => {
  mock.timers.enable({ apis: ["setTimeout"] });
  // 0. Switch while the query reply itself is in flight.
  const open = gated({ results_query: status("ready", "1", "5"), results_page: firstPage }, ["results_query"]);
  let opening = true;
  const first = [];
  openQuery(RUN, CODE, () => opening, update => first.push(update), () => first.push("kept"));
  await drain();
  opening = false; open();
  await drain();
  assert.deepEqual(statuses(first), ["building"]);

  // 1. Switch while still building: the poll timer is cleared.
  ipc({ results_query: status("building"), results_query_state: status("ready", "1", "5"), results_page: firstPage });
  let live = true;
  const updates = [];
  const stop = openQuery(RUN, CODE, () => live, update => updates.push(update), () => updates.push("kept"));
  await drain();
  live = false; stop();
  mock.timers.tick(1000);
  await drain();
  assert.deepEqual(statuses(updates), ["building"]);

  // 2. Switch while the state reply is in flight (timer already fired).
  let release = gated({ results_query: status("building"), results_query_state: status("ready", "1", "5"), results_page: firstPage }, ["results_query_state"]);
  live = true;
  const state = [];
  openQuery(RUN, CODE, () => live, update => state.push(update), () => state.push("kept"));
  await drain();
  mock.timers.tick(200);
  await drain();
  live = false; release();
  await drain();
  assert.deepEqual(statuses(state), ["building"]);

  // 3. Switch while the first page is in flight.
  release = gated({ results_query: status("ready", "1", "5"), results_page: firstPage }, ["results_page"]);
  live = true;
  const page = [];
  openQuery(RUN, CODE, () => live, update => page.push(update), () => page.push("kept"));
  await drain();
  live = false; release();
  await drain();
  assert.deepEqual(statuses(page), ["building", "ready"], "the late page is neither kept nor shown");
});

test("a failed query shows a failure, not the rows of the previous filters", async () => {
  mock.timers.enable({ apis: ["setTimeout"] });
  ipc({ results_query: status("building"), results_query_state: status("failed") });
  const updates = [];
  openQuery(RUN, CODE, () => true, update => updates.push(update), () => {});
  await drain();
  mock.timers.tick(200);
  await drain();
  assert.deepEqual(statuses(updates), ["building", "failed"]);
  // A technical error arriving after the switch is dropped as well.
  const release = gated({ results_query: status("building") }, ["results_query_state"], ["results_query_state"]);
  let live = true;
  const dropped = [];
  openQuery(RUN, CODE, () => live, update => dropped.push(update), () => {});
  await drain();
  mock.timers.tick(200);
  await drain();
  live = false; release();
  await drain();
  assert.deepEqual(statuses(dropped), ["building"]);
});
