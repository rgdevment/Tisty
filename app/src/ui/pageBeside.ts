import type { Slashed } from "tisty-editor/edit";
import { BOXES, charsAt } from "tisty-editor/read";
import { t } from "../locales";
import type { Begun } from "./pageBegun";
import { blockPlaced, placed } from "./pageBlocks";
import type { Caret } from "./pageCaret";
import { changeOf, type Edit, pastFeeds, type Span } from "./pageEdits";
import type { Formats } from "./pageFormats";
import { blockAsked } from "./pageLate";
import type { Laying } from "./pageLaying";
import { closing, opening } from "./pageLists";
import { type Act, acted, gripped, type Moved, movedTo, roomy, topsOf, unitOf } from "./pageMoving";
import { caretBox, elementOf, FIELDS, lineOfCaret, sectioned } from "./pageNodes";
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

  // The blocks wait on the sheet's left edge, beside the line being written, as the folded bar did.
  const plus = page.createElement("button");
  plus.type = "button";
  plus.className = "page-plus";
  plus.textContent = "+";
  plus.title = t("blocksOf");
  plus.setAttribute("aria-label", t("blocksOf"));
  plus.hidden = true;
  plus.addEventListener("mousedown", (event) => event.preventDefault());
  plus.addEventListener("click", () => hands.blocks(caret.caretPoint(), false));
  page.body.append(plus);

  const placedBeside = () => {
    const held = page.getSelection();
    const range = held?.rangeCount ? held.getRangeAt(0) : null;
    const sheet = page.querySelector<HTMLElement>(".page");
    const field = page.activeElement?.closest?.(FIELDS);
    const tabled = elementOf(range?.startContainer)?.closest(BOXES);
    if (!range || !sheet?.contains(range.startContainer) || field || tabled || !roomy(page)) {
      plus.hidden = true;
      return;
    }
    const ghost = page.querySelector(GHOST);
    const view = page.defaultView;
    let box = caretBox(range);
    if (!box) {
      // Without a letter to measure, the line is its block's first, as tall as the block's own line.
      const line = ghost ?? lineOfCaret(range);
      if (!line || line.matches("hr, .sheet-turn")) {
        plus.hidden = true;
        return;
      }
      const block = line.getBoundingClientRect();
      const tall = Number.parseFloat(view?.getComputedStyle(line).lineHeight ?? "") || block.height;
      box = new DOMRect(block.left, block.top, block.width, Math.min(block.height, tall));
    }
    const zoom = Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
    const edge = sheet.getBoundingClientRect().left;
    plus.hidden = false;
    const centred = box.top + (box.height - plus.offsetHeight) / 2;
    plus.style.top = `${(centred + (view?.scrollY ?? 0)) / zoom}px`;
    plus.style.left = `${(edge + (view?.scrollX ?? 0)) / zoom + 18}px`;
  };

  // A block just drawn is measured at its placeholder size until the next frame lays it out.
  let measuring = 0;
  const besideLine = () => {
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
      plus.remove();
      gripping.stop();
    },
  };
};
