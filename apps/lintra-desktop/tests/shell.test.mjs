// T-MIG-P5-01: shell model, semantic markup of the real component and theme
// contrast. The component is compiled by tests/tsx.mjs and rendered with
// react-dom/server.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { loadComponent } from "./tsx.mjs";
import { INITIAL_THEME, INITIAL_VIEW, THEMES, VIEWS, applyTheme, isThemePreference, isViewId } from "../src/shell.ts";

const base = new URL("../", import.meta.url);

test("the bar keeps the ratified sections and starts on Analisi with the system theme", () => {
  assert.deepEqual(VIEWS.map(view => view.label), ["Analisi", "Risultati", "Informazioni"]);
  assert.equal(INITIAL_VIEW, "analisi");
  assert.deepEqual(THEMES.map(theme => theme.id), ["system", "light", "dark"]);
  assert.equal(INITIAL_THEME, "system");
});

test("untrusted section and theme values are rejected", () => {
  for (const value of ["", "Analisi", "risultati ", "__proto__", "toString", "report"]) {
    assert.equal(isViewId(value), false, value);
  }
  for (const value of ["", "System", "auto", "constructor", "dark "]) {
    assert.equal(isThemePreference(value), false, value);
  }
  assert.ok(VIEWS.every(view => isViewId(view.id)));
  assert.ok(THEMES.every(theme => isThemePreference(theme.id)));
});

test("an explicit theme sets data-theme and the system theme removes it", () => {
  const root = { dataset: {} };
  applyTheme(root, "dark");
  assert.equal(root.dataset.theme, "dark");
  applyTheme(root, "light");
  assert.equal(root.dataset.theme, "light");
  applyTheme(root, "system");
  assert.equal("theme" in root.dataset, false);
  applyTheme(root, "system");
  assert.equal("theme" in root.dataset, false);
});

const { AppShell } = await loadComponent("AppShell");
const noop = () => {};
function render(props) {
  return renderToStaticMarkup(createElement(AppShell, {
    view: "analisi", onView: noop, theme: "system", onTheme: noop,
    service: { kind: "starting" }, reportAvailable: false, ...props,
  }));
}
const count = (html, pattern) => (html.match(pattern) ?? []).length;

test("landmarks: skip link, banner with labelled navigation, main with one heading", () => {
  const html = render({});
  assert.ok(html.startsWith('<div class="shell"><a class="skip-link" href="#content">Vai al contenuto</a><header class="bar">'));
  assert.match(html, /<nav aria-label="Sezioni"><ul>(<li><button type="button"[^>]*>.*?<\/button><\/li>){3}<\/ul><\/nav>/);
  assert.match(html, /<main id="content" tabindex="-1"><h1 tabindex="-1">Analisi<\/h1>/);
  assert.equal(count(html, /<h1\b/g), 1);
  assert.equal(count(html, /<header\b/g), 1);
  assert.equal(count(html, /<main\b/g), 1);
  // CSP: no inline style attributes and no images (img-src 'none').
  assert.equal(count(html, /\sstyle=/g), 0);
  assert.equal(count(html, /<img\b/g), 0);
  // Clickable things are buttons or links, never generic elements.
  assert.equal(count(html, /onclick|role="button"/gi), 0);
});

test("exactly the selected section is marked current and titles the main region", () => {
  for (const view of VIEWS) {
    const html = render({ view: view.id });
    assert.equal(count(html, /aria-current="page"/g), 1, view.id);
    assert.match(html, new RegExp(`<button type="button" aria-current="page"><span>${view.label}</span>`));
    assert.match(html, new RegExp(`<h1 tabindex="-1">${view.label}</h1>`));
  }
});

test("the theme is a labelled radio group with the preference checked", () => {
  for (const theme of THEMES) {
    const html = render({ theme: theme.id });
    assert.match(html, /<fieldset class="theme"><legend>Tema<\/legend>/);
    assert.equal(count(html, /<input type="radio" name="theme"/g), 3);
    assert.equal(count(html, /checked=""/g), 1);
    assert.match(html, new RegExp(`<label><input type="radio" name="theme" checked="" value="${theme.id}"/>${theme.label}</label>`));
  }
});

test("service and report states show only what the backend reported", () => {
  assert.match(render({}), /<p class="service" role="status">Avvio del servizio…<\/p>/);
  const ready = render({ service: { kind: "ready", version: "0.1.1-rc.3" }, view: "informazioni" });
  assert.match(ready, /role="status">Servizio locale pronto</);
  assert.match(ready, /<dt>Versione<\/dt><dd>0\.1\.1-rc\.3<\/dd>/);
  assert.match(render({ view: "informazioni" }), /<dd>Non ancora disponibile<\/dd>/);
  const failed = render({ service: { kind: "error", message: "Servizio non disponibile <x>" } });
  assert.match(failed, /<p class="service" role="alert">Servizio non disponibile &lt;x&gt;<\/p>/);
  assert.equal(count(failed, /role="status"/g), 0);
  assert.match(render({}), /<span class="nav-note">Nessun report<\/span>/);
  assert.match(render({ view: "risultati" }), /Nessun report disponibile\./);
  assert.match(render({ reportAvailable: true }), /<span class="nav-note">Report disponibile<\/span>/);
  // No invented success or KPI on the empty Analisi view.
  const empty = render({});
  assert.match(empty, /Prepara una nuova analisi\./);
  assert.doesNotMatch(empty, /conform|nessuna violazione|%/i);
});

// WCAG 2 relative luminance contrast on the solid token colours.
function luminance(hex) {
  const [r, g, b] = [1, 3, 5].map(i => parseInt(hex.slice(i, i + 2), 16) / 255)
    .map(c => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}
function contrast(a, b) {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}
function palette(scheme) {
  const tokens = readFileSync(new URL("src/tokens.css", base), "utf8");
  const raw = Object.fromEntries([...tokens.matchAll(/--([\w-]+):\s*([^;]+);/g)].map(m => [m[1], m[2].trim()]));
  const resolve = value => {
    const pair = value.match(/^light-dark\((.+),\s*(.+)\)$/);
    if (pair) return resolve(pair[scheme === "light" ? 1 : 2].trim());
    const reference = value.match(/^var\(--([\w-]+)\)$/);
    if (reference) return resolve(raw[reference[1]]);
    assert.match(value, /^#[0-9a-f]{6}$/);
    return value;
  };
  return name => resolve(raw[name]);
}

test("text roles keep at least 4.5:1 and the focus ring 3:1 in both themes", () => {
  const text = [["ink", "bg"], ["ink", "surface"], ["muted", "bg"], ["muted", "surface"],
    ["accent", "surface"], ["selected-ink", "selected-bg"], ["error", "surface"],
    ["warn", "surface"], ["surface", "accent"]];
  for (const scheme of ["light", "dark"]) {
    const color = palette(scheme);
    for (const [fg, bg] of text) {
      const ratio = contrast(color(fg), color(bg));
      assert.ok(ratio >= 4.5, `${scheme} ${fg}/${bg} ${ratio.toFixed(2)}`);
    }
    for (const bg of ["bg", "surface", "selected-bg"]) {
      const ratio = contrast(color("accent"), color(bg));
      assert.ok(ratio >= 3, `${scheme} focus/${bg} ${ratio.toFixed(2)}`);
    }
  }
});

test("colours live only in tokens.css", () => {
  for (const file of ["src/style.css", "src/AppShell.tsx", "src/main.tsx"]) {
    const text = readFileSync(new URL(file, base), "utf8");
    assert.doesNotMatch(text, /#[0-9a-f]{3,8}\b|rgb\(|hsl\(/i, file);
  }
});
