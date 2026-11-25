// T-MIG-P6-05: «Dominio ammesso» of the selected field. Rust lists the
// declarations of the captured schema and pages their values; the renderer
// shows them as text, never computes a domain and never parses an integer.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { loadComponent } from "./tsx.mjs";

globalThis.window = { crypto: globalThis.crypto };
const { DetailPanel, DomainSourceItem, DomainValues } = await loadComponent("Results");

const S = "s_" + "2".repeat(32);
const preview = (text, complete = true) => ({ schemaVersion: 1, text, complete });
const source = (kind, extra = {}) => ({ schemaVersion: 1, source_ref: `o_${kind}`, kind, conditional: false, rule: null, condition: null,
  dataset: null, comparison: preview("Confronto sul valore logico dopo l'estrazione del riempimento dichiarata."), ...extra });
const page = (status, entries, next_cursor = null) => ({ schemaVersion: 1, kind: "range", conditional: false,
  expected_sha256: null, actual_sha256: null, status, entries, next_cursor });
const entry = (text, complete = true) => ({ schemaVersion: 1, component_ref: `v_${text.length}_${text.slice(0, 4)}`, preview: preview(text, complete) });
const item = (value, shown = null) => renderToStaticMarkup(createElement("ol", null,
  createElement(DomainSourceItem, { selectionRef: S, source: value, number: "1", shown, onRead() {} })));

test("a declaration says its kind, where it comes from and how values are compared", () => {
  const html = item(source("allowed_values"));
  assert.match(html, /<p class="domain-kind">Valori dichiarati <span class="text-muted">· dichiarazione 1 · campo<\/span><\/p>/);
  assert.match(html, /Confronto sul valore logico dopo l&#x27;estrazione del riempimento dichiarata\./);
  assert.match(html, />Consulta i valori</);
  assert.match(item(source("fallback")), /Fallback autorizzati/);
  assert.match(item(source("range")), /Intervallo dichiarato/);
});

test("a conditional declaration keeps its rule and guard, never shown as a field constraint", () => {
  const html = item(source("allowed_values", { conditional: true, rule: preview("COND-F2"),
    condition: { schemaVersion: 1, component_ref: "x_guard", preview: preview("F1 = \"1\"") } }));
  assert.match(html, /dichiarazione 1 · regola condizionale COND-F2/);
  assert.doesNotMatch(html, /· campo</);
  assert.match(html, /Si applica quando: <\/span><span class="detail-text detail-value">F1 = &quot;1&quot;<\/span>/);
});

test("values are Rust's literals; integers beyond u128 stay exact text", () => {
  const big = "340282366920938463463374607431768211456"; // u128::MAX + 1
  const html = renderToStaticMarkup(createElement(DomainValues, { selectionRef: S,
    domain: page("available", [entry(big), entry("\"␠A\""), entry("\"00\"")], "c_next") }));
  assert.match(html, new RegExp(`<li><span class="detail-text detail-value">${big}</span></li>`));
  assert.match(html, /"␠A"|&quot;␠A&quot;/);
  assert.match(html, /&quot;00&quot;/, "leading zeros kept");
  // A long literal is Rust's preview, marked, readable in full through its handle.
  const long = renderToStaticMarkup(createElement(DomainValues, { selectionRef: S, domain: page("available", [entry("9".repeat(200), false)]) }));
  assert.match(long, />Leggi il testo completo</);
});

test("pages: next values through Rust's cursor, back to the first page", () => {
  const ready = { status: "ready", source: "o_range", domain: page("available", [entry("1"), entry("2")], "c_1"), first: true };
  const first = item(source("range"), ready);
  assert.match(first, />Valori successivi</);
  assert.doesNotMatch(first, /Primi valori|Consulta i valori/);
  const last = item(source("range"), { ...ready, domain: page("available", [entry("3")]), first: false });
  assert.match(last, />Primi valori</);
  assert.doesNotMatch(last, /Valori successivi/);
});

test("no list is invented: not enumerable, empty page, failure", () => {
  const regex = renderToStaticMarkup(createElement(DomainValues, { selectionRef: S, domain: { ...page("not_enumerable", null), kind: "regex" } }));
  assert.match(regex, /Nessun elenco finito: il vincolo non è enumerabile\. Non viene inventato un elenco di valori\./);
  assert.doesNotMatch(regex, /<li>/);
  assert.match(renderToStaticMarkup(createElement(DomainValues, { selectionRef: S, domain: page("available_empty", []) })),
    /Nessun valore in questa pagina del dominio\./);
  assert.match(item(source("regex")), /Espressione regolare/);
  const failed = item(source("range"), { status: "failed", source: "o_range", cursor: "c_1" });
  assert.match(failed, /role="alert">Lettura dei valori non riuscita\./);
  assert.match(failed, />Riprova</);
});

test("a reference dataset is named, without values here (T-MIG-P6-06)", () => {
  const html = item(source("reference", { dataset: { schemaVersion: 1, id: preview("union-A"), version: preview("1") } }));
  assert.match(html, /Dataset di riferimento/);
  assert.match(html, /Dataset union-A · versione 1/);
  assert.doesNotMatch(html, /Consulta i valori/);
});

test("the panel offers the domain only for a field diagnostic", () => {
  const RUN = { schemaVersion: 1, instance_id: "a".repeat(32), run_id: "1" };
  const part = (kind, text) => ({ schemaVersion: 1, component_ref: "x_" + kind, kind, preview: preview(text) });
  const detail = position => ({ schemaVersion: 1, selection_ref: S, query_ref: "q_" + "0".repeat(32), diagnostic_ref: "d_" + "1".repeat(32),
    run: RUN, line: "1", trace_ref: "trace_0", field_position: position, actual_visibility: "available", source_count: "2",
    components: [part("code", "E_X"), part("message", "M"), part("actual", "1")] });
  const render = value => renderToStaticMarkup(createElement(DetailPanel, { traces: new Map(), onRetry() {},
    state: { status: "ready", selection: { query_ref: "q", ordinal: "0", diagnostic_ref: "d" }, detail: value } }));
  const field = render(detail({ schemaVersion: 1, start: "1", end: "3", units: "one_based_inclusive_bytes" }));
  assert.match(field, /<h3 id="detail-domain-title" tabindex="-1">Dominio ammesso<\/h3>/);
  assert.match(field, />Mostra il dominio</);
  assert.doesNotMatch(render(detail(null)), /Dominio ammesso/);
});

test("the renderer computes no domain: no integer parsing of values", () => {
  const source = readFileSync(new URL("../src/Results.tsx", import.meta.url), "utf8");
  const domain = source.slice(source.indexOf("const SOURCE_KIND"), source.indexOf("export function DetailPanel"));
  assert.doesNotMatch(domain, /parseInt|parseFloat|Number\(|BigInt\(entry|\.sort\(|\.filter\(/);
});
