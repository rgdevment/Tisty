import type { Slashed } from "tisty-editor/edit";
import { BOXES, charsAt } from "tisty-editor/read";
import { t } from "../locales";
import { unitFrom } from "../placing";
import type { Begun } from "./pageBegun";
import { blockPlaced, placed } from "./pageBlocks";
import type { Caret } from "./pageCaret";
import { changeOf, type Edit, pastFeeds, type Span } from "./pageEdits";
import type { Formats } from "./pageFormats";
import { blockAsked } from "./pageLate";
import type { Laying } from "./pageLaying";
import { closing, opening } from "./pageLists";
import { type Act, acted, gripped, type Moved, movedTo, roomy, topsOf, unitOf } from "./pageMoving";
import { elementOf, FIELDS, sectioned } from "./pageNodes";
import { placeOf } from "./pagePlaces";
import { GHOST } from "./pageSpots";
import type { Writer } from "./pageWriter";

export type Beside = ReturnType<typeof besideOn>;

// A diagram, a formula or a widget comes drawn from a sample, so the page keeps reading as it will.
const SAMPLES: Record<string, () => string> = {
  mermaid: () => "```mermaid\nflowchart LR\n  A --> B\n```",
  math: () => "```math\na^2 + b^2 = c^2\n```",
  widget: () =>
    `\`\`\`widget\n<div class="grid">\n  <div class="stat"><small>${t("widgetSample")}</small><b>42</b></div>\n</div>\n\`\`\``,
};

