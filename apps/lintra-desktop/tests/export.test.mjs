// T-MIG-P6-08: «Salva il report». The renderer sends only report_ref and a
// format; Rust owns dialog, confirmations, writers and publication. The
// receipt is shown as Rust gives it: two separate publications, never an
// atomic pair; partial, cancelled or unknown outcomes are said so.
import assert from "node:assert/strict";
import { afterEach, mock, test } from "node:test";
import { readFileSync } from "node:fs";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { loadComponent } from "./tsx.mjs";
import * as ipc from "../src/ipc.ts";

globalThis.window = { crypto: globalThis.crypto };
afterEach(() => { clearMocks(); mock.timers.reset(); });
const { ExportCard, receiptText, startExport, followExport, cancelExport, exportStore, EXPORT_POLL_MS } = await loadComponent("Export");

const RUN = { schemaVersion: 1, instance_id: "a".repeat(32), run_id: "7" };
const REPORT = "rep_" + "1".repeat(32);
const EXPORT = "exp_" + "2".repeat(32);
const counts = (extra = {}) => ({ schemaVersion: 1, local_files: "2", remote_files: "0", undetermined_files: "0", warning_acknowledged: false, ...extra });
const ready = { schemaVersion: 1, state: "ready", export_ref: EXPORT, report_ref: REPORT, run: RUN, format: "jsonl", destination: counts() };
const writing = { schemaVersion: 1, export_ref: EXPORT, state: "writing" };
const terminal = (state, extra = {}) => ({ schemaVersion: 1, export_ref: EXPORT, report_ref: REPORT, run: RUN, format: "jsonl", state,
  published_roles: state === "succeeded" ? ["report", "summary"] : [], cleanup_failed: false, durability_confirmed: state === "succeeded",
  destination: counts(), error: state === "failed" ? { schemaVersion: 1, category: "technical", code: "IPC_TECHNICAL_FAILURE" } : null, ...extra });
const api = { exportPrepare: ipc.exportPrepare, exportExecute: ipc.exportExecute, exportState: ipc.exportState, exportCancel: ipc.exportCancel };
const settle = () => new Promise(resolve => setImmediate(resolve));
async function drain() { for (let i = 0; i < 10; i++) await settle(); }

/** Answers operations from queues ("fail" rejects), recording the requests. */
function bridge(queues) {
  const seen = [];
  mockIPC((_command, bytes) => {
    const request = JSON.parse(new TextDecoder().decode(bytes));
    seen.push(request);
    const payload = queues[request.operation]?.shift();
    if (payload === undefined) throw new Error(`unexpected ${request.operation}`);
    const reply = payload === "fail"
      ? { schemaVersion: 1, request_id: request.request_id, kind: "error", error: { schemaVersion: 1, category: "technical", code: "IPC_TECHNICAL_FAILURE" } }
      : { schemaVersion: 1, request_id: request.request_id, kind: "result", payload };
    return new TextEncoder().encode(JSON.stringify(reply)).buffer;
  });
  return seen;
}
const views = () => { const list = []; return [list, view => list.push(view)]; };

test("save: native prepare, owned job, polled to the receipt; only handles and a format leave the renderer", async () => {
  mock.timers.enable({ apis: ["setTimeout"] });
  const seen = bridge({ export_prepare: [ready], export_execute: [writing], export_state: [writing, terminal("succeeded")] });
  const [shown, set] = views();
  startExport(api, REPORT, "jsonl", () => true, set);
  for (let i = 0; i < 3; i++) { await drain(); mock.timers.tick(EXPORT_POLL_MS); }
  await drain();
  assert.deepEqual(seen.map(r => [r.operation, r.payload]), [
    ["export_prepare", { schemaVersion: 1, report_ref: REPORT, format: "jsonl" }],
    ["export_execute", { schemaVersion: 1, export_ref: EXPORT }],
    ["export_state", { schemaVersion: 1, export_ref: EXPORT }],
    ["export_state", { schemaVersion: 1, export_ref: EXPORT }]]);
  assert.deepEqual(shown.map(v => v.status), ["choosing", "writing", "done"]);
  assert.equal(shown.at(-1).result.state, "succeeded");
  // No path, file name, overwrite or consent ever travels from here.
  assert.doesNotMatch(JSON.stringify(seen), /path|file|overwrite|confirm|shared/i);
});

