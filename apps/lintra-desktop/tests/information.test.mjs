// T-MIG-P5-07: Informazioni, provenance and licenses. Acceptance: correct
// provenance data; the Slint notice stays where the legacy executable needs
// it; the Tauri notices are correct.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readdirSync, readFileSync } from "node:fs";
import { test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { loadComponent } from "./tsx.mjs";
import { previewText, reportStatusText, splitNotices } from "../src/information.ts";
import { INITIAL_SELECTION, provenanceText } from "../src/analysis.ts";

const base = new URL("../", import.meta.url);
const repository = new URL("../../", base);
const read = (path, root = base) => readFileSync(new URL(path, root));
const sha = bytes => createHash("sha256").update(bytes).digest("hex");
const declared = JSON.parse(read("assets-provenance.json")).notices;
const notices = read("public/THIRD_PARTY_NOTICES");
const header = block => `${block.packages.map(item => `${item.name} ${item.version}`).join(", ")} / ${block.file}`;
// A Windows checkout may turn the lock into CRLF; only its content matters.
const lockVersions = name => read("src-tauri/Cargo.lock", base).toString().replace(/\r\n/g, "\n")
  .split("[[package]]").filter(entry => entry.includes(`\nname = "${name}"\n`))
  .map(entry => entry.match(/\nversion = "([^"]+)"/)[1]);

test("the notices file is exactly the declared license texts, byte for byte", () => {
  assert.equal(declared.file, "public/THIRD_PARTY_NOTICES");
  const blocks = splitNotices(notices.toString("utf8"));
  assert.deepEqual(blocks.map(block => block.title), declared.blocks.map(header));
  blocks.forEach((block, index) => assert.equal(sha(Buffer.from(block.body)), declared.blocks[index].sha256, block.title));
  // Same layout the generator wrote: header, verbatim text, blocks joined by a blank line.
  assert.deepEqual(Buffer.from(blocks.map(block => `${block.title}\n${block.body}`).join("\n\n") + "\n"), notices);
});

test("npm notices cover the whole runtime closure of the lock at the locked versions", () => {
  const lock = JSON.parse(read("package-lock.json"));
  const runtime = Object.entries(lock.packages).filter(([path, entry]) => path && !entry.dev && !entry.devOptional)
    .map(([path, entry]) => `${path.slice("node_modules/".length)} ${entry.version}`).sort();
  const npm = declared.blocks.filter(block => block.ecosystem === "npm");
  assert.deepEqual([...new Set(npm.flatMap(block => block.packages.map(item => `${item.name} ${item.version}`)))].sort(), runtime);
  for (const block of npm) {
    const [{ name }] = block.packages;
    assert.equal(sha(read(`node_modules/${name}/${block.file}`)), block.sha256, header(block));
  }
});

test("Tauri runtime crates are noticed at the Cargo.lock versions and Slint is not linked", () => {
  const cargo = declared.blocks.filter(block => block.ecosystem === "cargo").flatMap(block => block.packages);
  for (const name of ["tauri", "tauri-runtime", "tauri-runtime-wry", "tauri-utils", "wry", "tao"]) {
    const versions = lockVersions(name);
    assert.equal(versions.length, 1, name);
    assert.ok(cargo.some(item => item.name === name && item.version === versions[0]), name);
  }
  for (const item of cargo) assert.deepEqual(lockVersions(item.name), [item.version], item.name);
  // Every dual-licensed crate ships both texts; tao is Apache-2.0 only.
  for (const name of ["tauri", "wry"]) {
    assert.equal(declared.blocks.filter(block => block.packages.some(item => item.name === name)).length, 2, name);
  }
  assert.doesNotMatch(read("src-tauri/Cargo.lock").toString(), /\nname = "slint/);
});

test("the Slint attribution stays in the legacy executable and its release archive", () => {
  assert.match(read("ui/ui/main.slint", repository).toString(), /^\s*AboutSlint \{/m);
  assert.match(read("scripts/release.py", repository).toString(), /"LICENSE-SLINT-ROYALTY-FREE-2\.0\.md": \(ROOT \/ "ui\/licenses\/SLINT-ROYALTY-FREE-2\.0\.md"\)/);
  assert.ok(read("ui/licenses/SLINT-ROYALTY-FREE-2.0.md", repository).length > 0);
  // This executable links no Slint code, so it ships no Slint notice.
  assert.doesNotMatch(notices.toString(), /slint/i);
});

test("bounded previews are marked when cut and outcomes never overstate conformity", () => {
  assert.equal(previewText({ schemaVersion: 1, text: "SIOC", complete: true }), "SIOC");
  assert.equal(previewText({ schemaVersion: 1, text: "SIO", complete: false }), "SIO…");
  const outcome = (status, completeness = "complete", truncated = false) => reportStatusText({ status, completeness, truncated });
  assert.equal(outcome("completed_valid"), "Nessuna violazione nei controlli eseguiti");
  assert.equal(outcome("completed_valid", "partial"), "Nessuna violazione nei controlli eseguiti, controlli incompleti");
  assert.equal(outcome("completed_invalid", "partial"), "Violazioni rilevate, controlli incompleti");
  assert.match(outcome("completed_valid", "complete", true), /^Analisi parziale/);
  assert.match(outcome("cancelled", "partial"), /^Analisi interrotta/);
  assert.equal(provenanceText({ bundle_source: "external", bundle_version: null }), "schemi esterni, versione non dichiarata");
  // Text without a "<packages> / LICENSE…" header yields no notice at all.
  assert.deepEqual(splitNotices(""), []);
  assert.deepEqual(splitNotices("MIT License\nreact / README\n"), []);
  assert.deepEqual(splitNotices("a 1 / LICENSE\nX\n\nb 2 / LICENSE-MIT\nY\n"), [{ title: "a 1 / LICENSE", body: "X" }, { title: "b 2 / LICENSE-MIT", body: "Y" }]);
});

test("no two source modules differ only by case or extension (P507-01)", () => {
  // "./About" must resolve to one file on a case-insensitive file system
  // too: Windows, the official target, tries About.ts before About.tsx.
  const stems = readdirSync(new URL("src/", base)).filter(name => /\.(ts|tsx)$/.test(name))
    .map(name => name.replace(/\.tsx?$/, "").toLowerCase());
  assert.deepEqual(stems.filter((stem, index) => stems.indexOf(stem) !== index), []);
});

const { About } = await loadComponent("About");
const { AppShell } = await loadComponent("AppShell");
const run = { schemaVersion: 1, instance_id: "a".repeat(32), run_id: "3" };
const bundle = { schemaVersion: 1, bundle_source: "embedded", bundle_version: "2026.11", bundle_manifest_status: "present" };
const preview = (text, complete = true) => ({ schemaVersion: 1, text, complete });
const about = { schemaVersion: 1, report_ref: "p_1", run, app_version: "0.1.1-rc.3", validator_version: preview("0.1.1"),
  flow_id: preview("SYNTHETIC"), schema_version: preview("v1"), schema_hash: preview("b".repeat(256), false), mode: "local_only",
  status: "completed_invalid", completeness: "complete", truncated: false, exportable: true, trace_count: "1",
  reference_dataset_count: "0", bundle: { ...bundle, bundle_version: null, bundle_manifest_status: "absent" } };
const selected = { ...INITIAL_SELECTION, catalog: { status: "ready", entries: [], provenance: bundle },
  configuration: { status: "ready", traces: [], selection: { schemaVersion: 1, configuration_ref: "c_1", flow_id: "SYNTHETIC",
    schema_version: "v2", schema_hash: "c".repeat(64), encoding: "utf8", trace_count: "1", parameter_count: "0", provenance: bundle } } };
const list = splitNotices(notices.toString("utf8"));
const render = props => renderToStaticMarkup(createElement(About,
  { version: "0.1.1-rc.3", selection: INITIAL_SELECTION, report: { status: "none" }, notices: list, ...props }));

test("version, selection and report provenance come from Rust and stay apart", () => {
  let html = render({});
  assert.match(html, /<dt>Versione<\/dt><dd>0\.1\.1-rc\.3<\/dd>/);
  assert.match(html, /Nessuno schema selezionato\./);
  assert.match(html, /Nessun report pubblicato in questa sessione\./);
  assert.match(render({ version: null }), /<dd>Non ancora disponibile<\/dd>/);
  html = render({ selection: { ...INITIAL_SELECTION, catalog: { status: "ready", entries: [], provenance: bundle } } });
  assert.match(html, /<dt>Origine<\/dt><dd>schemi incorporati, versione 2026\.11<\/dd><dt>Manifest del bundle<\/dt><dd>presente<\/dd>/);
  html = render({ selection: selected, report: { status: "ready", about } });
  // The selection shows v2 and its full hash; the report its own v1 and cut hash.
  assert.match(html, /<dd>SYNTHETIC · v2<\/dd><dt>Impronta dello schema<\/dt><dd>c{64}<\/dd>/);
  assert.match(html, /<dd>SYNTHETIC · v1<\/dd><dt>Impronta dello schema<\/dt><dd>b{256}…<\/dd>/);
  assert.match(html, /non la selezione corrente/);
  assert.match(html, /<dt>Esito<\/dt><dd>Violazioni rilevate<\/dd><dt>Modalità<\/dt><dd>Controlli locali sul tracciato<\/dd>/);
  assert.match(html, /<dd>schemi incorporati, versione non dichiarata<\/dd><dt>Manifest del bundle<\/dt><dd>assente<\/dd>/);
  assert.match(html, /<dt>Tracciati<\/dt><dd>1<\/dd><dt>Dataset di riferimento<\/dt><dd>0<\/dd>/);
  assert.match(render({ report: { status: "loading" } }), /<p role="status">Lettura della provenienza del report…<\/p>/);
  assert.match(render({ report: { status: "failed" } }), /<p role="alert">Non è stato possibile leggere/);
});

test("license texts are closed disclosures with keyboard-scrollable verbatim text", () => {
  const html = render({});
  assert.equal((html.match(/<details>/g) ?? []).length, list.length);
  assert.doesNotMatch(html, /<details open/);
  assert.equal((html.match(/<pre tabindex="0" aria-label="Testo della licenza: [^"]+">/g) ?? []).length, list.length);
  assert.match(html, /<summary>tauri 2\.12\.5, tauri-runtime 2\.12\.2, tauri-runtime-wry 2\.12\.3, tauri-utils 2\.10\.2 \/ LICENSE-MIT<\/summary>/);
  assert.match(html, /Copyright \(c\) 2017 - Present Tauri Apps Contributors/);
  assert.equal(count(html, /<h2\b/g), 4);
  assert.doesNotMatch(html, /slint/i);
});

test("the shell renders the Informazioni body it is given", () => {
  const html = renderToStaticMarkup(createElement(AppShell, { view: "informazioni", onView: () => {}, theme: "system", onTheme: () => {},
    service: { kind: "ready", version: "0.1.1-rc.3" }, reportAvailable: false, information: createElement("p", null, "corpo") }));
  assert.match(html, /<h1 tabindex="-1">Informazioni<\/h1><p class="page-subtitle">[^<]*<\/p><\/div><\/div><p>corpo<\/p><\/main>/);
});

function count(html, pattern) { return (html.match(pattern) ?? []).length; }
