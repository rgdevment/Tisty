// What publishing the contract decides. If what a package holds changes, its version goes up: Folio
// pins the hash of each one, so the same version with other bytes would break every install of it.
import { execFileSync, spawnSync } from "node:child_process";
import { appendFileSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const PACKAGE = "contracts/dist/typescript";
// Tisty takes the crate by path and nothing is released of it, but it gives the same verdicts.
const ALSO_PUBLISHED = ["contracts/dist/contract.json", "contracts/dist/rust/src", "contracts/dist/rust/Cargo.toml"];

const read = (said) => {
  const found = /^(\d+)\.(\d+)\.(\d+)(?:-(.+))?$/.exec(said);
  return found && { numbers: found.slice(1, 4).map(Number), draft: found[4] ?? null };
};

const isAfter = (now, was) => {
  for (const [at, one] of now.numbers.entries()) {
    if (one !== was.numbers[at]) return one > was.numbers[at];
  }
  if (now.draft === was.draft) return false;
  if (!was.draft || !now.draft) return Boolean(was.draft);
  return now.draft.localeCompare(was.draft, "en", { numeric: true }) > 0;
};

/** What a pull request has to hear about the version it leaves. */
export const judged = ({ was, now, changed }) => {
  const [before, after] = [read(was), read(now)];
  if (!after) return { wrong: [`contracts/VERSION says ${now}, which is no version`], reminders: [] };
  const wrong = [];
  if (changed && was === now) wrong.push(`what the package publishes changed, and contracts/VERSION is still ${now}`);
  if (was !== now && before && !isAfter(after, before)) wrong.push(`contracts/VERSION goes from ${was} to ${now}, and it only goes up`);
  const [majorNow, minorNow, patchNow] = after.numbers;
  const draftBecomesMajor = before?.draft && !after.draft && minorNow === 0 && patchNow === 0;
  const major = before && (majorNow > before.numbers[0] || draftBecomesMajor);
  return {
    wrong,
    reminders: major ? [`a new major version: deploy Folio first, and Tisty after it, since a reader refuses what is newer than it`] : [],
  };
};

/** What the workflow does with the version on main. */
export const planned = ({ version, packaged, releases, tags }) => {
  if (version !== packaged) throw new Error(`contracts/VERSION says ${version} and its package says ${packaged}`);
  const tag = `contracts-v${version}`;
  if (version.includes("-")) return { publish: false, tag, why: "a draft is never published" };
  if (releases.includes(tag)) return { publish: false, tag, why: `${tag} is out, and what is out is never replaced` };
  if (tags.includes(tag)) throw new Error(`the tag ${tag} is there with no release: look at it by hand before anything goes on it`);
  return { publish: true, tag, why: `${tag} goes out` };
};

const lines = (said) => (said ?? "").split(/\r?\n/).filter(Boolean);

const atRevision = (revision, path) => execFileSync("git", ["show", `${revision}:${path}`], { encoding: "utf8" }).trim();

const publishedPaths = (revision) => {
  const { files } = JSON.parse(atRevision(revision, `${PACKAGE}/package.json`));
  return [...[...files, "package.json"].map((one) => `${PACKAGE}/${one}`), ...ALSO_PUBLISHED];
};

const pullRequest = (base, head) => {
  const paths = [...new Set([...publishedPaths(base), ...publishedPaths(head)])];
  const differs = spawnSync("git", ["diff", "--quiet", base, head, "--", ...paths]);
  if (differs.status !== 0 && differs.status !== 1) throw new Error(`git diff could not compare ${base} and ${head}`);
  const { wrong, reminders } = judged({
    was: atRevision(base, "contracts/VERSION"),
    now: atRevision(head, "contracts/VERSION"),
    changed: differs.status === 1,
  });
  for (const one of reminders) console.log(`::warning::${one}`);
  for (const one of wrong) console.log(`::error::${one}`);
  process.exitCode = wrong.length ? 1 : 0;
};

const plan = () => {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const { publish, tag, why } = planned({
    version: readFileSync(join(root, "contracts/VERSION"), "utf8").trim(),
    packaged: JSON.parse(readFileSync(join(root, PACKAGE, "package.json"), "utf8")).version,
    releases: lines(process.env.RELEASES),
    tags: lines(process.env.TAGS),
  });
  console.log(why);
  appendFileSync(process.env.GITHUB_OUTPUT, `publish=${publish}\ntag=${tag}\n`);
};

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [what, ...rest] = process.argv.slice(2);
  if (what === "pr") pullRequest(...rest);
  else if (what === "plan") plan();
  else throw new Error("say pr <base> <head>, or plan");
}
