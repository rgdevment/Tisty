import assert from "node:assert/strict";
import { test } from "node:test";
import { metOn, wrongWith } from "./earlier-releases.mjs";

const listed = (count, onPullRequests) =>
  Array.from({ length: count }, (_, at) => ({ release: `1.${at}.0`, every_pull_request: at >= count - onPullRequests }));

test("a list within both maximums is fine", () => {
  assert.deepEqual(wrongWith(listed(10, 6)), []);
});

test("one release more than the maximum is refused", () => {
  assert.equal(wrongWith(listed(11, 6)).length, 1);
});

test("one more on every pull request than the maximum is refused", () => {
  assert.equal(wrongWith(listed(10, 7)).length, 1);
});

test("a release listed twice is refused", () => {
  const twice = [{ release: "1.0.0" }, { release: "1.0.0", every_pull_request: true }];
  assert.equal(wrongWith(twice).length, 1);
});

test("a newest release that pull requests skip is refused", () => {
  assert.equal(wrongWith(listed(3, 0)).length, 1);
});

test("a pull request meets its releases on one system, and anything else on every system", () => {
  const releases = listed(3, 1);
  assert.deepEqual(metOn("pull_request", releases), [{ release: "1.2.0", os: "windows-latest" }]);
  assert.equal(metOn("push", releases).length, 6);
  assert.equal(metOn("workflow_dispatch", releases).length, 6);
});
