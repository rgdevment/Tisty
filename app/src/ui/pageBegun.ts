import { charsAt } from "tisty-editor/read";
import { t } from "../locales";
import type { Caret } from "./pageCaret";
import { blankAt, hardBroken, pastFeeds, reachIn, type Span } from "./pageEdits";
import { fieldAsked } from "./pageFields";
import type { Laying } from "./pageLaying";
import { closing, joinedAt, opening } from "./pageLists";
import { closeOf, joinedAcross, openOf } from "./pageMarks";
import { caretLaidOn, inCode } from "./pageNodes";
import { placeOf, TITLE_SLOT } from "./pagePlaces";
import { GHOST, titledAfterFront } from "./pageSpots";
import { choiceKept, type Kept, type Writer } from "./pageWriter";

export type Begun = ReturnType<typeof begunOn>;

export const begunOn = (writer: Writer, caret: Caret, laying: Laying) => {
  const { page, hands } = writer;
  const { put } = laying;

  // Until the page is drawn again it shows the words before the last key, so the caret kept in the words decides.
  const waited = (kind: string, shown: AbstractRange | null): Span | null => {
    if (!writer.kept || writer.kept.lead !== undefined) return null;
    const chosen = choiceKept(writer.kept);
    if (chosen) return chosen;
    if (writer.kept.offset >= 0) {
      const range = caret.chosen();
      const here =
        range?.collapsed &&
        range.startContainer === writer.kept.node &&
        range.startOffset === writer.kept.offset;
      if (!here || (kind !== "deleteContentBackward" && kind !== "deleteContentForward"))
        return null;
    }
    const { at } = writer.kept;
    const text = hands.text();
    if (!kind.startsWith("delete")) return { from: at, to: at };
    const reach = reachIn(text, at, kind);
    const bare = kind === "deleteContentBackward" && blankAt(text, at);
    if (!bare && (kind.includes("Line") || hardBroken(text.slice(reach.from, reach.to))))
      return reach;
    if (!shown) return bare ? { from: pastFeeds(text, at, true), to: at } : reach;
    const across =
      kind.startsWith("deleteContent") && joinedAcross(text, at, kind.endsWith("Backward"));
    return across && !inCode(shown.startContainer) ? across : reach;
  };

  // A line begun and not written loses its stand-in on a press back, and the next key still begins it there.
  const dropBegun = () => {
    const ghosts = [...page.querySelectorAll(GHOST)];
    // The title's slot is the page's own, so an empty page keeps it and waits there.
    if (ghosts.some((one) => one.matches(TITLE_SLOT))) return;
    for (const one of ghosts) one.remove();
    if (writer.kept) writer.kept = { ...writer.kept, node: page.body, offset: -1 };
  };

  // A line begun from a hint, a label or a button takes what it opens with and its blank lines with its first key.
  const firstKey = (begun: Kept, typed: string) => {
    const lead = begun.lead ?? "";
    const wait = writer.pending?.at === begun.at && typed.trim() ? writer.pending : null;
    writer.pending = null;
    const open = wait ? openOf(wait) : "";
    put(
      {
        from: begun.at,
        to: begun.at,
        insert: `${lead}${open}${typed}${wait ? closeOf(wait) : ""}${begun.tail ?? ""}`,
      },
      begun.at + lead.length + open.length + typed.length,
    );
  };

  const typed = (said: string) => {
    if (writer.kept?.lead !== undefined) return firstKey(writer.kept, said);
    const span = caret.caretSpan();
    if (span) put({ ...span, insert: said });
  };

  const laidGhost = (
    near: Element,
    inside: boolean,
    at: number,
    lead: string,
    tail: string,
    above = false,
  ) => {
    for (const one of page.querySelectorAll(GHOST)) if (one !== near) one.remove();
    const titled = near.matches(TITLE_SLOT);
    const ghost = titled
      ? (near as HTMLElement)
      : page.createElement(above && near.matches("li") ? "li" : "p");
    ghost.dataset.ghost = "";
    if (!titled) {
      ghost.append(page.createElement("br"));
      if (near.matches(".page")) near.prepend(ghost);
      else if (inside) near.append(ghost);
      else if (above || near.matches(".add-step")) near.before(ghost);
      else near.after(ghost);
      titledAfterFront(ghost);
    }
    writer.kept = { node: ghost, offset: 0, at, lead, tail };
    page.querySelector<HTMLElement>(".page")?.focus();
    caretLaidOn(page, ghost, 0);
    caret.inSight(ghost);
  };

  const begin = (asked: number, seed: string, near: Element) => {
    const text = hands.text();
    const at = joinedAt(text, asked, seed);
    laidGhost(near, false, at, `${opening(text, at, seed)}${seed}`, closing(text, at));
  };

  // A quote or a sticky note with no words yet holds no caret, so its first line is begun inside it.
  const quoted = (event: MouseEvent) => {
    const target = event.target as Element | null;
    const fold = target?.closest?.<HTMLDetailsElement>(".page > details[data-at]");
    if (fold && !target?.closest("summary") && !fold.querySelector(`[data-at], ${GHOST}`))
      return intoFold(fold);
    const quote = target?.closest?.(".page > blockquote[data-at]");
    if (!quote || quote.querySelector(`[data-src], ${GHOST}`)) return;
    const span = placeOf(quote);
    if (!span) return;
    const text = hands.text();
    const at = pastFeeds(text, charsAt(text, span[1]), true);
    const line = text.slice(text.lastIndexOf("\n", at - 1) + 1, at);
    const bare = /^\s*>\s*$/.test(line);
    laidGhost(quote, true, at, bare ? (line.endsWith(" ") ? "" : " ") : "\n> ", "");
  };

  // A section's hint is a comment the page does not show as words, so it is mended in a field of its own.
  const hintWords = (at: [number, number], hint: HTMLElement) => {
    const text = hands.text();
    const from = charsAt(text, at[0]);
    const to = charsAt(text, at[1]);
    const box = /^<!--\s*(-\s*\[ \]\s*)/.exec(text.slice(from, to))?.[1] ?? "";
    const near = hint.getBoundingClientRect();
    const value = hint.dataset.hint ?? "";
    void fieldAsked(page, near, t("sectionHint"), { value }).then((said) => {
      if (said !== null) put({ from, to, insert: `<!-- ${box}${said} -->` });
    });
  };

  const beginAbove = (at: number, seed: string, tail: string, near: Element) =>
    laidGhost(near, false, at, seed, tail, true);

  // A collapsible keeps its words under its name, so a line begun in it opens right below the name.
  const intoFold = (fold: HTMLDetailsElement) => {
    const span = placeOf(fold);
    if (!span) return;
    const text = hands.text();
    const named = text.indexOf("\n", charsAt(text, span[0]));
    if (named < 0) return;
    const at = text[named - 1] === "\r" ? named - 1 : named;
    fold.open = true;
    laidGhost(fold, true, at, "\n\n", closing(text, at));
  };

  return { waited, dropBegun, firstKey, typed, begin, beginAbove, intoFold, quoted, hintWords };
};
