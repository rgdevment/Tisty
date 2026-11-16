import assert from "node:assert/strict";
import { test } from "node:test";
import { missingFromCheck } from "./check-runs-ci.mjs";

const window = (steps, folder = "app") => ({
  jobs: { window: { defaults: { run: { "working-directory": folder } }, steps } },
});

test("a step CI runs and the check does not is named with its folder", () => {
  const ci = window([{ run: "npm run lint" }, { name: "The budget", run: "node scripts/light-viewer.mjs" }]);
  const local = [["the window is linted", "npm", ["run", "lint"], "app"]];
  assert.deepEqual(missingFromCheck(ci, local), ["app: node scripts/light-viewer.mjs"]);
});

test("the same command in another folder is another step", () => {
  const ci = window([{ run: "npm test -w tisty-light-viewer" }]);
  const local = [["the light viewer's tests pass", "npm", ["test", "-w", "tisty-light-viewer"]]];
  assert.deepEqual(missingFromCheck(ci, local), ["app: npm test -w tisty-light-viewer"]);
});

test("a step's own folder wins over the job's", () => {
  const ci = window([{ run: "npm run lint:md", "working-directory": "." }]);
  const local = [["the documents are linted", "npm", ["run", "lint:md"]]];
  assert.deepEqual(missingFromCheck(ci, local), []);
});

test("installing is left to CI, and steps that only use an action are passed over", () => {
  const ci = window([{ uses: "actions/checkout@v7" }, { run: "npm ci", "working-directory": "." }, { run: "npm run build" }]);
  const local = [["the window builds", "npm", ["run", "build"], "app"]];
  assert.deepEqual(missingFromCheck(ci, local), []);
});

test("a job without a default folder runs at the root", () => {
  const ci = { jobs: { window: { steps: [{ run: "npm run build" }] } } };
  assert.deepEqual(missingFromCheck(ci, [["the window builds", "npm", ["run", "build"], "app"]]), [".: npm run build"]);
});

test("a workflow without the job, or with nothing to compare, stops the check", () => {
  assert.throws(() => missingFromCheck({ jobs: {} }, []), /no job window/);
  assert.throws(() => missingFromCheck(window([{ run: "npm ci" }]), []), /no longer knows/);
});
