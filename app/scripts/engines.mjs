import { chromium, webkit } from "playwright";
import { CASES, PIECES } from "./engines/cases.mjs";
import { changeOf, handsOn, launched, macIn, opened, sheetOf } from "./engines/hands.mjs";
import { served } from "./engines/served.mjs";

const SETTLED_MS = 600;
const NOISE = /^(window|page): (ResizeObserver loop|Script error\.)/;

const ENGINES = {
  webkit: { name: "WebKit", type: webkit },
  chromium: { name: "Chromium", type: chromium },
  edge: { name: "Edge", type: chromium, launch: { channel: "msedge" } },
};
const asked = process.argv.slice(2);
const unknown = asked.filter((one) => !Object.hasOwn(ENGINES, one));
if (unknown.length > 0) {
  console.error(`no engine called ${unknown.join(", ")}: the engines are ${Object.keys(ENGINES).join(", ")}`);
  process.exit(2);
}
const chosen = (asked.length > 0 ? asked : ["webkit", "chromium"]).map((one) => ENGINES[one]);

const listening = (page) => page.evaluate(() => window.engines.hear());

const heardSaid = (heard) =>
  heard.map(({ inputType, targets, chosen, caretAt }) => {
    const point = ({ node, offset, source }) => `${node}@${offset}=${source}`;
    const range = (one) => (one ? `${point(one.from)} .. ${point(one.to)}` : "none");
    return `${inputType}: targets [${targets.map(range).join("; ")}], chosen ${range(chosen)}, caret at ${caretAt}`;
  });

const ranIn = async (engine, url) => {
  const { browser, page } = await launched(engine.type, url, engine.launch);
  try {
    const mac = await macIn(page);
    const agent = await page.evaluate(() => navigator.userAgent);
    await page.evaluate((doc) => window.engines.open(doc), PIECES);
    await page.waitForTimeout(3000);
    await opened(page, url);
    await listening(page);
    const results = new Map();
    for (const one of CASES) {
      if (one.macOnly && !mac) continue;
      const frame = await sheetOf(page, one.doc);
      await page.waitForTimeout(400);
      let threw = null;
      try {
        await one.run(handsOn(page, frame));
      } catch (error) {
        threw = error.message;
      }
      await page.waitForTimeout(SETTLED_MS);
      if (!(await page.evaluate(() => Boolean(window.engines)).catch(() => false))) {
        results.set(one.name, { was: null, now: null, errors: [], fits: "ok", heard: [], threw: "the window walked back to another page" });
        await opened(page, url);
        await listening(page);
        continue;
      }
      const text = await page.evaluate(() => window.engines.text());
      const errors = (await page.evaluate(() => window.engines.errors())).filter((said) => !NOISE.test(said));
      const fits = await page.evaluate(() => window.engines.consistent());
      const heard = await page.evaluate(() => window.engines.heard());
      results.set(one.name, { ...changeOf(one.doc, text), errors, fits, heard, threw });
    }
    return { engine, mac, agent, results };
  } finally {
    await browser.close();
  }
};

const faultsOf = (one, name, result, agent) => {
  const faults = [];
  if (result.threw) faults.push(`${name} could not run the case: ${result.threw}`);
  if (result.was !== one.was || result.now !== one.now) {
    faults.push(
      `${name} took ${JSON.stringify(result.was)} and wrote ${JSON.stringify(result.now)}, not ${JSON.stringify(one.was)} and ${JSON.stringify(one.now)}`,
    );
    faults.push(`${name} runs as ${agent}`);
    for (const said of heardSaid(result.heard)) faults.push(`${name} heard ${said}`);
  }
  if (result.fits !== "ok") faults.push(`${name}: ${result.fits}`);
  for (const error of result.errors) faults.push(`${name} met an error: ${error.slice(0, 300)}`);
  return faults;
};

const toldWhereRan = (runs) => {
  for (const { engine, mac, results } of runs) {
    console.log(`${engine.name} on ${mac ? "macOS" : "Windows"} ran ${results.size} of ${CASES.length} cases`);
    for (const one of CASES.filter((left) => !results.has(left.name)))
      console.log(`   left out ${one.name}, Mac only: ${one.macOnly}`);
  }
};

const judged = (runs, stopped) => {
  const names = runs.map(({ engine }) => engine.name);
  let failed = 0;
  for (const one of CASES) {
    const faults = runs.flatMap(({ engine, agent, results }) =>
      results.has(one.name) ? faultsOf(one, engine.name, results.get(one.name), agent) : [],
    );
    if (faults.length === 0) continue;
    failed++;
    console.error(`x  ${one.name}`);
    for (const fault of faults) console.error(`     ${fault}`);
  }
  const ran = CASES.filter((one) => runs.some(({ results }) => results.has(one.name))).length;
  if (failed > 0) {
    console.error(`\n${failed} of ${ran} cases write differently from what they should, in ${names.join(" or in ")}`);
    process.exitCode = 1;
    return;
  }
  if (stopped.length > 0) {
    console.error(`\nthe ${ran} cases that ran in ${names.join(" and ")} wrote as they should, but ${stopped.join(" and ")} stopped`);
    return;
  }
  console.log(`ok all ${ran} cases write ${names.length > 1 ? "the same " : ""}in ${names.join(" and ")}, as they should`);
};

const ranApart = async (engine) => {
  const server = await served();
  try {
    return await ranIn(engine, server.url);
  } finally {
    await server.close();
  }
};

const settled = await Promise.allSettled(chosen.map(ranApart));
const stopped = settled.flatMap((one, at) => (one.status === "rejected" ? [[chosen[at], one.reason]] : []));
for (const [engine, why] of stopped) {
  const [headline, ...told] = String(why?.message ?? why).split("\n");
  console.error(`${engine.name} stopped: ${headline}`);
  for (const line of told) console.error(`     ${line}`);
}
const runs = settled.flatMap((one) => (one.status === "fulfilled" ? [one.value] : []));
if (runs.length > 0) {
  toldWhereRan(runs);
  judged(runs, stopped.map(([engine]) => engine.name));
}
if (stopped.length > 0) process.exitCode = 1;
