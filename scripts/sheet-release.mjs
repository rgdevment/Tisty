// Folio pins the hash of each package, so the same version with other styles would break every install of it.
import { spawnSync } from "node:child_process";
import { appendFileSync, copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { atRevision, isAfter, lines, planned, read } from "./contract-release.mjs";

const SHEET = "packages/tisty-sheet";
const STYLES = ["tokens.css", "page.css", "tables.css"];

export const wrongWith = ({ was, now, changed }) => {
  const [before, after] = [read(was), read(now)];
  if (!after) return [`${SHEET}/package.json says ${now}, which is no version`];
  if (was === now) return changed ? [`the styles the sheet publishes changed, and its version is still ${now}`] : [];
  return before && !isAfter(after, before) ? [`the sheet's version goes from ${was} to ${now}, and it only goes up`] : [];
};

export const lockedWrong = (version, locked) =>
  version === locked ? [] : [`${SHEET}/package.json says ${version} and package-lock.json says ${locked}`];

export const manifestOf = (version) => ({
  name: "@tisty/sheet",
  version,
  description: "How a Tisty document looks on its sheet, for whoever reads one.",
  license: "AGPL-3.0-only",
  repository: {
    type: "git",
    url: "git+https://github.com/rgdevment/Tisty.git",
    directory: SHEET,
  },
  sideEffects: ["**/*.css"],
  exports: Object.fromEntries(STYLES.map((one) => [`./${one}`, `./${one}`])),
  files: STYLES,
});

const versionAt = (revision) => JSON.parse(atRevision(revision, `${SHEET}/package.json`)).version;

const pullRequest = (base, head) => {
  const styles = STYLES.map((one) => `${SHEET}/src/${one}`);
  const differs = spawnSync("git", ["diff", "--quiet", base, head, "--", ...styles]);
  if (differs.status !== 0 && differs.status !== 1) throw new Error(`git diff could not compare ${base} and ${head}`);
  const wrong = wrongWith({ was: versionAt(base), now: versionAt(head), changed: differs.status === 1 });
  for (const one of wrong) console.log(`::error::${one}`);
  process.exitCode = wrong.length ? 1 : 0;
};

const main = () => {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const said = (path) => JSON.parse(readFileSync(join(root, path), "utf8"));
  const { version } = said(`${SHEET}/package.json`);
  const [what, ...rest] = process.argv.slice(2);
  if (what === "pr") {
    pullRequest(...rest);
  } else if (what === "check") {
    const wrong = lockedWrong(version, said("package-lock.json").packages[SHEET].version);
    for (const one of wrong) console.error(one);
    process.exitCode = wrong.length ? 1 : 0;
  } else if (what === "plan") {
    const { publish, tag, why } = planned({
      version,
      packaged: said("package-lock.json").packages[SHEET].version,
      releases: lines(process.env.RELEASES),
      tags: lines(process.env.TAGS),
      prefix: "sheet-v",
    });
    console.log(why);
    appendFileSync(process.env.GITHUB_OUTPUT, `publish=${publish}\ntag=${tag}\n`);
  } else if (what === "pack") {
    const [into] = rest;
    mkdirSync(into, { recursive: true });
    writeFileSync(join(into, "package.json"), `${JSON.stringify(manifestOf(version), null, 2)}\n`);
    for (const one of STYLES) copyFileSync(join(root, SHEET, "src", one), join(into, one));
  } else {
    throw new Error("say pr <base> <head>, check, plan, or pack <folder>");
  }
};

if (process.argv[1] === fileURLToPath(import.meta.url)) main();
