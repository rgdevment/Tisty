import { parseArgs } from "node:util";
import { chromium, webkit } from "playwright";
import { launched, opened, sheetOf } from "./engines/hands.mjs";
import { INVARIANT, walked } from "./engines/oracle.mjs";
import { served } from "./engines/served.mjs";
import { STARTS } from "./engines/starts.mjs";
import { stepsFor } from "./engines/steps.mjs";

const SEEDS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
const REPLAYS = [
  { seed: 7, steps: [["click", 0.5, 0.5], ["mark", "#ab "], ["beside", 0], ["press", "Backspace"]] },
];
const STEPS = 80;
const SHRINK_MS = 240_000;
const ENGINES = { webkit: ["WebKit", webkit], chromium: ["Chromium", chromium] };

const { values: asked } = parseArgs({
  options: {
    seed: { type: "string" },
    runs: { type: "string" },
    steps: { type: "string" },
    engine: { type: "string" },
    replay: { type: "string" },
  },
});

const counted = (name, fallback) => {
  const said = asked[name] ?? String(fallback);
  if (!/^\d+$/.test(said)) throw new Error(`--${name} takes a whole number, not ${said}`);
  return Number(said);
};

const startOf = (seed) => STARTS[seed % STARTS.length];

const walkOf = (seed) => ({ seed, steps: stepsFor(seed, startOf(seed), counted("steps", STEPS)) });

const walksAsked = () => {
  if (asked.replay !== undefined && asked.seed === undefined) throw new Error("--replay needs the --seed it was found with");
  if (asked.seed !== undefined) {
    const seed = counted("seed");
    return [asked.replay ? { seed, steps: JSON.parse(asked.replay), replayed: true } : walkOf(seed)];
  }
  if (asked.runs !== undefined)
    return Array.from({ length: counted("runs") }, () => walkOf(Math.floor(Math.random() * 1_000_000)));
  return [...REPLAYS.map((one) => ({ ...one, replayed: true })), ...SEEDS.map((seed) => walkOf(seed))];
};

const shrunk = async (replayed, steps, fault, until) => {
  const sameAgain = async (list) => {
    const again = await replayed(list);
    return again?.kind === fault.kind ? again : null;
  };
  const kept = steps[0]?.[0] === "click" ? 1 : 0;
  let small = steps.slice(0, fault.at + 1);
  let last = await sameAgain(small);
  if (!last) return null;
  small = small.slice(0, last.at + 1);
  for (let size = Math.ceil(small.length / 2); size >= 1 && Date.now() < until; size = Math.floor(size / 2)) {
    for (let at = kept; at + size <= small.length && Date.now() < until; ) {
      const without = [...small.slice(0, at), ...small.slice(at + size)];
      const again = await sameAgain(without);
      if (!again) {
        at += size;
        continue;
      }
      small = without.slice(0, again.at + 1);
      last = again;
    }
  }
  return { steps: small, fault: last };
};

const againOf = (name, { seed, steps, replayed }) =>
  `     again: node app/scripts/engines-fuzz.mjs --engine ${name.toLowerCase()} --seed ${seed} ${
    replayed ? `--replay '${JSON.stringify(steps)}'` : `--steps ${steps.length}`
  }`;

const toldBroken = (name, walk, fault) => {
  console.error(
    [
      `x  ${name} seed ${walk.seed} (${startOf(walk.seed).name}), step ${fault.at + 1} of ${walk.steps.length}: ${fault.kind}: ${fault.said}`,
      `     steps: ${JSON.stringify(walk.steps.slice(0, fault.at + 1))}`,
      againOf(name, walk),
    ].join("\n"),
  );
};

const toldShrunk = (name, seed, small) => {
  if (!small) return console.error(`   ${name} seed ${seed} did not fail the same way when replayed, so it was not shrunk`);
  console.error(
    [
      `   ${name} seed ${seed} shrunk to ${small.steps.length} steps: ${small.fault.kind}: ${small.fault.said}`,
      againOf(name, { seed, steps: small.steps, replayed: true }),
    ].join("\n"),
  );
};

const warmed = async (page, url) => {
  await sheetOf(page, STARTS.find((one) => one.name === "objects").doc);
  await page.waitForTimeout(3000);
  await opened(page, url);
};

const walkedAll = async (name, page, url, walks) => {
  await warmed(page, url);
  const replayed = (start) => async (list) => {
    const fault = await walked(page, start, list);
    if (fault) await opened(page, url);
    return fault;
  };
  const broken = [];
  let shrinkUntil = null;
  for (const walk of walks) {
    const start = startOf(walk.seed);
    const fault = await walked(page, start, walk.steps);
    if (!fault) {
      console.log(`ok ${name} ${walk.replayed ? "replay of " : ""}seed ${walk.seed} (${start.name}), ${walk.steps.length} steps`);
      continue;
    }
    broken.push(walk.seed);
    toldBroken(name, walk, fault);
    await opened(page, url);
    shrinkUntil ??= Date.now() + SHRINK_MS;
    if (Date.now() < shrinkUntil)
      toldShrunk(name, walk.seed, await shrunk(replayed(start), walk.steps, fault, shrinkUntil));
  }
  return broken;
};

const ranIn = async ([name, engine], walks) => {
  const server = await served();
  try {
    const { browser, page } = await launched(engine, server.url);
    try {
      return await walkedAll(name, page, server.url, walks);
    } finally {
      await browser.close();
    }
  } finally {
    await server.close();
  }
};

if (asked.engine !== undefined && !ENGINES[asked.engine])
  throw new Error(`--engine takes ${Object.keys(ENGINES).join(" or ")}, not ${asked.engine}`);
const engines = asked.engine ? [ENGINES[asked.engine]] : Object.values(ENGINES);
const walks = walksAsked();
const named = walks.map((one) => (one.replayed ? `${one.seed} (replayed, ${one.steps.length} steps)` : one.seed));
console.log(
  `seeds ${named.join(" ")} in ${engines.map(([name]) => name).join(" and ")}, ${asked.steps ?? STEPS} steps each`,
);
console.log(`each run checks that no page error is thrown, the window stays on its page, the page draws what the text says, and ${INVARIANT}`);
const settled = await Promise.allSettled(engines.map((one) => ranIn(one, walks)));
const failed = settled.flatMap((one) => (one.status === "fulfilled" ? one.value : [])).length;
const stopped = settled.filter((one) => one.status === "rejected");
const runs = walks.length * engines.length;
for (const [at, one] of settled.entries())
  if (one.status === "rejected") console.error(`x  ${engines[at][0]} stopped: ${one.reason?.stack ?? one.reason}`);
if (failed > 0) console.error(`\n${failed} of ${runs} runs broke the page, see the seeds and steps above`);
if (failed > 0 || stopped.length > 0) process.exitCode = 1;
else console.log(`ok all ${runs} runs keep their letters, draw what the text says and throw nothing`);
