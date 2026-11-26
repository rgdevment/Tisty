import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const MOST_RELEASES = 10;
const MOST_ON_A_PULL_REQUEST = 2;
const PULL_REQUEST_SYSTEM = "windows-latest";
const EVERY_SYSTEM = ["windows-latest", "macos-latest"];
const PINNED_AS = { "windows-latest": "windows", "macos-latest": "macos" };
const SHA256 = /^[0-9a-f]{64}$/;

const pinnedEverywhere = ({ sha256 = {} }) => Object.values(PINNED_AS).every((system) => SHA256.test(sha256[system] ?? ""));

export const wrongWith = (releases) => {
  const wrong = [];
  const onPullRequests = releases.filter((one) => one.every_pull_request);
  if (releases.length > MOST_RELEASES) {
    wrong.push(`${releases.length} earlier releases are met, and at most ${MOST_RELEASES} are: a new one takes the place of an older one`);
  }
  if (onPullRequests.length > MOST_ON_A_PULL_REQUEST) {
    wrong.push(`${onPullRequests.length} earlier releases are met on every pull request, and at most ${MOST_ON_A_PULL_REQUEST} are`);
  }
  if (new Set(releases.map((one) => one.release)).size !== releases.length) {
    wrong.push("an earlier release is listed twice");
  }
  if (!releases.at(0)?.every_pull_request) {
    wrong.push("the oldest earlier release is the first one listed, and every pull request meets it");
  }
  if (!releases.at(-1)?.every_pull_request) {
    wrong.push("the newest earlier release is the last one listed, and every pull request meets it");
  }
  for (const one of releases.filter((one) => !one.built && !pinnedEverywhere(one))) {
    wrong.push(`${one.release} is downloaded, so it pins the sha256 of what it downloads for ${Object.values(PINNED_AS).join(" and ")}, 64 lowercase hex digits each`);
  }
  return wrong;
};

const asLine = (os) => ({ release, schema, shape, accepts = "", built = false, sha256 = {} }) =>
  [release, schema, shape, accepts, built ? "built" : "", sha256[PINNED_AS[os]] ?? ""].join("|");

export const metOn = (event, releases) => {
  const onPullRequest = event === "pull_request";
  const met = onPullRequest ? releases.filter((one) => one.every_pull_request) : releases;
  const systems = onPullRequest ? [PULL_REQUEST_SYSTEM] : EVERY_SYSTEM;
  const downloaded = releases.filter((one) => !one.built).map((one) => one.release).join("_");
  return systems.map((os) => ({ os, releases: met.map(asLine(os)).join("\n"), downloaded }));
};

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const releases = JSON.parse(readFileSync(new URL("../.github/earlier-releases.json", import.meta.url), "utf8"));
  const [what, event] = process.argv.slice(2);
  if (what === "matrix") {
    console.log(`met=${JSON.stringify(metOn(event, releases))}`);
  } else if (what === "check") {
    const wrong = wrongWith(releases);
    for (const one of wrong) console.error(one);
    process.exitCode = wrong.length ? 1 : 0;
  } else {
    throw new Error("say matrix <event>, or check");
  }
}
