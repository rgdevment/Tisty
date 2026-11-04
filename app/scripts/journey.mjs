// The whole way a document goes: Rust puts what it names in place, a real browser colours and draws it, Rust seals and opens it, and the contract looks at what came out in the other language too.
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { drawnInChrome } from "./chrome.mjs";
import { bundledFinisher, finisherPage } from "./finisher.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
const { check, checkHtml } = await import(pathToFileURL(join(root, "contracts", "dist", "typescript", "index.js")));
const TEN_MINUTES = 600_000;

const folder = mkdtempSync(join(tmpdir(), "tisty-journey-"));
const inRust = (test) =>
  execFileSync("cargo", ["test", "-q", "-p", "tisty-envelope", "--test", "journey", "--", "--ignored", "--exact", test], {
    cwd: root,
    env: { ...process.env, TISTY_JOURNEY: folder },
    stdio: "inherit",
  });
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
