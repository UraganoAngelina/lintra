// T-MIG-P6-10: Full results, detail and export in the renderer. Rust sends
// rows, trace names, field guide, position and context, each entry with its
// own trace; these tests check that a Full report is shown from those DTOs
// alone (trace names from results_overview, never decoded from trace_ref) and
// that no cross-record logic exists in the renderer sources.
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { loadComponent } from "./tsx.mjs";

globalThis.window = { crypto: globalThis.crypto };
const { ContextEntries, DetailPanel, ResultsGrid } = await loadComponent("Results");
const { ExportCard } = await loadComponent("Export");

const RUN = { schemaVersion: 1, instance_id: "a".repeat(32), run_id: "3" };
const preview = (text, complete = true) => ({ schemaVersion: 1, text, complete });
// Overview order of the scope: trace_0 = S, trace_1 = T.
const TRACES = new Map([["trace_0", preview("S")], ["trace_1", preview("T")]]);
const row = (ordinal, trace_ref, code, field, line) => ({
  schemaVersion: 1, diagnostic_ref: `d_${ordinal}`, ordinal: String(ordinal), line: String(line), trace_ref,
  code: preview(code), field: field === null ? null : preview(field), message: preview(`messaggio ${ordinal}`),
  expected: null, actual: null });
const PAGE = { schemaVersion: 1, query_ref: "q_1", run: RUN, page_generation: "1", offset: "0", next_offset: "3",
  has_more: false, total_matches: "3", total_diagnostics: "3", rows: [
    row(0, "trace_0", "E_RECORD_LENGTH", null, 3),
    row(1, "trace_0", "E_CROSS_FIELD", "Synthetic bytes", 1),
    row(2, "trace_1", "E_MISSING_PARENT_RECORD", null, 1)] };

test("Full rows carry the trace name Rust lists for their handle", () => {
  const html = renderToStaticMarkup(createElement(ResultsGrid, { page: PAGE, traces: TRACES }));
  const body = html.slice(html.indexOf("<tbody>"));
  const traces = [...body.matchAll(/<td class="col-trace">(.*?)<\/td>/g)].map(match => match[1]);
  const codes = [...body.matchAll(/font-mono font-normal">(.*?)<\/span>/g)].map(match => match[1]);
  assert.deepEqual(traces, ["S", "S", "T"]);
  assert.deepEqual(codes, ["E_RECORD_LENGTH", "E_CROSS_FIELD", "E_MISSING_PARENT_RECORD"]);
  // An unknown handle is never decoded into a name.
  const unknown = renderToStaticMarkup(createElement(ResultsGrid, { page: { ...PAGE, rows: [row(0, "trace_7", "E_X", null, 1)] }, traces: TRACES }));
  assert.doesNotMatch(unknown, /trace_7/);
});

test("a Full cross-trace detail shows Rust's guide, position and trace", () => {
  const part = (kind, text) => ({ schemaVersion: 1, component_ref: `x_${kind}`, kind, preview: preview(text) });
  const detail = { schemaVersion: 1, selection_ref: "s_1", query_ref: "q_1", diagnostic_ref: "d_1", run: RUN, line: "1",
    trace_ref: "trace_0", field_position: { schemaVersion: 1, start: "5", end: "6", units: "one_based_inclusive_bytes" },
    actual_visibility: "available", source_count: "0", components: [part("code", "E_CROSS_FIELD"),
      part("message", "Synthetic COMPARE: vero"), part("field_name", "Synthetic bytes"),
      part("guide", "Alfanumerico. Slot: byte 5–6 inclusi (2 byte)."), part("actual", "NO")] };
  const html = renderToStaticMarkup(createElement(DetailPanel, {
    state: { status: "ready", selection: { query_ref: "q_1", ordinal: "1", diagnostic_ref: "d_1" }, detail },
    traces: TRACES, onRetry() {} }));
  assert.match(html, /<dt>Tracciato<\/dt><dd>S<\/dd>/);
  assert.match(html, /Slot: byte 5–6 inclusi \(2 byte\)\./);
  assert.match(html, /Synthetic bytes/);
  assert.match(html, />Mostra il contesto</);
});

test("context entries of two traces are listed with their own trace, as Rust sent them", () => {
  const entry = (trace, field, value, ref) => ({ schemaVersion: 1, trace: preview(trace), field: preview(field),
    component_ref: ref, value: value === null ? null : preview(value), visibility: value === null ? "local_report_only_record_scope" : "available" });
  const page = { schemaVersion: 1, selection_ref: "s_1", offset: "0", next_offset: "3", has_more: false,
    entries: [entry("S", "DATA", "NO", null), entry("T", "DATA", "OK", null), entry("T", "RECORD", null, null)] };
  const html = renderToStaticMarkup(createElement(ContextEntries, { selectionRef: "s_1", page }));
  const pairs = [...html.matchAll(/<dt>(.*?)<\/dt><dd>(.*?)<\/dd>/g)].map(match => [match[1], match[2]]);
  assert.deepEqual(pairs, [
    ["S · DATA", '<span class="detail-text detail-value">NO</span>'],
    ["T · DATA", '<span class="detail-text detail-value">OK</span>'],
    ["T · RECORD", '<span class="text-muted">Solo nel report locale.</span>']]);
  const empty = renderToStaticMarkup(createElement(ContextEntries, { selectionRef: "s_1", page: { ...page, entries: [] } }));
  assert.match(empty, /Nessun elemento di contesto\./);
});

test("a Full report is saved whole, like a LocalOnly one", () => {
  const about = { schemaVersion: 1, report_ref: "rep_" + "1".repeat(32), run: RUN, status: "completed_invalid", mode: "full",
    completeness: "complete", truncated: false, exportable: true, trace_count: "2" };
  const html = renderToStaticMarkup(createElement(ExportCard, { about, initial: "xlsx", view: { status: "idle" },
    onSave() {}, onCancel() {}, onCheck() {} }));
  assert.match(html, /Il salvataggio include report e riepilogo JSON/);
  assert.match(html, /checked="" value="xlsx"/);
  assert.match(html, />Salva report…</);
  assert.doesNotMatch(html, /non si può salvare/);
});

test("no renderer source joins records, resolves relations or reads the schema", () => {
  const dir = new URL("../src/", import.meta.url);
  for (const name of readdirSync(dir).filter(file => /\.(ts|tsx)$/.test(file))) {
    const source = readFileSync(new URL(name, dir), "utf8");
    // Schema concepts of PASS2 (joins, keys, cardinality, related traces)
    // exist only in Rust; the renderer has neither the data nor the rules.
    assert.doesNotMatch(source, /target_trace|source_fields|target_fields|record_key|cardinality|on_missing_record|join_parameter/i, name);
  }
});
