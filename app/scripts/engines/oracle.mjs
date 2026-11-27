import { changeOf, handsOn, sheetOf } from "./hands.mjs";
import { deletes, taken } from "./steps.mjs";

export const INVARIANT =
  "after every step that is not Backspace, Delete, a word or line delete, undo, redo or a key over a selection, " +
  "every letter of the Markdown before the step is still in it, in the same order; " +
  "a typed space with nothing selected grows the Markdown by exactly one space; " +
  "Enter on a blank top-level line adds one block to the page";

const NOISE = /^(window|page): (ResizeObserver loop|Script error\.)/;
const OPENED_MS = 400;
const DRAWN_EVERY = 5;
const SETTLE_MS = 3000;

// A kept blank line is `&nbsp;` and the x of a done step's box is its mark, neither a letter written:
// the first key written over a blank line takes its `&nbsp;` away, and an empty step ended by Enter takes its box.
const lettersOf = (text) =>
  text.replace(/&nbsp;/g, "").replace(/^([ \t>]*[-*+][ \t]+)\[[xX]\]/gm, "$1[ ]").normalize("NFC").match(/\p{L}/gu) ?? [];

const lettersLost = (was, now) => {
  const before = lettersOf(was);
  let at = 0;
  for (const one of lettersOf(now)) if (one === before[at]) at++;
  if (at === before.length) return null;
  const change = changeOf(was, now);
  return `lost "${before[at]}": took ${JSON.stringify(change.was)} and wrote ${JSON.stringify(change.now)}`;
};

const spacesOf = (text) => (text.match(/ /g) ?? []).length;

const spaceCollapsed = (was, now, step) => {
  if (step[0] !== "type" || step[1] !== " " || was.spread) return null;
  const grew = spacesOf(now.text) - spacesOf(was.text);
  return grew === 1 ? null : `a typed space grew the source by ${grew} spaces, not 1`;
};

const blockAdded = async (page, frame, was, step) => {
  if (step[0] !== "press" || step[1] !== "Enter" || !was.onBlankLine || was.spread) return null;
  await settled(page);
  const after = await stateOf(page, frame);
  if (after.walked || after.blocks === undefined) return null;
  return after.blocks === was.blocks + 1
    ? null
    : `Enter on a blank top-level line left the page with ${after.blocks} blocks, not ${was.blocks + 1}`;
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
  const lines = await frame
    .evaluate(() => ({
      blocks: document.querySelectorAll(".page > [data-at]").length,
      onBlankLine: Boolean(getSelection().anchorNode?.parentElement?.closest(".page > [data-ghost]")),
    }))
    .catch(() => ({}));
  return { text: held.text, errors: held.errors.filter((said) => !NOISE.test(said)), spread, ...lines };
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
  const squeezed = spaceCollapsed(was, now, step);
  if (squeezed) return { fault: { at, kind: "spaces", said: squeezed } };
  const unmade = await blockAdded(page, frame, was, step);
  if (unmade) return { fault: { at, kind: "blocks", said: unmade } };
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