test("dialog closed, confirmation declined, destination refused or prepare failed: no job is started", async () => {
  for (const [reply, status] of [[{ schemaVersion: 1, state: "cancelled", reason: "native_selection_cancelled" }, "closed"],
    [{ schemaVersion: 1, state: "cancelled", reason: "native_confirmation_declined" }, "closed"],
    [{ schemaVersion: 1, state: "rejected", reason: "destination_not_allowed" }, "rejected"], ["fail", "prepare_failed"]]) {
    const seen = bridge({ export_prepare: [reply] });
    const [shown, set] = views();
    startExport(api, REPORT, "xlsx", () => true, set);
    await drain();
    assert.deepEqual(seen.map(r => r.operation), ["export_prepare"], status);
    assert.equal(shown.at(-1).status, status);
    clearMocks();
  }
});

test("a lost reply leaves the outcome unknown, checked again on request; a stale export stops", async () => {
  bridge({ export_prepare: [ready], export_execute: ["fail"] });
  let [shown, set] = views();
  startExport(api, REPORT, "jsonl", () => true, set);
  await drain();
  assert.deepEqual(shown.at(-1), { status: "unknown", exportRef: EXPORT });
  clearMocks();
  // "Controlla l'esito": the receipt Rust kept, read once more.
  bridge({ export_state: [terminal("failed", { published_roles: ["report"] })] });
  [shown, set] = views();
  followExport(api, EXPORT, () => true, set);
  await drain();
  assert.deepEqual(shown.at(-1).result.published_roles, ["report"]);
  clearMocks();
  // A new report published meanwhile: no further read, no view update.
  mock.timers.enable({ apis: ["setTimeout"] });
  const seen = bridge({ export_state: [writing, terminal("succeeded")] });
  let live = true;
  [shown, set] = views();
  followExport(api, EXPORT, () => live, set);
  await drain();
  live = false;
  mock.timers.tick(EXPORT_POLL_MS);
  await drain();
  assert.equal(seen.length, 1);
  assert.deepEqual(shown, []);
});

test("cancel is a request: the receipt decides; too late or failed is said", async () => {
  for (const [reply, expected] of [[{ schemaVersion: 1, export_ref: EXPORT, disposition: "requested" }, "requested"],
    [{ schemaVersion: 1, export_ref: EXPORT, disposition: "already_terminal" }, "none"], ["fail", "failed"]]) {
    const seen = bridge({ export_cancel: [reply] });
    let view = { status: "writing", exportRef: EXPORT, cancel: "none" };
    cancelExport(api, EXPORT, () => true, change => { view = change(view); });
    await drain();
    assert.deepEqual(seen.map(r => [r.operation, r.payload]), [["export_cancel", { schemaVersion: 1, export_ref: EXPORT }]]);
    assert.equal(view.cancel, expected);
    clearMocks();
  }
});

test("receipt: two separate publications, never an atomic pair; partial outcomes stay explicit", () => {
  const ok = receiptText(ipcResult(terminal("succeeded")));
  assert.equal(ok.tone, "status");
  assert.deepEqual(ok.lines, ["Report e riepilogo salvati in formato JSONL.", "Sono due file pubblicati uno dopo l'altro, non un'unica scrittura."]);
  assert.doesNotMatch(ok.lines.join(" "), /atomic|insieme|transazion/i);
  const remote = receiptText(ipcResult(terminal("succeeded", { destination: counts({ local_files: "0", remote_files: "2", warning_acknowledged: true }) })));
  assert.match(remote.lines.at(-1), /Destinazione remota o non determinata confermata/);
  const lines = extra => receiptText(ipcResult(terminal(extra.state ?? "failed", extra))).lines;
  assert.deepEqual(lines({}), ["Salvataggio non riuscito.", "Nessun file è stato pubblicato."]);
  assert.deepEqual(lines({ published_roles: ["report"] }), ["Salvataggio non riuscito.",
    "È stato salvato solo il report: l'esportazione non è completa. Controlla la destinazione.",
    "La scrittura su disco dei file pubblicati non è confermata."]);
  assert.deepEqual(lines({ state: "cancelled", published_roles: ["summary"], durability_confirmed: true }),
    ["Salvataggio annullato.", "È stato salvato solo il riepilogo: l'esportazione non è completa. Controlla la destinazione."]);
  assert.deepEqual(lines({ state: "cancelled" }), ["Salvataggio annullato prima della pubblicazione dei file.", "Nessun file è stato pubblicato."]);
  // Both renamed, sync failed: published, but not a success.
  assert.deepEqual(lines({ published_roles: ["report", "summary"], cleanup_failed: true }), ["Salvataggio non riuscito.",
    "Report e riepilogo sono stati pubblicati, ma l'esportazione non è completa. Controlla la destinazione.",
    "La scrittura su disco dei file pubblicati non è confermata.",
    "Non è stato possibile eliminare tutti i file temporanei creati da questo salvataggio."]);
  assert.equal(receiptText(ipcResult(terminal("failed"))).tone, "alert");
});

