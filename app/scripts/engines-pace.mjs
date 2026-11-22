import { chromium, webkit } from "playwright";
import { PIECES } from "./engines/cases.mjs";
import { handsOn, launched, opened, sheetOf } from "./engines/hands.mjs";
import { pageOf, SPOTS } from "./engines/lengthy.mjs";
import { calmed, keyed, watched } from "./engines/paced.mjs";
import { served } from "./engines/served.mjs";

const ROUNDS = 3;
const WARMED_MS = 3000;
const OPENED_CALM_MS = 500;
const CLICKED_CALM_MS = 400;
const NOISE = /^(window|page): (ResizeObserver loop|Script error\.)/;
const WORD = process.platform === "darwin" ? "Alt+Backspace" : "Control+Backspace";
const DOCS = { short: pageOf(1), long: pageOf(100) };

const many = (times, kind, key) => Array.from({ length: times }, () => [kind, key]);
const letters = (said) => [...said].map((one) => ["letter", one]);
const STEPS = [
  ...letters(" uno dos tres cuatro"),
  ...many(4, "word", WORD),
  ...letters(" cinco seis"),
  ...many(10, "backspace", "Backspace"),
  ...[1, 2, 3, 4].flatMap(() => [["enter", "Enter"], ...letters("nuevo")]),
  ...many(12, "backspace", "Backspace"),
];
const FIRST_WORD = [["split", "Enter"], ...many(2, "move", "ArrowRight"), ["first word", WORD]];
const KINDS = ["letter", "enter", "backspace", "word", "split", "first word"];

const median = (keys, measure) => {
  const sorted = keys.map((one) => one[measure]).sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)];
};
const most = (keys, measure) => Math.max(...keys.map((one) => one[measure]));

const PAGE_BLOCKS = { letter: 1, enter: 2, backspace: 1, word: 1, split: 2, "first word": 1 };
const BUDGETS = [
  {
    what: "ms of work on the long page (median)",
    value: (short, long) => median(long, "work"),
    ceiling: (short) => 3 * median(short, "work") + 30,
    rule: "3 x the short page + 30",
  },
  {
    what: "ms to the next frame on the long page (median)",
    value: (short, long) => median(long, "ms"),
    ceiling: () => 150,
    rule: "a ceiling",
  },
  {
    what: "bytes sent to the core in one key (max)",
    value: (short, long) => Math.max(most(short, "sent"), most(long, "sent")),
    ceiling: () => 1024,
    rule: "the key, never the document",
  },
  {
    what: "whole drawings of the page in one key (max)",
    value: (short, long) => Math.max(most(short, "whole"), most(long, "whole")),
    ceiling: () => 0,
    rule: "none",
  },
  {
    what: "blocks of the page put in or taken out by a key on the long page (median)",
    value: (short, long) => median(long, "blocks"),
    ceiling: (short, kind) => PAGE_BLOCKS[kind],
    rule: "the blocks the key itself makes or joins",
  },
  {
    what: "nodes put in or taken out by a key on the long page (median)",
    value: (short, long) => median(long, "nodes"),
    ceiling: (short) => 2 * median(short, "nodes") + 10,
    rule: "2 x the short page + 10",
  },
  {
    what: "attributes written by a key on the long page (median)",
    value: (short, long) => median(long, "attributes"),
    ceiling: (short) => 2 * median(short, "attributes") + 10,
    rule: "2 x the short page + 10",
  },
];

const APART = /^the page shows \[(.*)\] but the text draws \[(.*)\]$/;

const apart = (fits) => {
  const [, shown, drawn] = APART.exec(fits) ?? [];
  if (shown === undefined) return fits;
  const was = shown.split(" ");
  const now = drawn.split(" ");
  const places = Array.from({ length: Math.max(was.length, now.length) }, (_, place) => place);
  const at = places.find((place) => was[place] !== now[place]);
  const near = (all) => all.slice(Math.max(0, at - 2), at + 3).join(" ");
  return `block ${at} is out of place (${was.length} on the page, ${now.length} in the text): the page shows ${near(was)} where the text draws ${near(now)}`;
};

