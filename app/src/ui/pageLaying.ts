import { byteFrom, bytesIn, checkAfter, checkOf, type Placed, type Reckoned } from "../placing";
import type { Caret } from "./pageCaret";
import {
  applied,
  changeOf,
  type Edit,
  inWholeChars,
  landedAfter,
  linedAs,
  unsplitPairs,
} from "./pageEdits";
import { type Change, holdingOn } from "./pageHolding";
import { lateOn } from "./pageLate";
import { joinedApart } from "./pageLists";
import { opened } from "./pageNodes";
import { matched, patched } from "./pagePatching";
import { placesSettled } from "./pagePlaces";
import { recorded, type Writer } from "./pageWriter";

// What the rest of the window reads of the page catches up once the keys rest, not on each one.
const RESTED_MS = 250;
const NEAR_PATIENCE_MS = 3000;
const COMPOSING_PATIENCE_MS = 10_000;

const answered = <T>(asked: Promise<T>, within: number): Promise<T | null> =>
  new Promise((done) => {
    const late = setTimeout(() => done(null), within);
    const settle = (one: T | null) => {
      clearTimeout(late);
      done(one);
    };
    asked.then(settle, () => settle(null));
  });

const reported = (error: unknown) =>
  window.dispatchEvent(
    new ErrorEvent("error", {
      error,
      message: error instanceof Error ? error.message : String(error),
    }),
  );

export type Laying = ReturnType<typeof layingOn>;

