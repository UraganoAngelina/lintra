// T-MIG-P6-04: field guide and byte position in «Dettaglio controllo».
// Rust writes the guide from the captured compiled schema
// (field_format_guide) and the position from the diagnostic; the renderer
// shows both as they are. The texts below are the ones Rust produces for the
// embedded synthetic schema, pinned on the Rust side by
// field_guides_and_positions_of_the_synthetic_schema_are_pinned (results/detail.rs).
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { loadComponent } from "./tsx.mjs";

globalThis.window = { crypto: globalThis.crypto };
const { DetailPanel } = await loadComponent("Results");

const RUN = { schemaVersion: 1, instance_id: "a".repeat(32), run_id: "1" };
const preview = (text, complete = true) => ({ schemaVersion: 1, text, complete });
const part = (kind, text, complete = true) => ({ schemaVersion: 1, component_ref: "x_" + kind, kind, preview: preview(text, complete) });
const SELECTION = { query_ref: "q_" + "0".repeat(32), ordinal: "0", diagnostic_ref: "d_" + "1".repeat(32) };
const GUIDE_F1 = "Numerico: cifre ASCII da 0 a 9, senza segno né separatori. Slot: byte 1–3 inclusi (3 byte). Contenuto a destra; posizioni residue a sinistra riempite con \"0\" (0x30); il valore logico esclude il riempimento iniziale; uno slot composto soltanto da fill produce un valore logico vuoto. Codifica: UTF-8.";
const GUIDE_F2 = "Alfanumerico: testo nella codifica dello schema; le cifre non sono vietate dal solo tipo AN. Restano applicabili i vincoli lessicali e di dominio dichiarati. Slot: byte 4–6 inclusi (3 byte). Contenuto a sinistra; posizioni residue a destra riempite con \"␠\" (0x20); il valore logico esclude il riempimento finale; uno slot composto soltanto da fill produce un valore logico vuoto. Codifica: UTF-8.";

/** The 256-byte UTF-8 preview Rust sends for a longer text (char boundary). */
function cut(text) {
  const bytes = new TextEncoder().encode(text);
  if (bytes.length <= 256) return preview(text);
  let end = 256;
  while ((bytes[end] & 0xc0) === 0x80) end -= 1;
  return preview(new TextDecoder().decode(bytes.slice(0, end)), false);
}
const detail = (name, guide, position) => ({ schemaVersion: 1, selection_ref: "s_" + "2".repeat(32), query_ref: SELECTION.query_ref,
  diagnostic_ref: SELECTION.diagnostic_ref, run: RUN, line: "1", trace_ref: "trace_0", field_position: position,
  actual_visibility: "available", source_count: "1",
  components: [part("code", "E_ALLOWED_VALUES"), part("message", "Valore non ammesso"), part("field_name", name),
    { ...part("guide", ""), preview: cut(guide) }, part("actual", "2")] });
const render = value => renderToStaticMarkup(createElement(DetailPanel, {
  state: { status: "ready", selection: SELECTION, detail: value }, traces: new Map([["trace_0", preview("T1")]]), onRetry() {} }));
const section = html => html.match(/<section class="detail-guide"[\s\S]*?<\/section>/)?.[0];
const position = (start, end) => ({ schemaVersion: 1, start, end, units: "one_based_inclusive_bytes" });

test("snapshot: a guide within the preview is shown whole, with the byte position", () => {
  // A short guide (a date field) fits the 256-byte preview: it is complete.
  const short = "Data: GGMMAAAA, con giorno, mese e anno validi. Slot: byte 7–14 inclusi (8 byte).";
  const html = render(detail("Data sintetica", short, position("7", "14")));
  assert.match(html, /<div><dt>Posizione in byte<\/dt><dd>7–14<\/dd><\/div>/);
  assert.equal(section(html),
    '<section class="detail-guide" aria-labelledby="detail-guide-title"><h3 id="detail-guide-title">Guida al campo</h3>'
    + `<span class="detail-text">${short}</span></section>`);
});

test("snapshot: the synthetic guides are Rust's preview, marked, with the full text on request", () => {
  for (const [name, guide, start, end] of [["Synthetic numeric code", GUIDE_F1, "1", "3"], ["Synthetic text code", GUIDE_F2, "4", "6"]]) {
    const html = render(detail(name, guide, position(start, end)));
    const shown = cut(guide);
    assert.equal(shown.complete, false, `${name}: longer than the 256-byte preview`);
    assert.match(html, new RegExp(`<dt>Posizione in byte</dt><dd>${start}–${end}</dd>`));
    const box = section(html);
    assert.ok(box?.startsWith('<section class="detail-guide" aria-labelledby="detail-guide-title"><h3 id="detail-guide-title">Guida al campo</h3><span class="detail-full">'), name);
    assert.ok(box.includes(`<span class="detail-text">${shown.text.replaceAll('"', "&quot;")}<span class="text-muted">…</span></span>`), name);
    assert.match(box, />Leggi il testo completo</);
  }
});

test("positions are Rust's decimal text, grouped, never recomputed", () => {
  const html = render(detail("Campo lontano", GUIDE_F1, position("18446744073709551614", "18446744073709551615")));
  assert.match(html, /<dd>18\.446\.744\.073\.709\.551\.614–18\.446\.744\.073\.709\.551\.615<\/dd>/);
  assert.doesNotMatch(html, /\(\d+ byte\)<\/dd>/, "no length computed in the renderer");
});

test("a diagnostic without a field has no position and no guide", () => {
  const html = renderToStaticMarkup(createElement(DetailPanel, { traces: new Map(), onRetry() {}, state: { status: "ready", selection: SELECTION,
    detail: { ...detail("x", GUIDE_F1, null), components: [part("code", "E_RECORD_LENGTH"), part("message", "Lunghezza"), part("actual", "4")] } } }));
  assert.match(html, /<dt>Posizione in byte<\/dt><dd><span class="text-muted">—<\/span><\/dd>/);
  assert.doesNotMatch(html, /Guida al campo/);
});

test("the renderer does not rebuild guides: no schema terms in its sources", () => {
  // The guide vocabulary of field_format_guide lives in Rust only.
  const source = readFileSync(new URL("../src/Results.tsx", import.meta.url), "utf8");
  for (const word of ["Numerico", "Alfanumerico", "riempimento", "Slot", "fill_byte", "left_pad", "right_pad"]) assert.ok(!source.includes(word), word);
});