export const besideOn = (
  writer: Writer,
  caret: Caret,
  laying: Laying,
  begun: Begun,
  formats: Formats,
) => {
  const { page, hands } = writer;
  const { put, late } = laying;

  // The blocks wait on the sheet's left edge, beside the block the pointer is over; a key typed puts them away.
  const plus = page.createElement("button");
  plus.type = "button";
  plus.className = "page-plus";
  plus.textContent = "+";
  plus.title = t("blocksOf");
  plus.setAttribute("aria-label", t("blocksOf"));
  plus.hidden = true;
  let over: Element | null = null;
  // A block with words gets what is picked on a line begun under it, and only once something is picked.
  let under: { block: Element; at: number; end: number; words: boolean } | null = null;
  let pointer: { clientX: number; clientY: number } | null = null;

  // Written only when it changes, so a key typed with the + already away touches nothing the frame waits on.
  const hide = () => {
    if (!plus.hidden) plus.hidden = true;
  };
  // A key typed writes where the caret is, so a line waiting under a block is no longer asked for.
  const keyed = () => {
    hide();
    under = null;
  };

  // The line the + stands beside: the item under the pointer in a list, else the block's first line still in sight.
  const lineOf = (block: Element) => {
    const y = pointer?.clientY;
    const items = block.matches("ul, ol") ? [...block.querySelectorAll("li[data-at]")] : [];
    // A list inside an item comes after it, so the last item holding the pointer is the innermost;
    // between two items, the one above it keeps the +.
    const back = items.reverse();
    const item =
      y === undefined
        ? undefined
        : (back.find((one) => {
            const box = one.getBoundingClientRect();
            return box.top <= y && y <= box.bottom;
          }) ?? back.find((one) => one.getBoundingClientRect().top <= y));
    const one = item ?? block;
    const box = one.getBoundingClientRect();
    // A block's first line is as tall as its own line, however many lines the block holds.
    const said = Number.parseFloat(page.defaultView?.getComputedStyle(one).lineHeight ?? "");
    const tall = Math.min(box.height, said || box.height);
    const top = Math.min(Math.max(box.top, 0), box.bottom - tall);
    return { top, bottom: top + tall };
  };

  // A bare marker, a `- ` or a `## ` with nothing written after it, is a line turned into what is picked.
  const BARE = /^\s*(?:#{1,6}|>|(?:[-*+]|\d+[.)])(?:\s+\[[ xX]\])?)?\s*$/;

  // The caret stays where it was written until something is picked, so a menu put away moves nothing.
  const asked = () => {
    const block = over?.isConnected ? over : null;
    const span = block ? placeOf(block) : null;
    under = null;
    if (!block || !span) return hands.blocks(caret.caretPoint(), false);
    const text = hands.text();
    const at = charsAt(text, span[1]);
    const from = charsAt(text, span[0]);
    const said = text.slice(from, at);
    const words = !BARE.test(said);
    // An empty item holds no words for the caret to stand in, so it is put right after its marker,
    // never at the block's end, which the page would read as the start of the next block.
    const marked = !words && said.trim() !== "";
    const end = marked
      ? from + said.replace(/[\r\n]+$/, "").length
      : unitFrom(writer.reckoned, text, span[1]);
    under = { block, at, end, words };
    const box = block.getBoundingClientRect();
    // A block whose end is out of sight is answered from the line the + stands beside.
    const seen = box.bottom <= (page.defaultView?.innerHeight ?? Number.POSITIVE_INFINITY);
    const line = seen ? box : lineOf(block);
    const frame = page.defaultView?.frameElement?.getBoundingClientRect();
    const [left, top] = [frame?.left ?? 0, frame?.top ?? 0];
    hands.blocks({ x: left + box.left, y: top + line.bottom + 10, top: top + line.top - 4 }, false);
  };

  const begunUnder = () => {
    const asked = under;
    under = null;
    if (!asked?.block.isConnected) return;
    if (asked.words) begun.begin(asked.at, "", asked.block);
    else caret.placeAt(asked.end);
  };
  plus.addEventListener("mousedown", (event) => event.preventDefault());
  plus.addEventListener("click", asked);
  page.body.append(plus);

  const blockUnder = (y: number): Element | null => {
    const ghost = page.querySelector(GHOST);
    const near = ghost?.getBoundingClientRect();
    if (ghost && near && near.top <= y && y <= near.bottom) return ghost;
    const sheet = page.querySelector(".page");
    if (!sheet) return null;
    return (
      topsOf(sheet).find((one) => {
        const box = one.getBoundingClientRect();
        return box.top <= y && y <= box.bottom;
      }) ?? null
    );
  };

  const placedBeside = () => {
    const sheet = page.querySelector<HTMLElement>(".page");
    const block = over?.isConnected ? over : null;
    const field = page.activeElement?.closest?.(FIELDS);
    const solid = block?.matches(`hr, .sheet-turn, ${BOXES}`);
    if (!sheet || !block || solid || field || !roomy(page)) {
      plus.hidden = true;
      return;
    }
    const view = page.defaultView;
    const line = lineOf(block);
    const zoom = Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
    const edge = sheet.getBoundingClientRect().left;
    plus.hidden = false;
    const centred = line.top + (line.bottom - line.top - plus.offsetHeight) / 2;
    plus.style.top = `${(centred + (view?.scrollY ?? 0)) / zoom}px`;
    plus.style.left = `${(edge + (view?.scrollX ?? 0)) / zoom + 18}px`;
  };

  // The pointer is read once a frame, and the + moves only when it reaches another line.
  let pointing = 0;
  const pointed = (event: MouseEvent) => {
    if (event.target === plus) return;
    pointer = { clientX: event.clientX, clientY: event.clientY };
    reread();
  };
  // A scroll brings another block under a pointer that did not move.
  const reread = () => {
    if (pointing || !pointer) return;
    const { clientX, clientY } = pointer;
    pointing =
      page.defaultView?.requestAnimationFrame(() => {
        pointing = 0;
        const sheet = page.querySelector(".page")?.getBoundingClientRect();
        const near = sheet && clientX >= sheet.left && clientX <= sheet.right;
        const now = near ? blockUnder(clientY) : null;
        // A list or a block begun above the window holds more than one line the + may stand by.
        const lined = now?.matches("ul, ol") || (now?.getBoundingClientRect().top ?? 0) < 0;
        if (now === over && !plus.hidden && !lined) return;
        over = now;
        placedBeside();
      }) ?? 0;
  };
  const left = () => {
    over = null;
    pointer = null;
    hide();
  };
  const pressed = (event: MouseEvent) => {
    if (event.target !== plus) under = null;
  };
  page.addEventListener("mousemove", pointed);
  page.addEventListener("scroll", reread, true);
  page.documentElement.addEventListener("mouseleave", left);
  page.addEventListener("keydown", keyed, true);
  page.addEventListener("mousedown", pressed, true);

  // A block just drawn is measured at its placeholder size until the next frame lays it out.
  let measuring = 0;
  const besideLine = () => {
    if (plus.hidden) return;
    placedBeside();
    const view = page.defaultView;
    if (!view) return;
    view.cancelAnimationFrame(measuring);
    measuring = view.requestAnimationFrame(() => {
      placedBeside();
      measuring = view.requestAnimationFrame(placedBeside);
    });
  };

  // A break holds no words, so what is typed after it goes on a line begun under it.
  const lineUnder = (laid: Edit) => {
    const text = hands.text();
    const laidTo =
      writer.kept && writer.kept.offset < 0
        ? writer.kept.at
        : Math.min(text.length, laid.from + laid.insert.replace(/\s+$/, "").length);
    const at = pastFeeds(text, laidTo, true);
    writer.kept = {
      node: page.body,
      offset: -1,
      at,
      lead: opening(text, at, ""),
      tail: closing(text, at),
    };
  };

  // A block asked for inside code goes under the code, never into its lines.
  const outOfCode = () => {
    const range = caret.chosen();
    const code = range ? elementOf(range.startContainer)?.closest("figure.code") : null;
    const top = code?.closest(".page > [data-at]");
    const span = placeOf(top);
    if (top && span) begun.begin(charsAt(hands.text(), span[1]), "", top);
  };

  const slashed = (asked: Slashed, drawnWhole = false): Edit | null => {
    const sample = SAMPLES[asked.key]?.();
    const one: Slashed = sample
      ? { ...asked, put: { block: sample, at: sample.length, blank: true } }
      : asked;
    begunUnder();
    outOfCode();
    const kept = writer.kept;
    const unlaid = kept?.lead !== undefined && `${kept.lead}${kept.tail ?? ""}` !== "";
    // Until the page is drawn again, a line just begun is known by its place, not by what the page shows.
    const begunAt = kept && unlaid ? kept.at + (kept.lead?.length ?? 0) : null;
    if (kept && unlaid) begun.firstKey(kept, "");
    const span = begunAt === null ? caret.caretSpan() : { from: begunAt, to: begunAt };
    if (!span) return null;
    const text = hands.text();
    const end = begunAt ?? Math.max(span.to, caret.blockEnd(span.to));
    const how = placed(text, span.from, end, one, begunAt ?? caret.paragraphStart());
    if ("wrap" in how) {
      formats.format(how.wrap, how.close);
      return null;
    }
    if ("edit" in how) {
      put(how.edit, how.caret, drawnWhole);
      if (one.key === "rule" || sample) lineUnder(how.edit);
      return how.edit;
    }
    const before = text.slice(0, span.from);
    const there = blockAsked(text, span.from, end);
    const laidOf = (said: string) => (now: string, place: Span) => {
      const at = place.from;
      // An icon is drawn only with a space on each side, so it never sticks to the words around it.
      const icon = !how.block && /^::[\w-]+::$/.test(said);
      const lead = icon && at > 0 && !/\s/.test(now[at - 1] ?? "") ? " " : "";
      const inline = `${lead}${said}${icon && !/^[ \t]/.test(now.slice(at)) ? " " : ""}`;
      return how.block
        ? blockPlaced(now, at, place.to, said, said.length)
        : { edit: { from: at, to: at, insert: inline }, caret: at + inline.length };
    };
    hands.ask(
      how.ask,
      caret.caretPoint(),
      (said) =>
        late(there, laidOf(said), (laid) => {
          if (how.block) lineUnder(laid.edit);
          // The picker held the keys; once it closes they belong to the page again.
          page.defaultView?.focus();
          page.querySelector<HTMLElement>(".page")?.focus({ preventScroll: true });
        }),
      !before.trim(),
      before,
    );
    return null;
  };

  const replaced = (moved: Moved | null) => {
    if (!moved) return;
    const at = caret.caretSpan()?.from ?? -1;
    const inside = at >= moved.was && at <= moved.was + moved.size;
    put(changeOf(hands.text(), moved.text), moved.caret + (inside ? at - moved.was : 0));
  };

  const unitActed = (top: Element, act: Act) => {
    const sheet = page.querySelector(".page");
    if (!sheet) return;
    const tops = topsOf(sheet);
    replaced(acted(hands.text(), tops, tops.indexOf(top), act, sectioned(page)));
  };

  const movedOnto = (top: Element, target: Element | null) => {
    const sheet = page.querySelector(".page");
    if (!sheet) return;
    const text = hands.text();
    const tops = topsOf(sheet);
    const unit = unitOf(text, tops, tops.indexOf(top));
    const at = placeOf(target);
    if (unit) replaced(movedTo(text, unit, at ? charsAt(text, at[0]) : text.length));
  };

  const gripping = gripped(page, { moved: movedOnto, asked: (top, at) => hands.acts(at, top) });

  return {
    besideLine,
    slashed,
    replaced,
    unitActed,
    rest: gripping.rest,
    stop: () => {
      page.defaultView?.cancelAnimationFrame(measuring);
      page.removeEventListener("mousemove", pointed);
      page.removeEventListener("scroll", reread, true);
      page.documentElement.removeEventListener("mouseleave", left);
      page.removeEventListener("keydown", keyed, true);
      page.removeEventListener("mousedown", pressed, true);
      page.defaultView?.cancelAnimationFrame(pointing);
      plus.remove();
      gripping.stop();
    },
  };
};
