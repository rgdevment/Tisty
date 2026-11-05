// The whole way a document goes: from a real store, Rust puts what it names in place, a real browser colours and draws it, Rust seals it, and the contract's own readers open every envelope, in Rust and in what a browser brings.
import { execFileSync } from "node:child_process";
import { appendFileSync, cpSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { drawnInChrome } from "./chrome.mjs";
import { bundledFinisher, finisherPage } from "./finisher.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
const dist = join(root, "contracts", "dist", "typescript");
const { check, checkHtml } = await import(pathToFileURL(join(dist, "index.js")));
const TEN_MINUTES = 600_000;

// The sample reader exports nothing and the contract is not touched, so a copy of it is handed one more test, written in its own words.
const EVERY_ENVELOPE = `
test("every envelope Tisty sealed on the way opens here", async () => {
  const sealed = JSON.parse(readFileSync(process.env.TISTY_SEALED, "utf8"));
  assert.ok(sealed.length > 100, "the journey sealed what it should");
  for (const { id, key, main, files } of sealed) {
    const linkKey = key ? b64(key) : undefined;
    const { main: open, held } = await reading(hex(main), { id, kind: "main" }, linkKey);
    (held.pictures ?? []).forEach((one, at) => assert.ok(pictureHolds(open.parts[at + 2], one), id));
    if (files) await filesFor(held, hex(files), { id, kind: "files" }, linkKey);
    assert.equal(!files && (held.files ?? []).length > 0, false, id);
  }
});
`;

const folder = mkdtempSync(join(tmpdir(), "tisty-journey-"));
const inRust = (test) =>
  execFileSync("cargo", ["test", "-q", "-p", "tisty-sharing", "--test", "journey", "--", "--ignored", "--exact", test], {
    cwd: root,
    env: { ...process.env, TISTY_JOURNEY: folder },
    stdio: "inherit",
  });
const inJavaScript = () => {
  const copy = join(folder, "reader");
  cpSync(dist, copy, { recursive: true });
  appendFileSync(join(copy, "vectors.test.js"), EVERY_ENVELOPE);
  execFileSync(process.execPath, ["--test", "--test-name-pattern=every envelope Tisty sealed", join(copy, "vectors.test.js")], {
    cwd: copy,
    env: { ...process.env, TISTY_SEALED: join(folder, "sealed.json") },
    stdio: "inherit",
  });
};
const read = (name) => JSON.parse(readFileSync(join(folder, name), "utf8"));

try {
  inRust("ready_for_the_window");
  const files = join(folder, "built");
  await bundledFinisher(files);
  const finished = await drawnInChrome(
    read("ready.json"),
    files,
    finisherPage("finishWhole"),
    "the documents on their way",
    TEN_MINUTES,
  );
  writeFileSync(join(folder, "finished.json"), JSON.stringify(finished));
  inRust("sealed_from_the_window");
  inJavaScript();

  const opened = read("opened.json");
  const wrong = opened.flatMap(({ name, document }) =>
    [...check("document", document), ...checkHtml(document.html)].map((one) => `${name}: ${one}`),
  );
  for (const one of wrong) console.error(`x  ${one}`);
  const notDrawn = finished.reduce((all, one) => all + one.notDrawn, 0);
  const looks = opened.reduce((all, { document }) => all + document.html.split('data-theme="dark"').length - 1, 0);
  console.log(`${opened.length - new Set(wrong.map((one) => one.split(": ")[0])).size} of ${opened.length} documents arrive as the contract says, with ${looks} diagrams drawn and ${notDrawn} gone as code`);
  process.exitCode = wrong.length ? 1 : 0;
} finally {
  rmSync(folder, { recursive: true, force: true });
}
