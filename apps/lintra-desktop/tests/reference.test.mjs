// T-MIG-P6-06: reference datasets in «Dominio ammesso». Rust reads the
// dataset on a worker from the evidence captured with the run; the renderer
// polls the read, shows the digests Rust took from that evidence and lists
// values only for verified evidence. Missing, degraded, unavailable,
// unsupported or cancelled evidence is said so: never an empty valid domain.
import assert from "node:assert/strict";
import { afterEach, mock, test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { loadComponent } from "./tsx.mjs";

globalThis.window = { crypto: globalThis.crypto };
afterEach(() => { clearMocks(); mock.timers.reset(); });
const { DomainValues, readDomain } = await loadComponent("Results");

const S = "s_" + "2".repeat(32);
const SHA = "b".repeat(64);
const preview = text => ({ schemaVersion: 1, text, complete: true });
const domain = (status, entries = null, extra = {}) => ({ schemaVersion: 1, kind: "reference", conditional: false,
  expected_sha256: SHA, actual_sha256: SHA, status, entries, next_cursor: null, ...extra });
const entry = text => ({ schemaVersion: 1, component_ref: `v_${text.replace(/\W/g, "")}`, preview: preview(text) });
const render = (value, first = true) => renderToStaticMarkup(createElement(DomainValues, { selectionRef: S, domain: value, first }));
const answer = (state, value = null) => ({ schemaVersion: 1, selection_ref: S, source_ref: "o_1", state,
  read_ref: state === "loading" ? "r_1" : null, domain: value });
const settle = () => new Promise(resolve => setImmediate(resolve));
async function drain() { for (let i = 0; i < 10; i++) await settle(); }

/** Answers operations from queues of payloads, recording the requests. */
function ipc(queues) {
  const seen = [];
  mockIPC((_command, bytes) => {
    const request = JSON.parse(new TextDecoder().decode(bytes));
    seen.push(request);
    const payload = queues[request.operation]?.shift();
    if (payload === undefined) throw new Error(`unexpected ${request.operation}`);
    return new TextEncoder().encode(JSON.stringify({ schemaVersion: 1, request_id: request.request_id, kind: "result", payload })).buffer;
  });
  return seen;
}

test("a reference read is polled until Rust publishes the page", async () => {
  mock.timers.enable({ apis: ["setTimeout"] });
  const ready = domain("available", [entry("\"AA\""), entry("\"ZZ\"")]);
  const seen = ipc({ results_domain: [answer("loading")], results_domain_state: [answer("loading"), answer("ready", ready)] });
  const pages = [];
  readDomain(S, "o_1", null, () => true, page => pages.push(page), () => pages.push("failed"));
  for (let i = 0; i < 3; i++) { await drain(); mock.timers.tick(200); }
  await drain();
  assert.deepEqual(seen.map(r => [r.operation, r.payload]), [
    ["results_domain", { schemaVersion: 1, selection_ref: S, source_ref: "o_1", cursor_ref: null }],
    ["results_domain_state", { schemaVersion: 1, selection_ref: S, read_ref: "r_1" }],
    ["results_domain_state", { schemaVersion: 1, selection_ref: S, read_ref: "r_1" }]]);
  assert.equal(pages.length, 1);
  assert.equal(pages[0].status, "available");
});

test("a read left behind stops polling and never reaches the view", async () => {
  mock.timers.enable({ apis: ["setTimeout"] });
  const seen = ipc({ results_domain: [answer("loading")], results_domain_state: [answer("ready", domain("available", [entry("\"AA\"")]))] });
  let live = true;
  const pages = [];
  readDomain(S, "o_1", null, () => live, page => pages.push(page), () => pages.push("failed"));
  await drain();
  live = false;
  mock.timers.tick(200);
  await drain();
  assert.deepEqual(seen.map(r => r.operation), ["results_domain"], "no poll after the switch");
  assert.deepEqual(pages, []);
  // A failure is reported while live, dropped once left behind.
  ipc({});
  const failed = [];
  readDomain(S, "o_1", null, () => true, () => failed.push("page"), () => failed.push("failed"));
  await drain();
  assert.deepEqual(failed, ["failed"]);
});

test("verified evidence: Rust's digest, then the values", () => {
  const html = render(domain("available", [entry("\"AA\""), entry("\"␠Z\"")]));
  assert.match(html, new RegExp(`Dataset verificato usato nell&#x27;analisi · SHA-256 <span class="font-mono">${SHA}</span>`));
  assert.match(html, /&quot;AA&quot;/);
  assert.match(html, /&quot;␠Z&quot;/);
  assert.match(render(domain("available_empty", [])), /Il dataset verificato non contiene codici\./);
  assert.match(render(domain("available_empty", []), false), /Nessun valore in questa pagina del dominio\./, "end of a longer dataset");
});

test("missing or unverified evidence is never an empty valid domain", () => {
  const cases = {
    not_captured: [/Dataset non presente nell&#x27;evidenza di questa analisi/, { expected_sha256: null, actual_sha256: null }],
    degraded: [/Integrità non verificata: i byte osservati non costituiscono un dominio ammesso/, { actual_sha256: "c".repeat(64) }],
    unavailable: [/Sorgente non disponibile durante l&#x27;analisi/, { actual_sha256: null }],
    unsupported: [/Il provider del dataset non espone l&#x27;enumerazione: nessun elenco vuoto è dichiarato valido\./, {}],
    cancelled: [/Lettura annullata: nessun elenco è dichiarato\./, {}],
  };
  for (const [status, [reason, digests]] of Object.entries(cases)) {
    const html = render(domain(status, null, digests));
    assert.match(html, reason, status);
    assert.match(html, /role="note"/, status);
    assert.doesNotMatch(html, /<li>|Dataset verificato|non contiene codici/, status);
  }
  // Digests shown as Rust sent them: expected and observed, never "verified".
  const degraded = render(domain("degraded", null, { actual_sha256: "c".repeat(64) }));
  assert.match(degraded, new RegExp(`SHA-256 atteso <span class="font-mono">${SHA}</span> · osservato <span class="font-mono">${"c".repeat(64)}</span>`));
  assert.match(render(domain("unavailable", null, { actual_sha256: null })), /osservato <span class="font-mono">non disponibile<\/span>/);
  assert.doesNotMatch(render(domain("not_captured", null, { expected_sha256: null, actual_sha256: null })), /SHA-256/);
});
