import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const base = new URL("../", import.meta.url);
const repository = new URL("../../", base);
const provenance = JSON.parse(readFileSync(new URL("assets-provenance.json", base), "utf8"));
const sha = bytes => createHash("sha256").update(bytes).digest("hex");

test("local fonts and OFL license retain source bytes and attribution", () => {
  for (const entry of provenance.fonts) {
    const source = readFileSync(new URL(entry.source, repository));
    const copy = readFileSync(new URL(entry.destination, base));
    assert.deepEqual(copy, source);
    assert.equal(sha(copy), entry.sha256);
  }
  for (const [path, hash] of Object.entries(provenance.token_sources)) {
    assert.equal(sha(readFileSync(new URL(path, repository))), hash, path);
  }
});

test("CSS font resources are local and every declared font is packaged", () => {
  const tokens = readFileSync(new URL("src/tokens.css", base), "utf8");
  assert.doesNotMatch(tokens, /https?:|@import/);
  const fonts = [...tokens.matchAll(/url\("(\/fonts\/[^"/]+)"\)/g)];
  assert.equal(fonts.length, 3);
  for (const [, path] of fonts) assert.ok(readFileSync(new URL(`public${path}`, base)).length > 0);
  assert.match(tokens, /--brand: #0b2a5b;/);
  assert.match(tokens, /--brand-cyan: #00c2e8;/);
});
