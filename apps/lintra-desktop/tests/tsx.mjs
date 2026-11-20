// Loads a real src/*.tsx component for server-side rendering in Node tests.
// Compiled with the Oxc transform already shipped by Vite (no new dependency);
// relative imports are rewritten to the source files (Node strips TS types).
import assert from "node:assert/strict";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { transformWithOxc } from "vite";

const base = new URL("../", import.meta.url);

export async function loadComponent(name) {
  const source = readFileSync(new URL(`src/${name}.tsx`, base), "utf8");
  const { code } = await transformWithOxc(source, `${name}.tsx`, { lang: "tsx", jsx: { runtime: "automatic" } });
  const rewritten = code.replace(/from "\.\/([A-Za-z]+)";/g, (_, module) => {
    const file = new URL(`src/${module}.ts`, base);
    assert.doesNotThrow(() => readFileSync(file), `missing src/${module}.ts`);
    return `from ${JSON.stringify(file.href)};`;
  });
  // Written under node_modules so bare imports (react) resolve as in the app.
  const directory = new URL("node_modules/.cache/lintra-component-test/", base);
  mkdirSync(directory, { recursive: true });
  const file = new URL(`${name}.mjs`, directory);
  writeFileSync(file, rewritten);
  return import(file.href);
}
