// What Tisty draws with and what its contracts promise are one thing, so neither can move alone.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { CONTRACTS } from "./dist/typescript/index.js";

const read = (path) => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");

test("the colours of a sheet are the ones Tisty draws with", () => {
  const [light, dark] = read("packages/tisty-editor/src/tokens.css").split('[data-theme="dark"]');
  const colors = (block) =>
    Object.fromEntries([...block.matchAll(/--tisty-([a-z-]+):\s*([^;]+);/g)].map(([, name, value]) => [name, value.trim()]));
  assert.deepEqual(CONTRACTS.style.values.colors.light, colors(light));
  assert.deepEqual(CONTRACTS.style.values.colors.dark, colors(dark));
});

test("the papers are the ones Tisty lays on", () => {
  const paper = read("packages/tisty-editor/src/paper.ts");
  const said = Object.fromEntries(
    [...paper.matchAll(/^\s*([a-z0-9]+): \[([\d.]+), ([\d.]+)\],$/gm)].map(([, name, wide, tall]) => [name, [Number(wide), Number(tall)]]),
  );
  assert.deepEqual(CONTRACTS.sheet.values.millimetres, said);
  assert.deepEqual(CONTRACTS.document.enums.paper, Object.keys(said));
  const margin = /MARGIN_MM = \{ x: (\d+), y: (\d+) \}/.exec(paper);
  assert.deepEqual(CONTRACTS.sheet.values.margin_millimetres, { x: Number(margin[1]), y: Number(margin[2]) });
});

test("a page holds what a document can hold, at most what Tisty keeps", () => {
  const most = /BODY_AT_MOST: u64 = (\d+) \* 1024/.exec(read("crates/tisty-core/src/docs.rs"));
  assert.equal(CONTRACTS.page.fields.body.at_most, Number(most[1]) * 1024);
});