/** A terminal DTO as the real client decodes it. */
function ipcResult(dto) { return { ...dto, published_roles: [...dto.published_roles] }; }

const preview = text => ({ schemaVersion: 1, text, complete: true });
const about = (changes = {}) => ({ schemaVersion: 1, report_ref: REPORT, run: RUN, app_version: "0.1.1-rc.3", validator_version: preview("0.1.1-rc.3"),
  flow_id: preview("SYNTHETIC"), schema_version: preview("v1"), schema_hash: preview("9".repeat(64)), mode: "local_only", status: "completed_invalid",
  completeness: "complete", truncated: false, exportable: true, trace_count: "1", reference_dataset_count: "0", bundle: {}, ...changes });
const card = (props = {}) => renderToStaticMarkup(createElement(ExportCard, { about: about(), initial: "xlsx", view: { status: "idle" },
  onSave: () => {}, onCancel: () => {}, onCheck: () => {}, ...props }));

test("the card: whole report, proposed format, states with accessible roles", () => {
  const idle = card();
  assert.match(idle, /<h2 id="export-title">Salva il report<\/h2>/);
  assert.match(idle, /I filtri della tabella non limitano l&#x27;esportazione\./);
  assert.match(idle, /<legend>Formato<\/legend>/);
  assert.match(idle, /checked="" value="xlsx"/);
  assert.match(idle, />Salva report…<\/button>/);
  assert.match(card({ initial: "csv" }), /checked="" value="csv"/);
  // Not exportable: the reason, no button.
  for (const [changes, reason] of [[{ status: "cancelled", exportable: false }, /analisi interrotta non si può salvare/],
    [{ truncated: true, exportable: false }, /raggiunto il limite operativo non si può salvare/]]) {
    const html = card({ about: about(changes) });
    assert.match(html, reason);
    assert.match(html, /role="note"/);
    assert.doesNotMatch(html, /Salva report…|<fieldset/);
  }
  // Partial alone stays exportable (EXPORT.md).
  assert.match(card({ about: about({ completeness: "partial" }) }), /Salva report…/);
  const choosing = card({ view: { status: "choosing" } });
  assert.match(choosing, /<fieldset class="export-formats" disabled="">/);
  assert.match(choosing, /aria-disabled="true"[^>]*>Salva report…/);
  assert.match(choosing, /role="status">Scegli la destinazione nella finestra di salvataggio…/);
  const running = card({ view: { status: "writing", exportRef: EXPORT, cancel: "none" } });
  assert.match(running, />Annulla salvataggio<\/button>/);
  assert.match(running, /role="status">Salvataggio in corso\. Attendi la conferma prima di usare i file\./);
  assert.match(card({ view: { status: "writing", exportRef: EXPORT, cancel: "requested" } }), /aria-disabled="true"[^>]*>Annulla salvataggio[\s\S]*Annullamento richiesto/);
  assert.match(card({ view: { status: "closed", reason: "native_selection_cancelled" } }), /role="status">Salvataggio annullato: nessun file scritto\./);
  assert.match(card({ view: { status: "closed", reason: "native_confirmation_declined" } }), /role="status">Conferma non data: nessun file scritto\./);
  assert.match(card({ view: { status: "rejected" } }), /role="alert">Destinazione non consentita: nessun file scritto\./);
  assert.match(card({ view: { status: "prepare_failed" } }), /role="alert">Non è stato possibile preparare il salvataggio: nessun file scritto\./);
  const unknown = card({ view: { status: "unknown", exportRef: EXPORT } });
  assert.match(unknown, /role="alert"><p>Esito del salvataggio non disponibile: alcuni file potrebbero essere già stati salvati\./);
  assert.match(unknown, />Controlla l&#x27;esito<\/button>/);
  // P608-02: an uncertain export blocks a new save, also in the markup.
  assert.match(unknown, /<fieldset class="export-formats" disabled="">/);
  assert.match(unknown, /aria-disabled="true"[^>]*>Salva report…/);
  // P608-01: an export started from an earlier report says so, and stays
  // reachable even when the shown report cannot be saved.
  const earlier = card({ previous: true, about: about({ status: "cancelled", exportable: false }), view: { status: "writing", exportRef: EXPORT, cancel: "none" } });
  assert.match(earlier, /role="note">Questo salvataggio è stato avviato dal report di un&#x27;analisi precedente/);
  assert.match(earlier, />Annulla salvataggio<\/button>[\s\S]*Salvataggio in corso/);
  assert.doesNotMatch(card({ previous: true }), /analisi precedente/);
  assert.match(card({ view: { status: "done", result: terminal("succeeded") } }), /role="status"><p>Report e riepilogo salvati in formato JSONL\.<\/p><p>Sono due file/);
  assert.match(card({ view: { status: "done", result: terminal("failed") } }), /role="alert"><p>Salvataggio non riuscito\.<\/p>/);
});

const OTHER = "rep_" + "3".repeat(32);
test("P608-01: a new report keeps the running export, its controls and its receipt", async () => {
  mock.timers.enable({ apis: ["setTimeout"] });
  const seen = bridge({ export_prepare: [ready], export_execute: [writing], export_state: [writing, writing, terminal("succeeded")], export_cancel: [{ schemaVersion: 1, export_ref: EXPORT, disposition: "requested" }] });
  const store = exportStore(api);
  store.report("A");
  store.save("A", REPORT, "jsonl");
  await drain();
  assert.equal(store.get().view.status, "writing");
  store.report("B");
  assert.deepEqual(store.get(), { owner: "A", view: { status: "writing", exportRef: EXPORT, cancel: "none" } });
  store.save("B", OTHER, "csv");
  store.cancel();
  await drain();
  assert.equal(store.get().view.cancel, "requested");
  for (let i = 0; i < 3; i++) { mock.timers.tick(EXPORT_POLL_MS); await drain(); }
  assert.equal(store.get().owner, "A");
  assert.equal(store.get().view.status, "done");
  assert.equal(store.get().view.result.export_ref, EXPORT);
  // No second prepare while A was writing; the cancel went to A.
  assert.deepEqual(seen.map(r => r.operation), ["export_prepare", "export_execute", "export_state", "export_cancel", "export_state", "export_state"]);
  assert.equal(seen.find(r => r.operation === "export_cancel").payload.export_ref, EXPORT);
  // A settled outcome is cleared by the next report, not by the same one.
  store.report("B");
  assert.equal(store.get().view.status, "done");
  store.report("C");
  assert.deepEqual(store.get(), { owner: "C", view: { status: "idle" } });
});

test("P608-02: a new save never replaces an unknown outcome; the receipt is still read", async () => {
  mock.timers.enable({ apis: ["setTimeout"] });
  const seen = bridge({ export_prepare: [ready], export_execute: [writing], export_state: ["fail", terminal("succeeded")] });
  const store = exportStore(api);
  store.report("A");
  store.save("A", REPORT, "jsonl");
  await drain();
  assert.deepEqual(store.get().view, { status: "unknown", exportRef: EXPORT });
  store.save("A", REPORT, "jsonl");
  store.report("B");
  store.save("B", OTHER, "jsonl");
  await drain();
  assert.deepEqual(store.get(), { owner: "A", view: { status: "unknown", exportRef: EXPORT } });
  store.check();
  await drain();
  assert.equal(store.get().view.status, "done");
  assert.equal(store.get().view.result.state, "succeeded");
  assert.deepEqual(seen.map(r => r.operation), ["export_prepare", "export_execute", "export_state", "export_state"]);
});

test("store: a settled outcome allows the next save; every view is published", async () => {
  const seen = bridge({ export_prepare: ["fail", { schemaVersion: 1, state: "cancelled", reason: "native_selection_cancelled" }] });
  const store = exportStore(api);
  const shown = [];
  store.subscribe(() => shown.push(store.get().view.status));
  store.save("A", REPORT, "jsonl");
  await drain();
  assert.equal(store.get().view.status, "prepare_failed");
  store.save("A", REPORT, "jsonl");
  await drain();
  assert.deepEqual(store.get().view, { status: "closed", reason: "native_selection_cancelled" });
  assert.deepEqual(shown, ["choosing", "choosing", "prepare_failed", "choosing", "choosing", "closed"]);
  assert.equal(seen.length, 2);
});

test("no report bytes, writer or file API in the renderer export", () => {
  const source = readFileSync(new URL("../src/Export.tsx", import.meta.url), "utf8");
  assert.doesNotMatch(source, /Blob|createObjectURL|download|FileReader|showSaveFilePicker|writeFile|JSON\.stringify|JSONL|csv\(/);
});
