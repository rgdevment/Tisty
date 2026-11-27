import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { after, test } from "node:test";
import { fileURLToPath } from "node:url";
import { floating } from "./promises.mjs";

const checker = resolve(dirname(fileURLToPath(import.meta.url)), "promises.mjs");
const TSCONFIG = JSON.stringify({
  compilerOptions: {
    strict: true,
    lib: ["ES2020", "DOM"],
    moduleResolution: "bundler",
    module: "ESNext",
    target: "ES2020",
    noEmit: true,
  },
  include: ["src"],
});
const FLOATING =
  "Promises must be awaited, end with a call to .catch, end with a call to .then with a rejection handler or be explicitly marked as ignored with the `void` operator.";
const NOT_A_FUNCTION = " A rejection handler that is not a function will be ignored.";

const roots = [];

after(() => {
  for (const root of roots) rmSync(root, { recursive: true, force: true });
});

const tree = (files, project = "app") => {
  const root = mkdtempSync(join(tmpdir(), "promises-"));
  roots.push(root);
  const base = join(root, project);
  mkdirSync(base, { recursive: true });
  writeFileSync(join(base, "tsconfig.json"), TSCONFIG);
  for (const [name, text] of Object.entries(files)) {
    const path = join(root, name);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, text);
  }
  return root;
};

const check = (source) => floating(tree({ "app/src/a.ts": source }));

const HEAD = "async function load(): Promise<number> { return 1; }\n";

test("a bare call to an async function is reported at its line and column", () => {
  const hits = check(`${HEAD}function run() {\n  load();\n}\n`);
  assert.deepEqual(hits, [{ file: "app/src/a.ts", line: 3, column: 3, message: FLOATING }]);
});

test("a bare call to a function returning a Promise is reported", () => {
  const hits = check(`function make(): Promise<void> { return Promise.resolve(); }\nmake();\n`);
  assert.equal(hits.length, 1);
  assert.equal(hits[0].line, 2);
  assert.equal(hits[0].column, 1);
});

test("void, await, return, assignment, catch and then with a rejection handler all handle it", () => {
  const hits = check(
    `${HEAD}async function run() {
  void load();
  await load();
  const kept = load();
  let other: Promise<number>;
  other = load();
  load().catch((e) => console.log(e));
  load().then((n) => n, (e) => console.log(e));
  return load();
}
void [kept()];
function kept() { return 1; }
`,
  );
  assert.deepEqual(hits, []);
});

test("then with only a success handler is still floating", () => {
  const hits = check(`${HEAD}load().then((n) => n);\n`);
  assert.equal(hits.length, 1);
  assert.equal(hits[0].message, FLOATING);
});

test("finally defers to its receiver", () => {
  const hits = check(
    `${HEAD}const f = () => 1;
const g = () => 2;
const h = (e: unknown) => e;
load().then(f).finally(g);
load().then(f).catch(h).finally(g);
`,
  );
  assert.deepEqual(
    hits.map((one) => one.line),
    [5],
  );
});

test("a local arrow whose type is inferred from a catch and finally chain is floating when called bare", () => {
  const hits = check(
    `${HEAD}const go = () => load().catch(() => 0).finally(() => undefined);
go();
`,
  );
  assert.deepEqual(
    hits.map((one) => one.line),
    [3],
  );
});

test("a catch handler typed by an uninstalled package is reported with the extra sentence", () => {
  const hits = check(
    `import { useCallback } from "react";
${HEAD}const handler = useCallback(() => 1, []);
load().catch(handler);
`,
  );
  assert.equal(hits.length, 1);
  assert.equal(hits[0].line, 4);
  assert.equal(hits[0].message, FLOATING + NOT_A_FUNCTION);
  assert.ok(hits[0].message.endsWith("A rejection handler that is not a function will be ignored."));
});

test("an inline arrow handler beside an uninstalled package is not reported", () => {
  const hits = check(
    `import { useCallback } from "react";
${HEAD}const unused = useCallback(() => 1, []);
load().catch((e) => console.log(e, unused));
`,
  );
  assert.deepEqual(hits, []);
});

test("an optional call with a catch is handled and a bare optional call is reported", () => {
  const hits = check(
    `declare const maybe: (() => Promise<number>) | undefined;
maybe?.().catch(() => undefined);
maybe?.();
`,
  );
  assert.deepEqual(
    hits.map((one) => one.line),
    [3],
  );
});

test("a conditional and a logical expression check their branches", () => {
  const hits = check(
    `${HEAD}declare const cond: boolean;
declare const ok: boolean;
cond ? load() : undefined;
ok && load();
cond ? load().catch(() => 0) : undefined;
ok && load().catch(() => 0);
`,
  );
  assert.deepEqual(
    hits.map((one) => one.line),
    [4, 5],
  );
});

test("an array of promises as a statement gets the array message", () => {
  const hits = check(`${HEAD}async function other() { return 2; }\n[load(), other()];\n`);
  assert.equal(hits.length, 1);
  assert.equal(hits[0].line, 3);
  assert.match(hits[0].message, /^An array of Promises may be unintentional/);
});

test("expressions that are not promises are left alone", () => {
  const hits = check(
    `let n = 0;
n++;
n = 2;
console.log(n);
"use strict";
[1, 2, 3].map((x) => x);
`,
  );
  assert.deepEqual(hits, []);
});

test("a file under a tests folder is skipped and a package source is scanned", () => {
  const root = tree(
    {
      "app/src/tests/a.ts": `${HEAD}load();\n`,
      "packages/x/tsconfig.json": TSCONFIG,
      "packages/x/src/b.ts": `${HEAD}load();\n`,
    },
    "app",
  );
  const hits = floating(root);
  assert.deepEqual(
    hits.map((one) => `${one.file}:${one.line}:${one.column}`),
    ["packages/x/src/b.ts:2:1"],
  );
});

test("the command line exits 1 with file, line and column on a floating tree", () => {
  const root = tree({ "app/src/a.ts": `${HEAD}load();\n` });
  const run = spawnSync(process.execPath, [checker, root], { encoding: "utf8" });
  assert.equal(run.status, 1);
  assert.match(run.stdout, /^app\/src\/a\.ts:2:1 Promises must be awaited/m);
});

test("the command line exits 0 and prints nothing on a clean tree", () => {
  const root = tree({ "app/src/a.ts": `${HEAD}void load();\n` });
  const run = spawnSync(process.execPath, [checker, root], { encoding: "utf8" });
  assert.equal(run.status, 0);
  assert.equal(run.stdout, "");
});
