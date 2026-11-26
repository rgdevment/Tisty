import { charsAt } from "tisty-editor/read";
import type { Begun } from "./pageBegun";
import { behindOn } from "./pageBehind";
import type { Caret, Standing } from "./pageCaret";
import { deletingOn } from "./pageDeleting";
import type { Span } from "./pageEdits";
import type { Formats } from "./pageFormats";
import type { PageHistory } from "./pageHistory";
import {
  beforeFeeds,
  breakAt,
  cutAtEdge,
  inOneCodeBlock,
  joined,
  leftOver,
  pastWhole,
} from "./pageInputEdges";
import { puttingOn } from "./pageLaidOver";
import type { Laying } from "./pageLaying";
import { headingAt } from "./pageLists";
import { escapeTaken } from "./pageMarks";
import { atGhost, inCode, onSheet, unwritable } from "./pageNodes";
import { pastingOn } from "./pagePasting";
import { placeOf } from "./pagePlaces";
import { typingOn } from "./pageTyping";
import { mendOf, wholeReach } from "./pageWholes";
import { composingKey, type Kept, type Writer } from "./pageWriter";

const PAIRS: Record<string, string> = {
  formatBold: "**",
  formatItalic: "*",
  formatStrikeThrough: "~~",
  formatUnderline: "++",
};

type Aim = {
  readonly picked: AbstractRange | null;
  readonly target: AbstractRange | undefined;
  readonly spread: boolean;
  readonly pastBlock: boolean;
  readonly range: AbstractRange | null;
};

