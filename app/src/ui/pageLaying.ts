import { byteFrom, bytesIn, checkAfter, checkOf, type Reckoned } from "../placing";
import type { Caret } from "./pageCaret";
import { applied, type Edit, inWholeChars, linedAs } from "./pageEdits";
import { opened } from "./pageNodes";
import { patched } from "./pagePatching";
import { placesSettled } from "./pagePlaces";
import { recorded, type Writer } from "./pageWriter";

// What the rest of the window reads of the page catches up once the keys rest, not on each one.
const RESTED_MS = 250;

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

  const reckonedOf = (text: string, unit: number): Reckoned => {
    if (writer.reckoned?.text === text) return writer.reckoned;
    const byte = bytesIn(text, 0, unit);
    return { text, check: checkOf(text, byte + bytesIn(text, unit)), unit, byte };
  };

  const redrawn = () => {
    writer.reckoned = null;
    writer.whole = true;
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
    told();
  };
  const rested = (now: string) => {
    clearTimeout(resting);
    unsaid = now;
    resting = setTimeout(told, RESTED_MS);
  };

  // Each change is laid in turn: the core draws again the blocks around it and only those change on the page.
  const laidOut = (was: string, now: string, asked: Edit) => {
    // A task's page is small and dressed with its hints and buttons, which only a whole drawing lays again.
    if (page.documentElement.classList.contains("untitled")) writer.whole = true;
    laying = laying.then(async () => {
      if (over) return;
      unsaid = null;
      clearTimeout(resting);
      if (writer.whole) return redrawn();
      const one = inWholeChars(was, asked);
      const known = reckonedOf(was, one.from);
      const from = byteFrom(known, was, one.from);
      const removed = bytesIn(was, one.from, one.to);
      const added = bytesIn(one.insert);
      const edit = { from, to: from + removed, insert: one.insert };
      const check = checkAfter(known.check, edit);
      writer.reckoned = {
        text: now,
        check,
        unit: one.from + one.insert.length,
        byte: from + added,
      };
      const sent = { body: now, was: known.check, now: check, edit };
      const near = await hands.near(sent, from, from + added).catch(() => null);
      if (over) return;
      const sheet = page.querySelector(".page");
      const laid = sheet && near && patched(sheet, near, added - removed);
      if (!sheet || !laid) return redrawn();
      hands.dressed(sheet, laid);
      opened(page, laid);
      rested(now);
      if (hands.text() === now) drawn(laid);
    });
  };

  const put = (asked: Edit, at = asked.from + asked.insert.length, drawnWhole = false) => {
    const text = hands.text();
    const [one, landed] = linedAs(text, asked, at);
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
    writer.kept = { node: page.body, offset: -1, at: landed };
    hands.write(next);
    if (drawnWhole) writer.whole = true;
    laidOut(text, next, one);
  };

  return { put, laidOut, stop };
};
