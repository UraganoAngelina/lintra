// Loads a real src/*.tsx component for server-side rendering in Node tests.
// Compiled with the Oxc transform already shipped by Vite (no new dependency);
// relative imports are rewritten to the source files (Node strips TS types).
import assert from "node:assert/strict";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { transformWithOxc } from "vite";

const base = new URL("../", import.meta.url);

// Written under node_modules so bare imports (react) resolve as in the app.
const directory = new URL("node_modules/.cache/lintra-component-test/", base);

/** Compiles src/<name>.tsx and, recursively, the .tsx modules it imports
 * (T-MIG-P6-01: src/ui.tsx); .ts modules are imported as sources. */
async function compile(name) {
  const source = readFileSync(new URL(`src/${name}.tsx`, base), "utf8");
  const { code } = await transformWithOxc(source, `${name}.tsx`, { lang: "tsx", jsx: { runtime: "automatic" } });
  const components = [];
  const rewritten = code.replace(/from "\.\/([A-Za-z]+)";/g, (_, module) => {
    const tsx = new URL(`src/${module}.tsx`, base);
    try { readFileSync(tsx); components.push(module); return `from ${JSON.stringify(new URL(`${module}.mjs`, directory).href)};`; } catch { /* a .ts module */ }
    const file = new URL(`src/${module}.ts`, base);
    assert.doesNotThrow(() => readFileSync(file), `missing src/${module}.ts`);
    return `from ${JSON.stringify(file.href)};`;
  });
  mkdirSync(directory, { recursive: true });
  for (const module of components) await compile(module);
  const file = new URL(`${name}.mjs`, directory);
  writeFileSync(file, rewritten);
  return file;
}

export async function loadComponent(name) {
  return import((await compile(name)).href);
}