export const inputOn = (
  writer: Writer,
  caret: Caret,
  laying: Laying,
  history: PageHistory,
  formats: Formats,
  begun: Begun,
) => {
  const { page, hands } = writer;
  const putting = puttingOn(writer, laying);
  const pasting = pastingOn(writer, caret, laying, begun);
  const typing = typingOn(writer, caret, formats, begun, putting);
  const deleting = deletingOn(writer, caret, laying, putting, (kind, named, standing) =>
    inputTaken(kind, named, null, null, standing),
  );

  const aimed = (kind: string, named: StaticRange | undefined, standing: Standing): Aim => {
    const picked = standing.range ?? caret.chosen();
    const shown = named && !standing.lagging ? named : undefined;
    const target =
      shown && !(shown.collapsed && picked?.collapsed) ? cutAtEdge(shown, picked, kind) : undefined;
    const spread = !leftOver(writer, picked) && Boolean(picked && !picked.collapsed);
    const pastBlock = spread && picked !== null && pastWhole(picked);
    const reachedWhole = spread && picked !== null && wholeReach(picked) !== null;
    const range = pastBlock || reachedWhole ? picked : (target ?? picked);
    return { picked, target, spread, pastBlock, range };
  };

  const reachedOf = (kind: string, standing: Standing, aim: Aim, held: AbstractRange | null) => {
    const { range, spread, pastBlock } = aim;
    const spanned =
      range &&
      ((held && caret.unbegun(kind, held)) ??
        (spread ? caret.spanChosen(range) : caret.spanOfRange(range, kind.startsWith("delete"))));
    return (
      (!spread && begun.waited(kind, standing.range)) ||
      (spanned && pastBlock ? beforeFeeds(hands.text(), spanned) : spanned)
    );
  };

  const shelfTaken = (kind: string, shelf: Element, data: string | null) => {
    const span = placeOf(shelf);
    const first = kind === "insertText" ? (data ?? "") : "";
    if (span && first) {
      begun.begin(charsAt(hands.text(), span[1]), "", shelf);
      if (writer.kept) begun.firstKey(writer.kept, first);
    }
  };

  const begunTaken = (line: Kept, kind: string, typed: string, carried: DataTransfer | null) => {
    if (kind === "insertFromPaste") return pasting.pastedOnBegun(line, carried);
    if (kind !== "insertText" && kind !== "insertReplacementText") return;
    if (kind === "insertText" && typed === "/") return hands.blocks(caret.caretPoint(), true);
    return begun.firstKey(line, typed);
  };

  const inputTaken = (
    kind: string,
    named: StaticRange | undefined,
    data: string | null,
    carried: DataTransfer | null,
    standing: Standing,
  ) => {
    const broken = deleting.besideGone(kind, standing.range);
    if (!broken && deleting.edgeHeld(kind, standing.range)) return;
    writer.leftFrom = undefined;
    if (kind === "insertParagraph") hands.settle?.();
    writer.typing = kind === "insertText" || kind.startsWith("delete");
    if (kind !== "insertText" && !kind.startsWith("format")) writer.pending = null;
    if (kind === "historyUndo") return history.undo();
    if (kind === "historyRedo") return history.redo();
    if (broken) return deleting.tookBeside(kind, broken);
    const aim = aimed(kind, named, standing);
    const { picked, spread, range } = aim;
    const bare =
      kind === "deleteContentBackward" && !spread ? deleting.caretFreed(standing.range) : null;
    if (bare) return laying.put(bare.edit, bare.caret);
    const shelf = range && !spread ? breakAt(range) : null;
    if (shelf) return shelfTaken(kind, shelf, data);
    // A caret dropped on a table, a picture or a widget has no words under it to write among.
    if (range && !spread && unwritable(range.startContainer)) return;
    const held = picked ?? range;
    const begunNow =
      writer.kept?.lead !== undefined &&
      (writer.kept.offset < 0 || Boolean(held && atGhost(held.startContainer, held.startOffset)));
    if (begunNow && kind.startsWith("delete")) return begun.dropBegun();
    const reached = reachedOf(kind, standing, aim, held);
    if (!range || !reached) return;
    const text = hands.text();
    const targeted = aim.target !== undefined;
    const reaching = deleting.keyReach(kind, text, reached, targeted, spread, standing.range);
    if ("gone" in reaching) return deleting.tookBeside(kind, reaching.gone);
    const joining = joined(text, reaching.reach, range) ?? reaching.ended;
    const kept = joining ?? reaching.reach;
    const coded = inCode(range.startContainer);
    const literal = inOneCodeBlock(range);
    const mend = spread ? mendOf(range, text, writer.reckoned) : null;
    const one: Span = coded ? kept : escapeTaken(text, kept);
    const typed = data ?? carried?.getData("text/plain") ?? "";
    const line = begunNow ? writer.kept : null;
    if (line) return begunTaken(line, kind, typed, carried);
    if (kind === "insertText") return typing.typedIn(one, range, typed, mend);
    writer.carried = null;
    if (kind === "insertReplacementText") return putting.putOver(one, typed, literal, mend);
    if (kind === "insertParagraph" || kind === "insertLineBreak")
      return typing.lineParted(kind, text, one, range, coded, mend);
    if (kind === "insertFromPaste") return pasting.pasted(one, carried, literal, mend);
    if (kind.startsWith("delete")) {
      const how = { joining: joining !== null, spread, literal, mend };
      return deleting.deleted(kind, text, one, how);
    }
    const pair = PAIRS[kind];
    if (pair) formats.format(pair, pair);
  };

  const lineKind = (kind: string) => {
    const at = kind === "insertLineBreak" ? caret.caretSpan()?.from : undefined;
    return at !== undefined && headingAt(hands.text(), at) ? "insertParagraph" : kind;
  };

  const behind = behindOn(writer, caret, {
    unplaced: deleting.unplaced,
    act: ({ kind, keyed }, standing) => {
      if (!keyed || !deleting.keyedDelete(kind, standing))
        inputTaken(kind, undefined, null, null, standing);
    },
  });

  const keyTaken = (
    kind: string,
    named: StaticRange | undefined,
    data: string | null,
    carried: DataTransfer | null,
  ) => {
    const standing = behind.caughtUp() ?? caret.standing();
    if (kind.startsWith("delete") && deleting.unplaced(kind, standing))
      return behind.hold(kind, false);
    behind.dropped();
    inputTaken(kind, named, data, carried, standing);
  };

  const asked = (event: InputEvent) => {
    if (!onSheet(page, event) || event.isComposing || event.inputType === "insertCompositionText")
      return;
    event.preventDefault();
    const [named] = event.getTargetRanges?.() ?? [];
    keyTaken(lineKind(event.inputType), named, event.data, event.dataTransfer);
  };

  const markHeld = (event: KeyboardEvent) => {
    const back = event.key === "Backspace";
    if ((!back && event.key !== "Delete") || event.defaultPrevented || event.isComposing) return;
    if (!onSheet(page, event) || event.ctrlKey || event.metaKey || event.altKey || event.shiftKey)
      return;
    const kind = back ? "deleteContentBackward" : "deleteContentForward";
    const standing = behind.caughtUp() ?? caret.standing();
    if (deleting.unplaced(kind, standing)) {
      event.preventDefault();
      return behind.hold(kind, true);
    }
    behind.dropped();
    if (deleting.keyedDelete(kind, standing)) event.preventDefault();
  };

  const lineBroken = (event: KeyboardEvent) => {
    const bare = !event.ctrlKey && !event.metaKey && !event.altKey && !composingKey(event);
    if (event.key !== "Enter" || !event.shiftKey || !bare || event.defaultPrevented) return;
    if (!onSheet(page, event)) return;
    event.preventDefault();
    keyTaken(lineKind("insertLineBreak"), undefined, null, null);
  };

  return {
    asked,
    started: typing.started,
    ended: typing.ended,
    pastedFiles: pasting.pastedFiles,
    markHeld,
    lineBroken,
    caughtUp: behind.caughtUp,
    behindLeft: behind.left,
    behindDropped: behind.dropped,
    pressed: behind.pressed,
  };
};
