import { t } from "../locales";
import { blank } from "./pageBlocks";
import type { Caret } from "./pageCaret";
import type { Edit, Span } from "./pageEdits";
import { fieldAsked } from "./pageFields";
import type { Laying } from "./pageLaying";
import { closing, opening } from "./pageLists";
import { closedTokens, closeOf, openOf, pastClose, rejoined, trimmed, wornOver } from "./pageMarks";
import { elementOf } from "./pageNodes";
import type { Writer } from "./pageWriter";
import { unformattable } from "./quickly";

export type Formats = ReturnType<typeof formatsOn>;

// An address typed as a site, «tisty.app», opens the site, never a file of that name beside the document.
export const addressed = (typed: string) =>
  !/^[a-z][a-z0-9+.-]*:/i.test(typed) && /^[\w-]+(\.[\w-]+)+(?:[/:?#]|$)/.test(typed)
    ? `https://${typed}`
    : typed;

const LINK =
  /(?<!!)\[((?:\\.|[^\]\\])*)\]\((<[^>]*>|(?:[^\s()]|\([^\s()]*\))*)((?:\s+(?:"[^"]*"|'[^']*'|\([^)]*\)))?)\)/g;

type Written = {
  readonly from: number;
  readonly to: number;
  readonly label: string;
  readonly address: string;
  readonly title: string;
};

export const writtenLink = (text: string, at: Span): Written | null => {
  const start = text.lastIndexOf("\n", at.from - 1) + 1;
  const end = text.indexOf("\n", at.to);
  const line = text.slice(start, end < 0 ? text.length : end);
  for (const hit of line.matchAll(LINK)) {
    const from = start + (hit.index ?? 0);
    const to = from + hit[0].length;
    if (from <= at.from && at.to <= to)
      return { from, to, label: hit[1], address: hit[2], title: hit[3] };
  }
  return null;
};

const sameLink = (link: Written) => `[${link.label}](${link.address}${link.title})`;

export const wornWords = (text: string, one: Span, change: Edit, open: string, close: string) => {
  const end = change.from + change.insert.length;
  const within = text.slice(one.from + open.length, one.to - close.length);
  if (change.from < one.from || change.insert === within) return { from: change.from, to: end };
  const words = change.insert.trimEnd();
  const from = change.from + (words.startsWith(open) ? open.length : 0);
  return { from, to: change.from + words.length - (words.endsWith(close) ? close.length : 0) };
};

export const formatsOn = (writer: Writer, caret: Caret, laying: Laying) => {
  const { page, hands } = writer;
  const { put } = laying;

  const fieldNear = () => {
    const held = page.getSelection();
    const box = held?.rangeCount ? held.getRangeAt(0).getBoundingClientRect() : null;
    return box ?? page.querySelector(".page")?.getBoundingClientRect();
  };

  // A link already written keeps its words and its title; only its address changes.
  const relinked = (link: Written, back: number) => {
    const near = fieldNear();
    if (!near) return;
    const value = link.address.replace(/^<([^>]*)>$/, "$1");
    void fieldAsked(page, near, t("linkAddress"), { kind: "url", value }).then((asked) => {
      if (!asked || hands.text().slice(link.from, link.to) !== sameLink(link))
        return caret.placeAt(back);
      const to = addressed(asked);
      const aimed = /\s/.test(to) ? `<${to}>` : to;
      put({ from: link.from, to: link.to, insert: sameLink({ ...link, address: aimed }) }, back);
    });
  };

  const linked = (one: Span) => {
    const near = fieldNear();
    if (!near) return;
    void fieldAsked(page, near, t("linkAddress"), { kind: "url" }).then((asked) => {
      if (!asked) return caret.placeAt(one.to);
      const to = addressed(asked);
      const text = hands.text();
      const inner = text.slice(one.from, one.to) || to;
      const insert = `[${inner}](${to})`;
      const alone = one.from === one.to && blank(text, one.from);
      put({ ...one, insert });
      if (!alone) return;
      // A link alone on its line is drawn as a card, so what is typed next goes on a line under it.
      const at = one.from + insert.length;
      const now = hands.text();
      writer.kept = {
        node: page.body,
        offset: -1,
        at,
        lead: opening(now, at, ""),
        tail: closing(now, at),
      };
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
    const one = trimmed(text, caret.chosenNow());
    if (!one && carry && carry.caret === caret.caretSpan()?.from && carry.close.startsWith(close))
      return;
    if (close === "]()") {
      const span = one ?? caret.caretSpan();
      const written = span && writtenLink(text, span);
      if (written) return relinked(written, span.to);
      // A link written another way, by reference or as a bare address, is left as it is rather than nested.
      if (elementOf(caret.chosen()?.startContainer)?.closest(".page a")) return;
      if (span) linked(span);
      return;
    }
    if (!one) return waitFor(open, close);
    writer.pending = null;
    const wrapped = wornOver(text, one, open, close);
    if (!wrapped) return;
    const { change } = wrapped;
    put(change);
    const words = wrapped.words ?? wornWords(text, wrapped.one, change, open, close);
    if (writer.kept?.at === change.from + change.insert.length)
      writer.kept = { ...writer.kept, at: words.to, chosenFrom: words.from };
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
