import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const MOST_RELEASES = 10;
const MOST_ON_A_PULL_REQUEST = 6;
const PULL_REQUEST_SYSTEM = "windows-latest";
const EVERY_SYSTEM = ["windows-latest", "macos-latest"];

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
  if (!releases.at(-1)?.every_pull_request) {
    wrong.push("the newest earlier release is the last one listed, and every pull request meets it");
  }
  return wrong;
};

const withoutItsTurn = ({ every_pull_request, ...one }) => one;

export const metOn = (event, releases) =>
  event === "pull_request"
    ? releases.filter((one) => one.every_pull_request).map((one) => ({ ...withoutItsTurn(one), os: PULL_REQUEST_SYSTEM }))
    : releases.flatMap((one) => EVERY_SYSTEM.map((os) => ({ ...withoutItsTurn(one), os })));

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
