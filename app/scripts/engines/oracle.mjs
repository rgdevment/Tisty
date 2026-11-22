import { changeOf, handsOn, sheetOf } from "./hands.mjs";
import { deletes, taken } from "./steps.mjs";

export const INVARIANT =
  "after every step that is not Backspace, Delete, a word or line delete, undo, redo or a key over a selection, " +
  "every letter of the Markdown before the step is still in it, in the same order";

const NOISE = /^(window|page): (ResizeObserver loop|Script error\.)/;
const OPENED_MS = 400;
const DRAWN_EVERY = 5;
const SETTLE_MS = 3000;

const lettersOf = (text) => text.normalize("NFC").match(/\p{L}/gu) ?? [];

const lettersLost = (was, now) => {
  const before = lettersOf(was);
  let at = 0;
  for (const one of lettersOf(now)) if (one === before[at]) at++;
  if (at === before.length) return null;
  const change = changeOf(was, now);
  return `lost "${before[at]}": took ${JSON.stringify(change.was)} and wrote ${JSON.stringify(change.now)}`;
};

const stateOf = async (page, frame) => {
  const held = await page
    .evaluate(() => (window.engines ? { text: window.engines.text(), errors: window.engines.errors() } : null))
    .catch(() => null);
  if (!held) return { walked: "the window walked away to another page" };
  if (frame.isDetached()) return { walked: "the sheet left the window" };
  if (frame.url() !== "about:srcdoc") return { walked: `the sheet walked away to ${frame.url()}` };
  const spread = await frame
    .evaluate(() => !getSelection().isCollapsed || Boolean(document.querySelector(".chosen-block")))
    .catch(() => true);
  return { text: held.text, errors: held.errors.filter((said) => !NOISE.test(said)), spread };
};

const settled = async (page) => {
  const until = Date.now() + SETTLE_MS;
  for (;;) {
    const fits = await page.evaluate(() => window.engines.consistent()).catch((error) => error.message);
    if (fits === "ok" || Date.now() > until) return fits;
    await page.waitForTimeout(100);
  }
};

const brokenIn = (now, at) => {
  if (now.walked) return { at, kind: "walked", said: now.walked };
  if (now.errors.length) return { at, kind: "error", said: now.errors[0].slice(0, 400) };
  return null;
};

const faultAfter = async (page, frame, was, step, at, last) => {
  const now = await stateOf(page, frame);
  const broken = brokenIn(now, at);
  if (broken) return { fault: broken };
  const keeps = !was.spread && !deletes(step);
  const lost = keeps && lettersLost(was.text, now.text);
  if (lost) return { fault: { at, kind: "letters", said: lost } };
  if ((at + 1) % DRAWN_EVERY !== 0 && !last) return { now };
  const fits = await settled(page);
  const after = await stateOf(page, frame);
  const late = brokenIn(after, at);
  if (late) return { fault: late };
  const lostLate = keeps && lettersLost(was.text, after.text);
  if (lostLate) return { fault: { at, kind: "letters", said: lostLate } };
  return fits === "ok" ? { now: after } : { fault: { at, kind: "drawn", said: fits } };
};

export const walked = async (page, start, steps) => {
  const frame = await sheetOf(page, start.doc);
  await page.waitForTimeout(OPENED_MS);
  const hands = handsOn(page, frame);
  let was = await stateOf(page, frame);
  const opening = brokenIn(was, -1);
  if (opening) return { ...opening, said: `while the page opened: ${opening.said}` };
  for (const [at, step] of steps.entries()) {
    try {
      await taken(hands, step);
    } catch (error) {
      return { at, kind: "threw", said: error.message.split("\n")[0] };
    }
    const { fault, now } = await faultAfter(page, frame, was, step, at, at === steps.length - 1);
    if (fault) return fault;
    was = now;
  }
  return null;
};