export const layingOn = (
  writer: Writer,
  caret: Caret,
  drawn: (laid?: readonly Element[]) => void,
) => {
  const { page, hands } = writer;
  let laying: Promise<void> = Promise.resolve();
  let resting: ReturnType<typeof setTimeout> | undefined;
  let unsaid: string | null = null;
  let over = false;
  let unlaid = 0;
  let placed: Placed | null = null;
  // What the core holds while the browser writes ahead of it, with the check it holds it under.
  let behind: Reckoned | null = null;
  let catching: ReturnType<typeof setTimeout> | undefined;
  let drawing = 0;
  const holding = holdingOn(page);

  const reckonedOf = (text: string, unit: number): Reckoned => {
    if (writer.reckoned?.text === text) return writer.reckoned;
    const byte = bytesIn(text, 0, unit);
    return { text, check: checkOf(text, byte + bytesIn(text, unit)), unit, byte };
  };

  const redrawn = (change: Change) => {
    writer.reckoned = null;
    writer.whole = true;
    holding.before(change);
    return hands.redraw(hands.text());
  };

  const told = () => {
    clearTimeout(resting);
    placesSettled(page);
    if (unsaid !== null) hands.shown(unsaid);
    unsaid = null;
  };
  // The same page shows the next document, so a drawing still queued by this writer must never land on it.
  const stop = () => {
    over = true;
    clearTimeout(catching);
    holding.stop();
    told();
    return unlaid > 0;
  };
  const rested = (now: string) => {
    clearTimeout(resting);
    unsaid = now;
    resting = setTimeout(told, RESTED_MS);
  };

  const composed = async () => {
    const open = writer.composing;
    if (!open) return;
    const ended = await answered(
      open.over.then(() => true),
      COMPOSING_PATIENCE_MS,
    );
    if (!ended && writer.composing === open) writer.composing = null;
  };

  const layOne = async (was: string, now: string, asked: Edit) => {
    if (over) return;
    unsaid = null;
    clearTimeout(resting);
    const one = inWholeChars(was, asked);
    const removed = bytesIn(was, one.from, one.to);
    const added = bytesIn(one.insert);
    const by = added - removed;
    if (writer.whole) {
      const from = byteFrom(placed?.text === was ? placed : writer.reckoned, was, one.from);
      placed = { text: now, unit: one.from + one.insert.length, byte: from + added };
      await composed();
      if (over) return;
      return redrawn({ from, removed, by });
    }
    const known = reckonedOf(was, one.from);
    const from = byteFrom(known, was, one.from);
    const edit = { from, to: from + removed, insert: one.insert };
    const check = checkAfter(known.check, edit);
    writer.reckoned = {
      text: now,
      check,
      unit: one.from + one.insert.length,
      byte: from + added,
    };
    const sent = { body: now, was: known.check, now: check, edit };
    const near = await answered(hands.near(sent, from, from + added), NEAR_PATIENCE_MS);
    await composed();
    if (over) return;
    const sheet = page.querySelector(".page");
    const laid = sheet && near && patched(sheet, near, by, [edit.from, edit.to]);
    if (!sheet || !laid) return redrawn({ from, removed, by });
    hands.dressed(sheet, laid);
    opened(page, laid);
    rested(now);
    if (hands.text() === now) drawn(laid);
  };

  // The page already shows what the browser wrote: a block the core draws alike only takes its places.
  const matchOne = async (known: Reckoned, now: string, asked: Edit) => {
    if (over) return;
    const was = known.text;
    const one = inWholeChars(was, asked);
    const removed = bytesIn(was, one.from, one.to);
    const added = bytesIn(one.insert);
    const from = byteFrom(known, was, one.from);
    const edit = { from, to: from + removed, insert: one.insert };
    const check = checkAfter(known.check, edit);
    const held = { text: now, check, unit: one.from + one.insert.length, byte: from + added };
    // The core goes on from its own check, also when a slow key already waits to be told after this.
    if (hands.text() === now || writer.reckoned?.text === now) writer.reckoned = held;
    else if (behind?.text === now) behind = held;
    const sent = { body: now, was: known.check, now: check, edit };
    const near = await answered(hands.near(sent, from, from + added), NEAR_PATIENCE_MS);
    // A composition open over the page holds nodes of its own, which no drawing may take before it ends.
    await composed();
    if (over || hands.text() !== now) return;
    const sheet = page.querySelector(".page");
    if (sheet) placesSettled(page);
    // Blocks taken over lose the caret's nodes, so where it stood, or what was chosen, is laid again after.
    const stood = caret.caretSpan();
    const taken = sheet && near && matched(sheet, near);
    if (!sheet || !taken) {
      writer.kept = { node: page.body, offset: -1, at: caret.caretSpan()?.from ?? held.unit };
      return redrawn({ from, removed, by: added - removed });
    }
    if (taken.length) {
      if (stood) {
        const chosenFrom = stood.from === stood.to ? undefined : stood.from;
        writer.kept = { node: page.body, offset: -1, at: stood.to, chosenFrom };
      }
      hands.dressed(sheet, taken);
      opened(page, taken);
      drawn(taken);
    }
    rested(now);
  };

  const caughtUp = (upTo = hands.text()) => {
    clearTimeout(catching);
    const known = behind;
    behind = null;
    if (!known || over) return;
    // Keys that undid each other leave the text the core holds, and its check with it.
    if (known.text === upTo) {
      if (writer.reckoned?.text === upTo) writer.reckoned = known;
      return;
    }
    unlaid += 1;
    const step = laying
      .then(() => matchOne(known, upTo, changeOf(known.text, upTo)))
      .catch(recovered);
    laying = step.finally(() => {
      unlaid -= 1;
    });
  };

  /** A key the browser wrote is already in the words and on the page; the core hears of it once the keys rest. */
  const typedNatively = (was: string, now: string, one: Edit, from: number) => {
    const known = reckonedOf(was, one.from);
    behind ??= known;
    const edit = { from, to: from + bytesIn(was, one.from, one.to), insert: one.insert };
    writer.reckoned = {
      text: now,
      check: checkAfter(known.check, edit),
      unit: one.from + one.insert.length,
      byte: from + bytesIn(one.insert),
    };
    rested(now);
    clearTimeout(catching);
    catching = setTimeout(caughtUp, RESTED_MS);
  };

  const idle = () => drawing === 0;

  const recovered = (error: unknown) => {
    reported(error);
    if (over) return;
    placed = null;
    writer.reckoned = null;
    writer.whole = true;
    try {
      hands.redraw(hands.text());
    } catch (again) {
      reported(again);
    }
  };

  // Each change is laid in turn: the core draws again the blocks around it and only those change on the page.
  const laidOut = (was: string, now: string, asked: Edit) => {
    // A task's page is small and dressed with its hints and buttons, which only a whole drawing lays again.
    if (page.documentElement.classList.contains("untitled")) writer.whole = true;
    // A page left with nothing written waits at its title, which only a whole drawing lays.
    if (!now.trim()) writer.whole = true;
    caughtUp(was);
    unlaid += 1;
    drawing += 1;
    const step = laying.then(() => layOne(was, now, asked)).catch(recovered);
    laying = step.finally(() => {
      unlaid -= 1;
      drawing -= 1;
    });
  };

  const put = (wanted: Edit, at = wanted.from + wanted.insert.length, drawnWhole = false) => {
    const text = hands.text();
    const asked = unsplitPairs(text, wanted);
    const landing = landedAfter(wanted, asked, at);
    const [one, landed] = linedAs(text, ...joinedApart(text, asked, landing));
    const next = applied(text, one);
    if (next === text) return;
    const stood =
      writer.kept && writer.kept.offset < 0
        ? writer.kept.at
        : (caret.caretSpan()?.from ?? one.from);
    recorded(writer.turns, {
      from: one.from,
      removed: text.slice(one.from, one.to),
      added: one.insert,
      at: Math.min(stood, text.length),
      after: landed,
      typed: writer.typing ? Date.now() : undefined,
    });
    const shown = caret.chosen();
    writer.leftChosen =
      shown && !shown.collapsed
        ? {
            from: [shown.startContainer, shown.startOffset],
            to: [shown.endContainer, shown.endOffset],
          }
        : null;
    writer.kept = { node: page.body, offset: -1, at: landed };
    hands.write(next);
    if (drawnWhole) writer.whole = true;
    laidOut(text, next, one);
  };

  const heldAcross = (was: string, now: string) => {
    const one = changeOf(was, now);
    const removed = bytesIn(was, one.from, one.to);
    const from = bytesIn(was, 0, one.from);
    holding.before({ from, removed, by: bytesIn(one.insert) - removed });
  };

  const late = lateOn(writer, put, () => over);

  return {
    put,
    late,
    laidOut,
    stop,
    landed: holding.landed,
    heldAcross,
    typedNatively,
    caughtUp,
    idle,
  };
};
