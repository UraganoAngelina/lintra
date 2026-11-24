// T-MIG-P5-04: requested report formats. Acceptance: user parity with the
// Slint ReportSelection (ui/src/analysis.rs, ui/src/export.rs,
// ui/tests/bootstrap.rs) and no writer or conversion in JavaScript.
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { loadComponent } from "./tsx.mjs";
import { DEFAULT_REPORTS, INITIAL_SELECTION, REPORT_FORMATS, preferredFormat, reduce } from "../src/analysis.ts";

const provenance = { schemaVersion: 1, bundle_source: "embedded", bundle_version: "synthetic-v1", bundle_manifest_status: "present" };
const catalog = [{ schemaVersion: 1, schema_ref: "s_v1", flow_id: "SYNTHETIC", schema_version: "v1", trace_count: "1" }];
const configuration = { schemaVersion: 1, configuration_ref: "config_1", flow_id: "SYNTHETIC", schema_version: "v1", schema_hash: "a".repeat(64),
  encoding: "utf8", trace_count: "1", parameter_count: "0", provenance };
const traces = [{ trace_ref: "config_1_trace_0", trace_id: "T1" }];
const draft = { schemaVersion: 1, plan_ref: "plan_1", configuration_ref: "config_1", mode: "local_only", assigned_input_count: "0" };

test("defaults, order and labels match the Slint form", () => {
  // ReportSelection::default(): jsonl and xlsx requested, csv not.
  assert.deepEqual(INITIAL_SELECTION.reports, { xlsx: true, jsonl: true, csv: false });
  assert.equal(INITIAL_SELECTION.reports, DEFAULT_REPORTS);
  assert.deepEqual(REPORT_FORMATS.map(format => format.label), ["XLSX", "JSONL", "CSV"]);
});

test("the proposed format follows Slint preferred_format for every combination", () => {
  for (const xlsx of [false, true]) for (const jsonl of [false, true]) for (const csv of [false, true]) {
    const expected = xlsx ? "xlsx" : csv ? "csv" : "jsonl";
    assert.equal(preferredFormat({ xlsx, jsonl, csv }), expected, JSON.stringify({ xlsx, jsonl, csv }));
  }
  assert.equal(preferredFormat(DEFAULT_REPORTS), "xlsx");
});

test("each format toggles alone, any combination is allowed and unknown formats are ignored", () => {
  let state = reduce(INITIAL_SELECTION, { type: "set_report", format: "xlsx", requested: false });
  assert.deepEqual(state.reports, { xlsx: false, jsonl: true, csv: false });
  state = reduce(state, { type: "set_report", format: "csv", requested: true });
  assert.deepEqual(state.reports, { xlsx: false, jsonl: true, csv: true });
  state = reduce(state, { type: "set_report", format: "jsonl", requested: false });
  state = reduce(state, { type: "set_report", format: "csv", requested: false });
  assert.deepEqual(state.reports, { xlsx: false, jsonl: false, csv: false }, "nothing requested is a valid choice, as in Slint");
  assert.equal(preferredFormat(state.reports), "jsonl");
  assert.equal(reduce(state, { type: "set_report", format: "pdf", requested: true }), state);
  assert.equal(reduce(state, { type: "set_report", format: "__proto__", requested: true }), state);
});

test("the choice survives catalog, flow, version, trace and plan changes", () => {
  const requested = { xlsx: false, jsonl: false, csv: true };
  let state = reduce(INITIAL_SELECTION, { type: "set_report", format: "xlsx", requested: false });
  state = reduce(state, { type: "set_report", format: "jsonl", requested: false });
  state = reduce(state, { type: "set_report", format: "csv", requested: true });
  state = [
    { type: "catalog_loaded", entries: catalog, provenance }, { type: "select_flow", flow: "SYNTHETIC" },
    { type: "select_version", schemaRef: "s_v1" }, { type: "configuration_loaded", token: 1, selection: configuration, traces },
    { type: "select_trace", traceRef: "config_1_trace_0" }, { type: "plan_created", token: 2, draft },
  ].reduce(reduce, state);
  assert.equal(state.plan.status, "ready");
  assert.deepEqual(state.reports, requested);
  for (const event of [{ type: "select_trace", traceRef: null }, { type: "select_version", schemaRef: null }, { type: "select_flow", flow: null },
    { type: "catalog_failed", message: "x" }]) {
    assert.deepEqual(reduce(state, event).reports, requested, event.type);
  }
});

const { AnalysisForm } = await loadComponent("AnalysisForm");
const render = state => renderToStaticMarkup(createElement(AnalysisForm, { state, onEvent: () => {} }));

test("the form shows a labelled group of native checkboxes with the current choice", () => {
  let html = render(INITIAL_SELECTION);
  assert.match(html, /<fieldset class="report-formats" aria-describedby="report-formats-note"><legend>Formati del report<\/legend>/);
  assert.match(html, /<label><input type="checkbox" checked=""\/>XLSX<\/label><label><input type="checkbox" checked=""\/>JSONL<\/label><label><input type="checkbox"\/>CSV<\/label>/);
  assert.match(html, /<p id="report-formats-note">Nessun file viene scritto ora\./);
  html = render(reduce(INITIAL_SELECTION, { type: "set_report", format: "csv", requested: true }));
  assert.match(html, /<label><input type="checkbox" checked=""\/>CSV<\/label>/);
});

test("the form dispatches one typed event per checkbox", async () => {
  const events = [];
  const tree = AnalysisForm({ state: INITIAL_SELECTION, onEvent: event => events.push(event) });
  const inputs = [];
  const walk = node => {
    if (Array.isArray(node)) return node.forEach(walk);
    if (!node || typeof node !== "object") return;
    if (typeof node.type === "function") return walk(node.type(node.props));
    if (node.type === "input" && node.props.type === "checkbox") inputs.push(node.props);
    walk(node.props?.children);
  };
  walk(tree);
  assert.equal(inputs.length, 3);
  inputs[0].onChange({ target: { checked: false } });
  inputs[2].onChange({ target: { checked: true } });
  assert.deepEqual(events, [
    { type: "set_report", format: "xlsx", requested: false },
    { type: "set_report", format: "csv", requested: true },
  ]);
});

test("no report writer or conversion exists in the frontend", () => {
  const source = new URL("../src/", import.meta.url);
  for (const name of readdirSync(source).filter(file => /\.(ts|tsx)$/.test(file))) {
    const text = readFileSync(new URL(name, source), "utf8");
    // TextEncoder is allowed: ipc.ts only measures IPC payload sizes with it.
    assert.doesNotMatch(text, /new Blob|createObjectURL|download=|writeFile|writeTextFile|plugin-fs|showSaveFilePicker|sheetjs|exceljs|from "xlsx"|papaparse/i, name);
  }
  const manifest = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8"));
  // DEC-MIG-08 stack, approved by the author for T-MIG-P6-01: UI and grid
  // libraries only; none of them reads or writes files.
  assert.deepEqual(Object.keys(manifest.dependencies).sort(), ["@radix-ui/react-slot", "@tanstack/react-table",
    "@tanstack/react-virtual", "@tauri-apps/api", "class-variance-authority", "clsx", "lucide-react", "react", "react-dom", "tailwind-merge"]);
});
