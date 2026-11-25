// T-MIG-P6-01: paginated virtualised DataGrid. Model arithmetic on the
// decimal u64 counters of Rust, the two-page cache, stale replies, and the
// real ResultsGrid/Results components rendered with react-dom/server.
import assert from "node:assert/strict";
import { test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { loadComponent } from "./tsx.mjs";
import { MAX_PAGE_ROWS } from "../src/ipc.ts";
import { CACHE_PAGES, NO_FILTERS, PAGE_ROWS, acceptPage, cached, pageCount, pageItems, pageOf, pageOffset, rangeText, remember } from "../src/pages.ts";

const RUN = { schemaVersion: 1, instance_id: "i1", run_id: "r1" };
const MAX_U64 = "18446744073709551615";
function page({ query = "q1", run = RUN, offset = "0", count = PAGE_ROWS, total = "1000000" } = {}) {
  const start = BigInt(offset);
  const rows = Array.from({ length: count }, (_, index) => ({
    schemaVersion: 1, diagnostic_ref: `d${start + BigInt(index)}`, ordinal: (start + BigInt(index)).toString(),
    line: (start + BigInt(index) + 1n).toString(), trace_ref: "trace_0",
    code: { schemaVersion: 1, text: "E_CROSS_FIELD", complete: true },
    field: index % 3 === 0 ? null : { schemaVersion: 1, text: "Synthetic detail", complete: index % 5 !== 0 },
    message: { schemaVersion: 1, text: "Lunghezza del record non valida", complete: true },
    expected: index % 3 === 0 ? null : { schemaVersion: 1, text: "6 byte", complete: true },
    actual: index % 3 === 0 ? null : { schemaVersion: 1, text: "4 byte", complete: true },
  }));
  const next = (start + BigInt(count)).toString();
  return { schemaVersion: 1, query_ref: query, run, page_generation: "1", offset, rows, next_offset: next,
    has_more: BigInt(next) < BigInt(total), total_matches: total, total_diagnostics: total };
}
const ready = (extra = {}) => ({ status: "ready", run: RUN, query_ref: "q1", total_matches: "1000000", total_diagnostics: "1000000",
  page: 0n, loading: true, pageFailed: false, current: null, ...extra });

test("page arithmetic stays exact on u64 counters, without Number casts", () => {
  assert.equal(PAGE_ROWS, 128);
  assert.equal(PAGE_ROWS, MAX_PAGE_ROWS);
  assert.equal(pageCount("0"), 0n);
  assert.equal(pageCount("1"), 1n);
  assert.equal(pageCount("128"), 1n);
  assert.equal(pageCount("129"), 2n);
  assert.equal(pageCount(MAX_U64), (BigInt(MAX_U64) + 127n) / 128n);
  assert.equal(pageOffset(0n), "0");
  assert.equal(pageOffset(3n), "384");
  const last = pageCount(MAX_U64) - 1n;
  assert.equal(pageOffset(last), (last * 128n).toString());
  assert.ok(BigInt(pageOffset(last)) <= BigInt(MAX_U64));
  assert.equal(pageOf("383"), 2n);
  assert.equal(pageOf("384"), 3n);
  assert.throws(() => pageOffset(-1n), RangeError);
});

test("the pager shows at most 7 items for any page count, always first and last", () => {
  assert.deepEqual(pageItems(0n, 0n), []);
  assert.deepEqual(pageItems(0n, 1n), [0n]);
  assert.deepEqual(pageItems(2n, 5n), [0n, 1n, 2n, 3n, 4n]);
  assert.deepEqual(pageItems(0n, 100n), [0n, 1n, 2n, 3n, 4n, "gap", 99n]);
  assert.deepEqual(pageItems(50n, 100n), [0n, "gap", 49n, 50n, 51n, "gap", 99n]);
  assert.deepEqual(pageItems(99n, 100n), [0n, "gap", 95n, 96n, 97n, 98n, 99n]);
  const huge = pageCount(MAX_U64);
  for (const current of [0n, 1n, 3n, 4n, huge / 2n, huge - 5n, huge - 4n, huge - 1n]) {
    const items = pageItems(current, huge);
    assert.ok(items.length <= 7, String(current));
    assert.equal(items[0], 0n);
    assert.equal(items.at(-1), huge - 1n);
    assert.ok(items.includes(current), String(current));
    const pages = items.filter(item => item !== "gap");
    assert.deepEqual([...pages].sort((a, b) => (a < b ? -1 : 1)), pages, "ascending, no duplicates");
    assert.equal(new Set(pages).size, pages.length);
  }
});

test("the range text groups digits and states an empty page explicitly", () => {
  assert.equal(rangeText(page({ offset: "128" })), "Righe 129–256 di 1.000.000");
  assert.equal(rangeText(page({ count: 0, offset: "0", total: "0" })), "Nessuna riga su 0");
});

test("the renderer keeps at most two pages, all of the current query", () => {
  assert.equal(CACHE_PAGES, 2);
  let cache = [];
  for (let index = 0; index < 50; index++) cache = remember(cache, page({ offset: pageOffset(BigInt(index)) }));
  assert.equal(cache.length, 2);
  assert.deepEqual(cache.map(entry => entry.offset), [pageOffset(49n), pageOffset(48n)]);
  assert.ok(cache.reduce((rows, entry) => rows + entry.rows.length, 0) <= 2 * PAGE_ROWS);
  // Re-reading a page does not duplicate it; a new query drops the old one.
  cache = remember(cache, page({ offset: pageOffset(48n) }));
  assert.deepEqual(cache.map(entry => entry.offset), [pageOffset(48n), pageOffset(49n)]);
  cache = remember(cache, page({ query: "q2" }));
  assert.deepEqual(cache.map(entry => entry.query_ref), ["q2"]);
  assert.equal(cached(cache, "q2", "0")?.query_ref, "q2");
  assert.equal(cached(cache, "q1", pageOffset(48n)), undefined);
});

test("late pages of another page, query or run never replace the requested one", () => {
  const wanted = ready({ page: 1n });
  assert.equal(acceptPage(wanted, page({ offset: "0" })), wanted);
  assert.equal(acceptPage(wanted, page({ offset: "128", query: "q0" })), wanted);
  assert.equal(acceptPage(wanted, page({ offset: "128", run: { ...RUN, run_id: "r0" } })), wanted);
  const building = { status: "building", run: RUN };
  assert.equal(acceptPage(building, page()), building);
  const accepted = acceptPage(wanted, page({ offset: "128" }));
  assert.equal(accepted.loading, false);
  assert.equal(accepted.current.offset, "128");
});

const { ResultsGrid, Results, ROW_HEIGHT, OVERSCAN } = await loadComponent("Results");
const count = (html, pattern) => (html.match(pattern) ?? []).length;

test("a full page of a million matches puts only the rows in view in the DOM", () => {
  const html = renderToStaticMarkup(createElement(ResultsGrid, { page: page({ offset: "999872", total: "1000000" }) }));
  const rows = count(html, /<tr aria-rowindex="(?!1")\d+">/g);
  // initialRect: ten rows of viewport, plus the overscan below them.
  assert.ok(rows > 0 && rows <= 10 + OVERSCAN, String(rows));
  assert.ok(rows < PAGE_ROWS);
  assert.equal(ROW_HEIGHT, 56);
  // One header row, absolute row indices over the filtered total + header.
  assert.match(html, /<table class="grid-table" aria-rowcount="1000001">/);
  assert.match(html, /<tr aria-rowindex="1"><th scope="col" class="col-ordinal">N\.<\/th><th scope="col" class="col-line">Riga<\/th><th scope="col" class="col-trace">Tracciato<\/th><th scope="col" class="col-field">Campo<\/th><th scope="col" class="col-code">Controllo<\/th><th scope="col" class="col-reason">Motivo<\/th><\/tr>/);
  assert.match(html, /<tr aria-rowindex="999874"><td class="col-ordinal">999.873<\/td><td class="col-line">999.873<\/td>/);
  // The spacer after the rendered rows keeps the page scroll height.
  assert.equal(count(html, /<tr aria-hidden="true">/g), 1);
  // The scroll viewport is a focusable, labelled region.
  assert.match(html, /<div class="grid-viewport" tabindex="0" role="region" aria-label="Diagnostiche, righe 999\.873–1\.000\.000 di 1\.000\.000">/);
});

test("rows carry only the row DTO: references, previews and the reason", () => {
  const traces = new Map([["trace_0", { schemaVersion: 1, text: "T1", complete: true }]]);
  const html = renderToStaticMarkup(createElement(ResultsGrid, { page: page(), traces }));
  // The trace is named only through the map Rust sent (results_overview).
  assert.match(html, /<td class="col-trace">T1<\/td>/);
  assert.match(renderToStaticMarkup(createElement(ResultsGrid, { page: page() })), /<td class="col-trace"><span class="text-muted">—<\/span><\/td>/);
  // Reason: message, then what was expected and what was found.
  assert.match(html, /<span class="reason-message">Lunghezza del record non valida<\/span><span class="reason-detail"><span class="reason-key">Atteso<\/span> 6 byte<span aria-hidden="true"> · <\/span><span class="reason-key">Trovato<\/span> <span class="font-mono">4 byte<\/span><\/span>/);
  // Without expected and found the row shows the message alone.
  assert.match(html, /<span class="reason"><span class="reason-message">Lunghezza del record non valida<\/span><\/span>/);
  assert.match(html, /<td class="col-code"><span class="[^"]*font-mono[^"]*">E_CROSS_FIELD<\/span><\/td>/);
  assert.match(html, /<td class="col-field"><span class="text-muted">—<\/span><\/td>/);
  // An incomplete preview is announced as such.
  assert.match(html, /Synthetic detail…<span class="sr-only"> \(anteprima incompleta\)<\/span>/);
  assert.doesNotMatch(html, /trace_0|d0\b|raw_hex|fiscal|record_identity/);
  assert.equal(count(html, /\sstyle=/g), 0);
});

const noop = () => {};
const about = { status: "ready", about: { schemaVersion: 1, report_ref: "rep", run: RUN, app_version: "0.1.1-rc.3",
  validator_version: { schemaVersion: 1, text: "0.1.1-rc.3", complete: true },
  flow_id: { schemaVersion: 1, text: "SYNTHETIC", complete: true }, schema_version: { schemaVersion: 1, text: "v1", complete: true },
  schema_hash: { schemaVersion: 1, text: "99f89afe8552edb6485ce48529e5bf099990d0f47bf76ac3b013169458e779ef", complete: true },
  mode: "local_only", status: "completed_invalid", completeness: "complete", truncated: false, exportable: true,
  trace_count: "1", reference_dataset_count: "0", bundle: {} } };
const overview = { status: "ready", overview: { schemaVersion: 1, run: RUN, records_processed: "1200000", valid_records: "200000",
  invalid_records: "1000000", total_errors: "1000000",
  by_code: [{ schemaVersion: 1, code: { schemaVersion: 1, text: "E_RECORD_LENGTH", complete: true }, count: "1000000" }],
  by_trace: [{ schemaVersion: 1, trace_ref: "trace_0", trace: { schemaVersion: 1, text: "T1", complete: true }, count: "1000000" }] } };
const results = props => renderToStaticMarkup(createElement(Results, { about, overview, reportAvailable: true, filters: NO_FILTERS, onFilter: noop, onPage: noop, onRetry: noop, ...props }));

test("Risultati states: no report, building, failed, empty and paged", () => {
  const none = renderToStaticMarkup(createElement(Results, { view: { status: "none" }, about: { status: "none" }, overview: { status: "none" }, reportAvailable: false, filters: NO_FILTERS, onFilter: noop, onPage: noop, onRetry: noop }));
  assert.match(none, /Nessun report disponibile\./);
  assert.doesNotMatch(none, /<table/);
  assert.match(results({ view: { status: "building", run: RUN } }), /role="status">Preparazione dei risultati…</);
  const failed = results({ view: { status: "failed", run: RUN } });
  assert.match(failed, /role="alert"><p>Non è stato possibile preparare i risultati del report\.<\/p><button[^>]*>Riprova<\/button>/);
  const empty = results({ view: ready({ total_matches: "0", total_diagnostics: "0", loading: false }) });
  assert.match(empty, /Il report non contiene diagnostiche\./);
  assert.doesNotMatch(empty, /<table|Nessuna violazione/);
  const shown = results({ view: ready({ page: 1n, loading: false, current: page({ offset: "128" }) }) });
  assert.match(shown, /<nav class="pager" aria-label="Pagine delle diagnostiche">/);
  assert.match(shown, /aria-current="page" aria-label="Pagina 2"/);
  // Page 2 of 7.813: pages 1–5, a gap, the last page.
  assert.equal(count(shown, /aria-label="Pagina \d[\d.]*"/g), 6);
  assert.match(shown, /aria-label="Pagina 7\.813"/);
  const failedPage = results({ view: ready({ page: 1n, loading: false, pageFailed: true, current: page() }) });
  assert.match(failedPage, /role="alert"><p>Non è stato possibile leggere questa pagina dei risultati\.<\/p>/);
});

test("the summary shows only what Rust reported, with the outcome in words", () => {
  const html = results({ view: ready({ loading: false, current: page() }) });
  assert.match(html, /<h2>Record analizzati<\/h2><p class="kpi-value">1\.200\.000<\/p>/);
  assert.match(html, /<h2>Diagnostiche nel report<\/h2><p class="kpi-value tone-error">1\.000\.000<\/p>/);
  assert.match(html, /<h2>Record con violazioni<\/h2><p class="kpi-value tone-error">1\.000\.000<\/p><p class="kpi-note">Senza violazioni: 200\.000<\/p>/);
  assert.match(html, /<h2>Esito<\/h2><p class="kpi-value kpi-text tone-error">Violazioni rilevate<\/p><p class="kpi-note">SYNTHETIC · v1<\/p>/);
  // No severity Lintra does not have, no "conforme".
  assert.doesNotMatch(html, /conform|Avviso|Warning|Valido/i);
  const partial = results({ view: ready({ loading: false, current: page() }),
    about: { status: "ready", about: { ...about.about, status: "completed_valid", completeness: "partial" } } });
  assert.match(partial, /kpi-value kpi-text tone-warn">Nessuna violazione nei controlli eseguiti, controlli incompleti</);
  // Counts not read yet are a dash, never a zero.
  assert.match(results({ view: ready(), overview: { status: "loading" } }), /<h2>Record analizzati<\/h2><p class="kpi-value">—<\/p>/);
});

const { Charts, codeSeries, permille, percentText, MAX_SERIES } = await loadComponent("Charts");
const code = (text, count) => ({ schemaVersion: 1, code: { schemaVersion: 1, text, complete: true }, count });

test("the validity share is exact on u64 and never rounded up to 100%", () => {
  assert.equal(permille(0n, 0n), null);
  assert.equal(percentText(permille(982n, 1000n)), "98,2%");
  assert.equal(percentText(permille(999_999n, 1_000_000n)), "99,9%");
  assert.equal(percentText(permille(1n, 3n)), "33,3%");
  assert.equal(percentText(permille(1n, 1000n)), "0,1%");
  assert.equal(percentText(permille(1_000_000n, 1_000_000n)), "100,0%");
  const max = 18446744073709551615n;
  assert.equal(percentText(permille(max - 1n, max)), "99,9%");
});

test("codes are ordered by count and the tail becomes one honest entry", () => {
  const few = codeSeries({ by_code: [code("E_A", "2"), code("E_B", "9"), code("E_C", "0")] });
  assert.deepEqual(few.map(item => [item.label, item.count]), [["E_B", 9n], ["E_A", 2n]]);
  const many = codeSeries({ by_code: Array.from({ length: 10 }, (_, i) => code(`E_${i}`, String(100 - i))) });
  assert.equal(many.length, MAX_SERIES);
  assert.deepEqual(many.at(-1), { label: "Altri codici (4)", count: 94n + 93n + 92n + 91n, other: true });
  assert.equal(many.reduce((sum, item) => sum + item.count, 0n), 955n);
});

test("the charts show Rust counts with text, and warn when the controls are incomplete", () => {
  const data = { ...overview.overview, by_code: [code("E_RECORD_LENGTH", "700000"), code("E_CROSS_FIELD", "300000")],
    by_trace: [{ schemaVersion: 1, trace_ref: "trace_0", trace: { schemaVersion: 1, text: "S", complete: true }, count: "1000000" },
      { schemaVersion: 1, trace_ref: "trace_1", trace: { schemaVersion: 1, text: "T", complete: true }, count: "0" }] };
  const html = renderToStaticMarkup(createElement(Charts, { overview: data, scope: "complete" }));
  for (const title of ["Errori per tracciato", "Tipologia errori", "Distribuzione errori", "Record senza violazioni"]) {
    assert.match(html, new RegExp(`<h2>${title}</h2>`));
  }
  assert.match(html, /role="img" aria-label="Diagnostiche per tracciato\. S: 1\.000\.000; T: 0"/);
  assert.match(html, /<text class="donut-total"[^>]*>1\.000\.000<\/text>/);
  assert.match(html, /<span class="legend-label font-mono">E_RECORD_LENGTH<\/span><span class="legend-value">700\.000<\/span>/);
  assert.match(html, /<span class="hbar-label font-mono">E_CROSS_FIELD<\/span>.*?<span class="hbar-value">300\.000<\/span>/);
  assert.match(html, /<text class="validity-value"[^>]*>16,6%<\/text>/);
  assert.match(html, /<strong>200\.000<\/strong> senza violazioni su 1\.200\.000 analizzati/);
  assert.doesNotMatch(html, /Risultati incompleti|Completezza|notice-warn|\sstyle=|Warning|Avviso/);
  const partial = renderToStaticMarkup(createElement(Charts, { overview: data, scope: "incomplete" }));
  assert.match(partial, /<p class="notice-warn" role="note"><svg[^>]*aria-hidden="true"[^>]*>.*?<\/svg><span><strong>Risultati incompleti\.<\/strong> Non tutti i controlli sono stati eseguiti/);
  // The charts stay visible under the warning (author decision, 24 Nov 2026).
  assert.match(partial, /<h2>Record senza violazioni<\/h2>/);
});

test("Risultati shows the charts once the overview is read, with the warning for partial reports", () => {
  const partialAbout = { status: "ready", about: { ...about.about, completeness: "partial" } };
  assert.match(results({ view: ready({ loading: false, current: page() }) }), /<section class="charts" aria-label="Grafici del report">/);
  assert.doesNotMatch(results({ view: ready({ loading: false, current: page() }) }), /Risultati incompleti/);
  assert.match(results({ view: ready({ loading: false, current: page() }), about: partialAbout }), /Risultati incompleti/);
  // P601-01: the same counts never read as a complete analysis while
  // report_about is pending or failed; the charts stay, qualified.
  const view = ready({ loading: false, current: page() });
  for (const [pending, role, text] of [[{ status: "loading" }, "status", "Completezza in verifica"],
    [{ status: "none" }, "status", "Completezza in verifica"],
    [{ status: "failed" }, "alert", "Completezza non disponibile"]]) {
    const html = results({ view, about: pending });
    assert.match(html, /<section class="charts" aria-label="Grafici del report">/);
    assert.match(html, new RegExp(`<p class="notice-warn" role="${role}"><svg[^>]*aria-hidden="true"[^>]*>.*?</svg><span><strong>${text}\\.</strong>`));
    assert.doesNotMatch(html, /Risultati incompleti/);
  }
  assert.doesNotMatch(results({ view }), /notice-warn/);
  assert.doesNotMatch(results({ view: ready(), overview: { status: "loading" } }), /class="charts"/);
  assert.match(results({ view: ready(), overview: { status: "failed" } }), /role="alert">Non è stato possibile leggere i conteggi del report per i grafici\./);
});