const typedAt = async (page, name, doc, spot) => {
  const restless = [];
  const still = async (ms, when) => {
    if (!(await calmed(page, ms))) restless.push(`the ${name} page never stayed still for ${ms} ms ${when}`);
  };
  const frame = await sheetOf(page, doc);
  const hands = handsOn(page, frame);
  const keys = [];
  const typed = async (steps) => {
    for (const [kind, step] of steps) {
      if (kind === "move") {
        await page.keyboard.press(step);
        await still(CLICKED_CALM_MS, `once ${step} moved the caret`);
        continue;
      }
      const one = { doc: name, kind, step, ...(await keyed(page, step)) };
      keys.push(one);
      if (one.unsettled) return false;
    }
    return true;
  };
  await watched(page);
  await still(OPENED_CALM_MS, "once opened");
  await hands.clickAt(spot.words);
  await still(CLICKED_CALM_MS, "once clicked");
  const unopened = [];
  if (await typed(STEPS)) {
    await hands.clickAt(spot.lead, true);
    await still(CLICKED_CALM_MS, "once clicked before the words that will open a paragraph");
    const deleted = await typed(FIRST_WORD);
    const text = await page.evaluate(() => window.engines.text());
    if (deleted && !text.includes(`\n ${spot.words}`))
      unopened.push(`the first word deleted on the ${name} page left no paragraph opening with " ${spot.words}"`);
  }
  await still(CLICKED_CALM_MS, "once the keys rested");
  const fits = await page.evaluate(() => window.engines.consistent());
  const errors = (await page.evaluate(() => window.engines.errors())).filter((said) => !NOISE.test(said));
  return {
    keys: keys.filter((one) => !one.unsettled),
    faults: [
      ...restless,
      ...unopened,
      ...errors.map((said) => `the ${name} page met an error: ${said.slice(0, 300)}`),
      ...keys
        .filter((one) => one.unsettled)
        .map((one) => `a ${one.kind} key (${one.step}) on the ${name} page never settled, so its spot stopped there`),
      ...(fits === "ok" ? [] : [`once the keys rested the ${name} page did not match its text: ${apart(fits)}`]),
    ],
  };
};

const ranIn = async (engine, url) => {
  const { browser, page } = await launched(engine, url);
  await page.evaluate((doc) => window.engines.open(doc), PIECES);
  await page.waitForTimeout(WARMED_MS);
  await opened(page, url);
  const ran = { keys: [], faults: [] };
  try {
    for (let round = 0; round < ROUNDS; round++) {
      for (const [name, doc] of Object.entries(DOCS)) {
        for (const spot of Object.values(SPOTS)) {
          const typed = await typedAt(page, name, doc, spot);
          for (const part of Object.keys(ran)) ran[part].push(...typed[part]);
        }
      }
    }
  } finally {
    await browser.close();
  }
  return ran;
};

const shown = (value) => (Math.abs(value - Math.round(value)) < 0.05 ? String(Math.round(value)) : value.toFixed(1));

const printed = (name, keys) => {
  const columns = ["frame ms", "work ms", "core ms", "sent B", "whole", "blocks", "nodes", "attrs"];
  console.log(`${name}: medians per key (sent and whole: the most in one key)`);
  console.log(`  ${"".padEnd(16)}${columns.map((one) => one.padStart(9)).join("")}`);
  for (const kind of KINDS) {
    for (const doc of Object.keys(DOCS)) {
      const some = keys.filter((one) => one.kind === kind && one.doc === doc);
      if (!some.length) {
        console.log(`  ${`${kind} ${doc}`.padEnd(16)}no key settled`);
        continue;
      }
      const values = [
        median(some, "ms"),
        median(some, "work"),
        median(some, "core"),
        most(some, "sent"),
        most(some, "whole"),
        median(some, "blocks"),
        median(some, "nodes"),
        median(some, "attributes"),
      ];
      console.log(`  ${`${kind} ${doc}`.padEnd(16)}${values.map((one) => shown(one).padStart(9)).join("")}`);
    }
  }
};

const overBudget = (name, keys) =>
  KINDS.flatMap((kind) => {
    const short = keys.filter((one) => one.kind === kind && one.doc === "short");
    const long = keys.filter((one) => one.kind === kind && one.doc === "long");
    if (!short.length || !long.length) return [`${name} ${kind}: no key of this kind was measured`];
    return BUDGETS.flatMap((budget) => {
      const value = budget.value(short, long);
      const ceiling = budget.ceiling(short, kind);
      if (value <= ceiling) return [];
      return [`${name} ${kind}: ${budget.what} is ${shown(value)}, over ${shown(ceiling)} (${budget.rule})`];
    });
  });

const server = await served();
try {
  const size = (doc) => `${Math.round(Buffer.byteLength(doc) / 1024)} KB`;
  console.log(`a short page of ${size(DOCS.short)} and a long one of ${size(DOCS.long)}, ${ROUNDS} rounds each`);
  const faults = [];
  for (const [name, engine] of [
    ["WebKit", webkit],
    ["Chromium", chromium],
  ]) {
    const ran = await ranIn(engine, server.url);
    printed(name, ran.keys);
    faults.push(...ran.faults.map((one) => `${name}: ${one}`), ...overBudget(name, ran.keys));
  }
  if (faults.length > 0) {
    for (const fault of faults) console.error(`x  ${fault}`);
    console.error(`\ntyping on a long page broke ${faults.length} of its budgets, in WebKit or in Chromium`);
    process.exitCode = 1;
  } else {
    console.log("ok typing and deleting on a long page keeps within its budgets in WebKit and Chromium");
  }
} finally {
  await server.close();
}
