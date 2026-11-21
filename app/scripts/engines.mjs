import { chromium, webkit } from "playwright";
import { CASES, PIECES } from "./engines/cases.mjs";
import { served } from "./engines/served.mjs";

const KEY_MS = 80;
const SETTLED_MS = 600;
const NOISE = /^(window|page): (ResizeObserver loop|Script error\.)/;

const changeOf = (was, now) => {
  let from = 0;
  while (from < was.length && was[from] === now[from]) from++;
  let back = 0;
  while (back < was.length - from && was[was.length - 1 - back] === now[now.length - 1 - back]) back++;
  return { was: was.slice(from, was.length - back), now: now.slice(from, now.length - back) };
};

const handsOn = (page, frame) => {
  const clickAt = async (words, before = false) => {
    const spot = await frame.evaluate(
      ([said, early]) => {
        const walk = document.createTreeWalker(document.querySelector(".page"), NodeFilter.SHOW_TEXT);
        for (let node = walk.nextNode(); node; node = walk.nextNode()) {
          const at = node.data.indexOf(said);
          if (at < 0) continue;
          const shown = node.parentElement.getBoundingClientRect();
          if (shown.top < 0 || shown.bottom > innerHeight) node.parentElement.scrollIntoView({ block: "center" });
          const range = document.createRange();
          const end = early ? at : at + said.length;
          range.setStart(node, early ? end : end - 1);
          range.setEnd(node, early ? end + 1 : end);
          const box = range.getBoundingClientRect();
          return { x: early ? box.left + 0.5 : box.right - 0.5, y: box.top + box.height / 2 };
        }
        return null;
      },
      [words, before],
    );
    if (!spot) throw new Error(`no "${words}" on the page`);
    const sheet = await (await frame.frameElement()).boundingBox();
    await page.mouse.click(sheet.x + spot.x, sheet.y + spot.y);
    await page.waitForTimeout(150);
  };
  const typed = async (said) => {
    for (const letter of said) {
      await page.keyboard.type(letter);
      await page.waitForTimeout(KEY_MS);
    }
  };
  const press = async (key, many = 1) => {
    for (let time = 0; time < many; time++) {
      await page.keyboard.press(key);
      await page.waitForTimeout(KEY_MS);
    }
  };
  return { clickAt, typed, press, page, frame };
};

const opened = async (page, url) => {
  await page.goto(url);
  await page.waitForFunction(() => window.enginesReady === true, null, { timeout: 120_000 });
};

const ranIn = async (engine, url) => {
  const browser = await engine.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 1000, height: 1400 } });
  await opened(page, url);
  await page.evaluate((doc) => window.engines.open(doc), PIECES);
  await page.waitForTimeout(3000);
  await opened(page, url);
  const results = new Map();
  for (const one of CASES) {
    await page.evaluate((doc) => window.engines.open(doc), one.doc);
    const frame = page.frames().find((other) => other !== page.mainFrame() && !other.isDetached());
    await page.waitForTimeout(400);
    let threw = null;
    try {
      await one.run(handsOn(page, frame));
    } catch (error) {
      threw = error.message;
    }
    await page.waitForTimeout(SETTLED_MS);
    if (!(await page.evaluate(() => Boolean(window.engines)).catch(() => false))) {
      results.set(one.name, { was: null, now: null, errors: [], fits: "ok", threw: "the window walked back to another page" });
      await opened(page, url);
      continue;
    }
    const text = await page.evaluate(() => window.engines.text());
    const errors = (await page.evaluate(() => window.engines.errors())).filter((said) => !NOISE.test(said));
    const fits = await page.evaluate(() => window.engines.consistent());
    results.set(one.name, { ...changeOf(one.doc, text), errors, fits, threw });
  }
  await browser.close();
  return results;
};

const faultsOf = (one, name, result) => {
  const faults = [];
  if (result.threw) faults.push(`${name} could not run the case: ${result.threw}`);
  if (result.was !== one.was || result.now !== one.now)
    faults.push(
      `${name} took ${JSON.stringify(result.was)} and wrote ${JSON.stringify(result.now)}, not ${JSON.stringify(one.was)} and ${JSON.stringify(one.now)}`,
    );
  if (result.fits !== "ok") faults.push(`${name}: ${result.fits}`);
  for (const error of result.errors) faults.push(`${name} met an error: ${error.slice(0, 300)}`);
  return faults;
};

const server = await served();
try {
  const [inWebKit, inChromium] = await Promise.all([ranIn(webkit, server.url), ranIn(chromium, server.url)]);
  let failed = 0;
  for (const one of CASES) {
    const faults = [
      ...faultsOf(one, "WebKit", inWebKit.get(one.name)),
      ...faultsOf(one, "Chromium", inChromium.get(one.name)),
    ];
    if (faults.length === 0) continue;
    failed++;
    console.error(`x  ${one.name}`);
    for (const fault of faults) console.error(`     ${fault}`);
  }
  if (failed > 0) {
    console.error(`\n${failed} of ${CASES.length} cases write differently from what they should, in WebKit or in Chromium`);
    process.exitCode = 1;
  } else {
    console.log(`ok all ${CASES.length} cases write the same in WebKit and Chromium, as they should`);
  }
} finally {
  await server.close();
}
