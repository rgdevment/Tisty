import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { test } from "node:test";
import { check, checkHtml, CONTRACTS, ALLOWED, PASSTHROUGH, STYLES, URLS, VERSION, picturesIn, referencesIn } from "./index.js";
import * as entry from "./check.js";

test("the check entry leaves out what a verdict does not read", () => {
  assert.equal(entry.CONTRACTS, undefined);
  const held = readFileSync(new URL("./check.js", import.meta.url), "utf8");
  assert.doesNotMatch(held, /"(cases|html_cases)":/);
  assert.ok(held.length < 48 * 1024, "check.js holds " + held.length + " bytes");
});

test("the check entry says the same as the full one", () => {
  for (const [name, one] of [["ALLOWED", ALLOWED], ["PASSTHROUGH", PASSTHROUGH], ["STYLES", STYLES], ["URLS", URLS], ["VERSION", VERSION]]) {
    assert.deepEqual(entry[name], one, name);
  }
  const html = '<a href="tisty:page/1">a</a><img src="tisty:picture/0" alt="a" width="1" height="1">';
  assert.deepEqual(entry.referencesIn(html), referencesIn(html));
  assert.deepEqual(entry.picturesIn(html), picturesIn(html));
});

test("the edit address hands over its link whole, key and all", () => {
  const { edit, edit_example: { link, address } } = CONTRACTS.link.values;
  assert.equal(address, edit.split("{")[0] + encodeURIComponent(link));
  const read = new URL(address);
  assert.deepEqual([...read.searchParams], [["link", link]]);
  assert.equal(read.hash, "");
});

test("every entry of the package is a file it holds", () => {
  const { exports: entries, files } = JSON.parse(readFileSync(new URL("./package.json", import.meta.url), "utf8"));
  for (const entry of Object.values(entries)) {
    for (const file of typeof entry === "string" ? [entry] : Object.values(entry)) {
      assert.ok(existsSync(new URL(file, import.meta.url)), file);
      assert.ok(files.includes(file.slice(2)), file);
    }
  }
});

for (const [name, contract] of Object.entries(CONTRACTS)) {
  for (const one of contract.cases ?? []) {
    test(name + ": " + one.says, () => {
      assert.equal(check(name, one.value).length === 0, one.holds, check(name, one.value).join("; "));
      assert.deepEqual(entry.check(name, one.value), check(name, one.value));
    });
  }
  for (const one of contract.html_cases ?? []) {
    test(name + " html: " + one.says, () => {
      assert.equal(checkHtml(one.html).length === 0, one.holds, checkHtml(one.html).join("; "));
      assert.deepEqual(entry.checkHtml(one.html), checkHtml(one.html));
    });
  }
}
