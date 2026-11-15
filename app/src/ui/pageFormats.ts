import { t } from "../locales";
import type { Caret } from "./pageCaret";
import type { Span } from "./pageEdits";
import { fieldAsked } from "./pageFields";
import type { Laying } from "./pageLaying";
import {
  closedTokens,
  closeOf,
  covered,
  openOf,
  pastClose,
  rejoined,
  trimmed,
  worn,
  wornEach,
} from "./pageMarks";
import type { Writer } from "./pageWriter";
import { unformattable } from "./quickly";

export type Formats = ReturnType<typeof formatsOn>;

// An address typed as a site, «tisty.app», opens the site, never a file of that name beside the document.
export const addressed = (typed: string) =>
  !/^[a-z][a-z0-9+.-]*:/i.test(typed) && /^[\w-]+(\.[\w-]+)+(?:[/:?#]|$)/.test(typed)
    ? `https://${typed}`
    : typed;

export const formatsOn = (writer: Writer, caret: Caret, laying: Laying) => {
  const { page, hands } = writer;
  const { put } = laying;

  const linked = (one: Span) => {
    const held = page.getSelection();
    const box = held?.rangeCount ? held.getRangeAt(0).getBoundingClientRect() : null;
    const near = box ?? page.querySelector(".page")?.getBoundingClientRect();
    if (!near) return;
    void fieldAsked(page, near, t("linkAddress"), { kind: "url" }).then((asked) => {
      if (!asked) return caret.placeAt(one.to);
      const to = addressed(asked);
      const inner = hands.text().slice(one.from, one.to) || to;
      put({ ...one, insert: `[${inner}](${to})` });
    });
  };

  const waitFor = (open: string, close: string) => {
    const at = caret.caretSpan()?.from;
    if (at === undefined) return;
    const text = hands.text();
    const marks = writer.pending?.at === at ? writer.pending.marks : [];
    if (marks.some(([one]) => one === open)) {
      const rest = marks.filter(([one]) => one !== open);
      writer.pending = rest.length ? { at, marks: rest } : null;
      return;
    }
    const closers = closedTokens(text, at);
    const shut = closers.findIndex(
      (one) => one === close || (open === "==" && one.startsWith("==")),
    );
    if (shut >= 0) {
      const passed = closers.slice(0, shut + 1);
      const reopened = passed
        .slice(0, shut)
        .reverse()
        .map((one) => [one.startsWith("==") ? "==" : one, one] as const);
      const past = at + passed.join("").length;
      const next = [...reopened, ...marks];
      writer.kept = { node: page.body, offset: -1, at: past };
      writer.pending = next.length ? { at: past, marks: next } : null;
      return;
    }
    writer.pending = { at, marks: [...marks, [open, close]] };
    caret.focused();
  };

  const format = (open: string, close: string) => {
    if (unformattable(page)) return;
    const carry = writer.carried;
    writer.carried = null;
    const text = hands.text();
    const chosenSpan = trimmed(text, caret.chosenNow());
    const one = chosenSpan && close !== "]()" ? covered(text, chosenSpan, open) : chosenSpan;
    if (!one && carry && carry.caret === caret.caretSpan()?.from && carry.close.startsWith(close))
      return;
    if (close === "]()") {
      const span = one ?? caret.caretSpan();
      if (span) linked(span);
      return;
    }
    if (!one) return waitFor(open, close);
    writer.pending = null;
    const spread = text.slice(one.from, one.to).includes("\n");
    const change = spread ? wornEach(text, one, open, close) : worn(text, one, open, close);
    put(change, change.from + change.insert.length);
  };

  const pendingKey = (at: number, typed: string) => {
    const wait = writer.pending;
    if (!wait) return;
    if (!typed.trim()) {
      put({ from: at, to: at, insert: typed });
      writer.pending = { ...wait, at: at + typed.length };
      return;
    }
    writer.pending = null;
    const lead = /^\s*/.exec(typed)?.[0] ?? "";
    const words = typed.trim();
    const trail = typed.slice(lead.length + words.length);
    const [from, marks] = lead ? [at, wait.marks] : rejoined(hands.text(), at, wait.marks);
    const left = { at: from, marks };
    const open = openOf(left);
    put(
      { from, to: from, insert: `${lead}${open}${words}${closeOf(left)}${trail}` },
      from + lead.length + open.length + words.length,
    );
  };

  const carriedKey = (text: string, one: Span, typed: string): boolean => {
    const carry = writer.carried?.caret === one.from ? writer.carried : null;
    writer.carried = null;
    if (carry && !typed.trim()) {
      writer.carried = { ...carry, caret: one.from + typed.length };
      put({ ...one, insert: typed });
      return true;
    }
    if (carry) {
      const spaces = text.slice(carry.from + carry.close.length, one.from);
      put(
        { from: carry.from, to: one.from, insert: `${spaces}${typed}${carry.close}` },
        carry.from + spaces.length + typed.length,
      );
      return true;
    }
    const past = typed.trim() ? null : pastClose(text, one.from);
    if (past === null) return false;
    writer.carried = {
      from: one.from,
      close: text.slice(one.from, past),
      caret: past + typed.length,
    };
    put({ from: past, to: past, insert: typed });
    return true;
  };

  return { format, pendingKey, carriedKey };
};
