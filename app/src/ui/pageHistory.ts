import type { Caret } from "./pageCaret";
import { changeOf, type Edit } from "./pageEdits";
import type { Laying } from "./pageLaying";
import { recorded, type Turns, type Writer } from "./pageWriter";

type Stack = Turns["undone"];

export type PageHistory = ReturnType<typeof historyOn>;

export const historyOn = (writer: Writer, caret: Caret, laying: Laying) => {
  const { page, hands, turns } = writer;
  const { undone, redone } = turns;
  // What was written in the Markdown since the page was left is one step back.
  if (turns.left !== undefined && turns.left !== hands.text())
    recorded(turns, { text: turns.left, at: changeOf(turns.left, hands.text()).from });
  turns.left = undefined;

  const turned = (from: Stack, to: Stack, forward = false) => {
    const back = from.pop();
    if (!back) return;
    writer.pending = null;
    to.push({ text: hands.text(), at: back.at, after: back.after });
    writer.kept = { node: page.body, offset: -1, at: forward ? (back.after ?? back.at) : back.at };
    hands.write(back.text);
    writer.whole = true;
    hands.redraw(back.text);
  };

  // The caret goes with the words it stood in, as far into them as it was.
  // Words changed from beside the page, as a table's cell, are laid block by block and leave the caret where it is.
  const keptAcross = (one: Edit) => {
    const by = one.insert.length - (one.to - one.from);
    const moved = (at: number) => (at <= one.from ? at : Math.max(one.from, at + by));
    writer.pending = null;
    // A line begun and not yet written keeps what it will open with, only its place moves.
    const begun = writer.kept?.lead !== undefined ? writer.kept : null;
    if (begun) {
      writer.kept = { ...begun, node: page.body, offset: -1, at: moved(begun.at) };
      return;
    }
    const held = caret.chosen();
    const sheet = page.querySelector(".page");
    const was =
      held?.collapsed && sheet?.contains(held.startContainer) ? caret.caretSpan()?.from : null;
    writer.kept =
      was === null || was === undefined ? null : { node: page.body, offset: -1, at: moved(was) };
  };

  const rewrite = (next: string) => {
    const text = hands.text();
    if (next === text) return;
    const one = changeOf(text, next);
    recorded(turns, { text, at: one.from });
    keptAcross(one);
    hands.write(next);
    laying.laidOut(text, next, one);
  };

  const outside = (was: string, next: string) => {
    if (next === was) return;
    const one = changeOf(was, next);
    keptAcross(one);
    laying.laidOut(was, next, one);
  };

  return {
    undo: () => turned(undone, redone),
    redo: () => turned(redone, undone, true),
    rewrite,
    outside,
  };
};
