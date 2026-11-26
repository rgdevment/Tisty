import assert from "node:assert/strict";
import { test } from "node:test";
import { metOn, wrongWith } from "./earlier-releases.mjs";

const WINDOWS = "a".repeat(64);
const MACOS = "b".repeat(64);
const pinned = { windows: WINDOWS, macos: MACOS };

const listed = (count) =>
  Array.from({ length: count }, (_, at) => ({ release: `1.${at}.0`, every_pull_request: at === 0 || at === count - 1, sha256: pinned }));

test("a list within its maximum, met on pull requests at its two ends, is fine", () => {
  assert.deepEqual(wrongWith(listed(10)), []);
});

test("one release more than the maximum is refused", () => {
  assert.equal(wrongWith(listed(11)).length, 1);
});

test("a release between the two ends that pull requests meet is refused", () => {
  const releases = listed(10);
  releases[4].every_pull_request = true;
  assert.equal(wrongWith(releases).length, 1);
});

test("a release listed twice is refused", () => {
  const twice = listed(3);
  twice[1].release = twice[0].release;
  assert.equal(wrongWith(twice).length, 1);
});

test("an oldest release that pull requests skip is refused", () => {
  const releases = listed(3);
  releases[0].every_pull_request = false;
  assert.equal(wrongWith(releases).length, 1);
});

test("a newest release that pull requests skip is refused", () => {
  const releases = listed(3);
  releases[2].every_pull_request = false;
  assert.equal(wrongWith(releases).length, 1);
});

test("a downloaded release without the hashes of what it downloads is refused", () => {
  const releases = listed(3);
  delete releases[1].sha256;
  assert.equal(wrongWith(releases).length, 1);
});

test("a downloaded release pinned for one system only, or with a hash that is not 64 lowercase hex digits, is refused", () => {
  for (const sha256 of [{ windows: WINDOWS }, { macos: MACOS }, { ...pinned, macos: "b".repeat(63) }, { ...pinned, windows: "A".repeat(64) }]) {
    const releases = listed(3);
    releases[1].sha256 = sha256;
    assert.equal(wrongWith(releases).length, 1);
  }
});

test("a release built from its tag needs no hashes", () => {
  const releases = listed(3);
  delete releases[1].sha256;
  releases[1].built = true;
  assert.deepEqual(wrongWith(releases), []);
});

test("a pull request meets its releases on one system, and anything else on every system", () => {
  const releases = listed(3);
  assert.deepEqual(metOn("pull_request", releases), [
    { os: "windows-latest", releases: `1.0.0|||||${WINDOWS}\n1.2.0|||||${WINDOWS}`, downloaded: "1.0.0_1.1.0_1.2.0" },
  ]);
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
    { release: "1.30.5", schema: 17, shape: 2, accepts: "1.30.0", sha256: pinned },
    { release: "1.33.0", schema: 18, shape: 3, accepts: "1.31.0", built: true, every_pull_request: true },
  ];
  assert.equal(metOn("push", releases)[0].releases, `1.30.5|17|2|1.30.0||${WINDOWS}\n1.33.0|18|3|1.31.0|built|`);
  assert.equal(metOn("pull_request", releases)[0].releases, "1.33.0|18|3|1.31.0|built|");
});

test("each system is handed the hash of what it downloads, and no other", () => {
  const releases = [{ release: "1.30.5", schema: 17, shape: 2, sha256: pinned, every_pull_request: true }];
  assert.deepEqual(
    metOn("push", releases).map((one) => [one.os, one.releases]),
    [
      ["windows-latest", `1.30.5|17|2|||${WINDOWS}`],
      ["macos-latest", `1.30.5|17|2|||${MACOS}`],
    ],
  );
  assert.equal(metOn("pull_request", releases)[0].releases, `1.30.5|17|2|||${WINDOWS}`);
});

test("every system is told the same releases it downloads, whatever the event, so a pull request finds main's", () => {
  const releases = [
    { release: "1.23.1", every_pull_request: true },
    { release: "1.30.5" },
    { release: "1.33.0", built: true },
    { release: "1.47.1", built: true, every_pull_request: true },
  ];
  for (const event of ["pull_request", "push"]) {
    for (const one of metOn(event, releases)) assert.equal(one.downloaded, "1.23.1_1.30.5");
  }
});

test("a list built entirely from its tags downloads nothing", () => {
  const releases = [{ release: "1.33.0", built: true, every_pull_request: true }];
  assert.equal(metOn("push", releases)[0].downloaded, "");
});
