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
  assert.deepEqual(metOn("pull_request", releases), [{ os: "windows-latest", releases: "1.2.0||||" }]);
  for (const event of ["push", "workflow_dispatch"]) {
    assert.deepEqual(
      metOn(event, releases).map((one) => [one.os, one.releases.split("\n").length]),
      [
        ["windows-latest", 3],
        ["macos-latest", 3],
      ],
    );
  }
});

test("a system is handed one line for each release, with what the meeting needs to know of it", () => {
  const releases = [
    { release: "1.30.5", schema: 17, shape: 2, accepts: "1.30.0" },
    { release: "1.33.0", schema: 18, shape: 3, accepts: "1.31.0", built: true, every_pull_request: true },
  ];
  assert.equal(metOn("push", releases)[0].releases, "1.30.5|17|2|1.30.0|\n1.33.0|18|3|1.31.0|built");
  assert.equal(metOn("pull_request", releases)[0].releases, "1.33.0|18|3|1.31.0|built");
});
