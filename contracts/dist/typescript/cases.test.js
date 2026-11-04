import assert from "node:assert/strict";
import { test } from "node:test";
import { check, checkHtml, CONTRACTS } from "./index.js";

for (const [name, contract] of Object.entries(CONTRACTS)) {
  for (const one of contract.cases ?? []) {
    test(name + ": " + one.says, () => {
      assert.equal(check(name, one.value).length === 0, one.holds, check(name, one.value).join("; "));
    });
  }
  for (const one of contract.html_cases ?? []) {
    test(name + " html: " + one.says, () => {
      assert.equal(checkHtml(one.html).length === 0, one.holds, checkHtml(one.html).join("; "));
    });
  }
}
