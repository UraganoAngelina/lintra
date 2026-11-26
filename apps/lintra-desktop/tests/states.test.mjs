// T-MIG-P6-07: result states. No global conformity for LocalOnly or
// incomplete data; an outcome or count Rust could not give is an accessible
// error state, never "in lettura" or a success.
import assert from "node:assert/strict";
import { test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { loadComponent } from "./tsx.mjs";
import { NO_FILTERS } from "../src/pages.ts";

globalThis.window = { crypto: globalThis.crypto };
const { Results } = await loadComponent("Results");

const RUN = { schemaVersion: 1, instance_id: "a".repeat(32), run_id: "1" };
const preview = text => ({ schemaVersion: 1, text, complete: true });
const base = { schemaVersion: 1, report_ref: "rep", run: RUN, app_version: "0.1.1-rc.3", validator_version: preview("0.1.1-rc.3"),
  flow_id: preview("SYNTHETIC"), schema_version: preview("v1"), schema_hash: preview("9".repeat(64)), mode: "local_only",
  status: "completed_valid", completeness: "complete", truncated: false, exportable: true, trace_count: "1",
  reference_dataset_count: "0", bundle: {} };
const overview = { status: "ready", overview: { schemaVersion: 1, run: RUN, records_processed: "2", valid_records: "2",
  invalid_records: "0", total_errors: "0", by_code: [], by_trace: [] } };
const empty = { status: "ready", run: RUN, query_ref: "q_1", filters: NO_FILTERS, total_matches: "0", total_diagnostics: "0",
  page: 0n, current: null, loading: false, pageFailed: false };
const noop = () => {};
const render = (about, props = {}) => renderToStaticMarkup(createElement(Results, { view: empty, about, overview, reportAvailable: true,
  filters: NO_FILTERS, onFilter: noop, onPage: noop, onRetry: noop, ...props }));
const ready = changes => ({ status: "ready", about: { ...base, ...changes } });
const esito = html => html.match(/<h2>Esito<\/h2><p class="([^"]*)">([^<]*)<\/p><p class="kpi-note"( role="alert")?>([^<]*)<\/p>/);

test("only a complete Full report without violations takes the success tone", () => {
  const full = esito(render(ready({ mode: "full", trace_count: "2" })));
  assert.equal(full[1], "kpi-value kpi-text tone-success");
  assert.equal(full[4], "SYNTHETIC · v1 · verifiche complete nell&#x27;ambito richiesto");
  // LocalOnly: same words, no success tone, the scope is said.
  const local = render(ready({}));
  assert.equal(esito(local)[1], "kpi-value kpi-text");
  assert.match(local, /<h2>Record analizzati<\/h2><p class="kpi-value">2<\/p><p class="kpi-note">Controlli locali sul tracciato<\/p>/);
  assert.match(local, /Il report non contiene diagnostiche nell&#x27;ambito richiesto: controlli locali sul tracciato\./);
  assert.doesNotMatch(local, /tone-success|conform[ei]\b|Valido/);
});

test("partial, truncated or cancelled data is never a conformity", () => {
  for (const [changes, words] of [[{ completeness: "partial" }, "Nessuna violazione nei controlli eseguiti, controlli incompleti"],
    [{ truncated: true }, "Analisi parziale: raggiunto il limite operativo"],
    [{ status: "cancelled" }, "Analisi interrotta: i controlli non sono stati completati"]]) {
    for (const mode of ["local_only", "full"]) {
      const html = render(ready({ ...changes, mode }));
      const [, tone, text, , note] = esito(html);
      assert.equal(tone, "kpi-value kpi-text tone-warn", `${JSON.stringify(changes)} ${mode}`);
      assert.equal(text, words);
      assert.equal(note, "SYNTHETIC · v1 · verifiche parziali");
      assert.match(html, /Il report non contiene diagnostiche, ma alcuni controlli non hanno un esito conclusivo: non è una conformità completa\./);
      assert.doesNotMatch(html, /tone-success|nell&#x27;ambito richiesto: controlli locali/);
    }
  }
});

test("an outcome Rust could not read is an alert, never pending or valid", () => {
  const html = render({ status: "failed" });
  const [, tone, text, alert, note] = esito(html);
  assert.equal(tone, "kpi-value kpi-text");
  assert.equal(text, "Esito non disponibile");
  assert.equal(alert, ' role="alert"');
  assert.equal(note, "Non è stato possibile leggere l&#x27;esito del report: nessun esito è mostrato.");
  assert.match(html, /<p class="kpi-note">Ambito non disponibile<\/p>/);
  assert.doesNotMatch(html, /In lettura…|tone-success|nell&#x27;ambito richiesto/);
  // The empty report claims nothing beyond the absence of diagnostics.
  assert.match(html, /<p class="grid-message">Il report non contiene diagnostiche\.<\/p>/);
  // Still being read: a pending note, no alert, no outcome.
  const loading = render({ status: "loading" });
  assert.match(loading, /<p class="kpi-note">In lettura…<\/p>/);
  assert.equal(esito(loading)[2], "—");
  assert.doesNotMatch(loading, /Esito non disponibile/);
});

test("technical errors of the report are alerts with a retry, never an empty valid page", () => {
  const failed = render(ready({ mode: "full" }), { view: { status: "failed", run: RUN } });
  assert.match(failed, /<div class="grid-message" role="alert"><p>Non è stato possibile preparare i risultati del report\.<\/p><button[^>]*>Riprova<\/button>/);
  assert.doesNotMatch(failed, /Il report non contiene diagnostiche/);
  const counts = render(ready({ mode: "full" }), { overview: { status: "failed" } });
  assert.match(counts, /<p class="grid-message" role="alert">Non è stato possibile leggere i conteggi del report per i grafici\.<\/p>/);
  assert.match(counts, /<h2>Record analizzati<\/h2><p class="kpi-value">—<\/p>/);
});
