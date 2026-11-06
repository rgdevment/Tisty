import assert from "node:assert/strict";
import { test } from "node:test";
import { planned } from "./contract-release.mjs";
import { lockedWrong, manifestOf, wrongWith } from "./sheet-release.mjs";

test("styles that changed ask for a higher version", () => {
  assert.equal(wrongWith({ was: "1.0.0", now: "1.0.0", changed: true }).length, 1);
  assert.deepEqual(wrongWith({ was: "1.0.0", now: "1.0.1", changed: true }), []);
});

test("styles that did not change leave the version free", () => {
  assert.deepEqual(wrongWith({ was: "1.0.0", now: "1.0.0", changed: false }), []);
  assert.deepEqual(wrongWith({ was: "1.0.0", now: "1.1.0", changed: false }), []);
});

test("a version only goes up", () => {
  assert.equal(wrongWith({ was: "1.1.0", now: "1.0.9", changed: true }).length, 1);
});

test("something that is no version is refused", () => {
  assert.equal(wrongWith({ was: "1.0.0", now: "one", changed: true }).length, 1);
});

test("the lockfile has to say the same version", () => {
  assert.deepEqual(lockedWrong("1.0.0", "1.0.0"), []);
  assert.equal(lockedWrong("1.0.1", "1.0.0").length, 1);
});

test("the package holds only the styles", () => {
  const { files, exports: entries, dependencies, peerDependencies, version } = manifestOf("1.0.0");
  assert.deepEqual(files, ["tokens.css", "page.css", "tables.css"]);
  assert.deepEqual(Object.keys(entries), ["./tokens.css", "./page.css", "./tables.css"]);
  assert.equal(dependencies, undefined);
  assert.equal(peerDependencies, undefined);
  assert.equal(version, "1.0.0");
});

test("a sheet version goes out under its own tag, and only once", () => {
  const said = { version: "1.0.0", packaged: "1.0.0", tags: [], prefix: "sheet-v" };
  assert.equal(planned({ ...said, releases: [] }).tag, "sheet-v1.0.0");
  assert.equal(planned({ ...said, releases: [] }).publish, true);
  assert.equal(planned({ ...said, releases: ["sheet-v1.0.0"] }).publish, false);
  assert.equal(planned({ ...said, releases: ["contracts-v1.0.0"] }).publish, true);
});
