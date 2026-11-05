import assert from "node:assert/strict";
import { test } from "node:test";
import { judged, planned } from "./contract-release.mjs";

const wrong = (said) => judged(said).wrong.length > 0;

test("what the package publishes changed, so its version goes up", () => {
  assert.ok(wrong({ was: "1.0.0", now: "1.0.0", changed: true }));
  assert.ok(!wrong({ was: "1.0.0", now: "1.0.1", changed: true }));
});

test("nothing it publishes changed, so the version is free to stay", () => {
  assert.ok(!wrong({ was: "1.0.0", now: "1.0.0", changed: false }));
});

test("a version only goes up", () => {
  assert.ok(wrong({ was: "1.2.0", now: "1.1.9", changed: true }));
  assert.ok(wrong({ was: "1.0.0", now: "0.9.0", changed: true }));
  assert.ok(wrong({ was: "1.0.0", now: "1.0.0-draft.1", changed: true }));
  assert.ok(wrong({ was: "1.1.0-draft.2", now: "1.1.0-draft.1", changed: true }));
  assert.ok(!wrong({ was: "1.0.10", now: "1.0.11", changed: true }));
  assert.ok(!wrong({ was: "1.9.0", now: "1.10.0", changed: true }));
  assert.ok(!wrong({ was: "1.1.0-draft.2", now: "1.1.0-draft.10", changed: true }));
  assert.ok(!wrong({ was: "1.1.0-draft.2", now: "1.1.0", changed: true }));
});

test("what is no version is refused", () => {
  assert.ok(wrong({ was: "1.0.0", now: "uno", changed: true }));
  assert.ok(wrong({ was: "1.0.0", now: "1.1", changed: true }));
});

test("every new version reminds who deploys first", () => {
  assert.match(judged({ was: "1.4.0", now: "1.5.0", changed: true }).reminders[0], /Folio first/);
  assert.match(judged({ was: "1.1.0-draft.3", now: "1.1.0", changed: true }).reminders[0], /Folio first/);
  assert.deepEqual(judged({ was: "1.1.0-draft.1", now: "1.1.0-draft.2", changed: true }).reminders, []);
  assert.deepEqual(judged({ was: "1.1.0", now: "1.1.0", changed: false }).reminders, []);
});

test("the major version stays at 1 while nothing has been shared for real", () => {
  assert.ok(wrong({ was: "1.4.0", now: "2.0.0", changed: true }));
  assert.ok(wrong({ was: "1.4.0", now: "2.0.0-draft.1", changed: true }));
});

const out = { version: "1.0.1", packaged: "1.0.1", releases: ["contracts-v1.0.0", "v1.41.1"], tags: ["contracts-v1.0.0", "v1.41.1"] };

test("a new version goes out under a tag of its own, which no release of the app answers to", () => {
  const { publish, tag } = planned(out);
  assert.ok(publish);
  assert.equal(tag, "contracts-v1.0.1");
  assert.ok(!tag.startsWith("v"));
});

test("what is out is never replaced", () => {
  assert.equal(planned({ ...out, version: "1.0.0", packaged: "1.0.0" }).publish, false);
});

test("a draft is never published", () => {
  assert.equal(planned({ ...out, version: "1.1.0-draft.1", packaged: "1.1.0-draft.1" }).publish, false);
});

test("a package that says another version than the contract stops it", () => {
  assert.throws(() => planned({ ...out, packaged: "1.0.0" }));
});

test("a tag left with no release stops it, rather than passing for published", () => {
  assert.throws(() => planned({ ...out, tags: [...out.tags, "contracts-v1.0.1"] }));
});
