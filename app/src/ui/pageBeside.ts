import type { Slashed } from "tisty-editor/edit";
import { charsAt, spanned } from "tisty-editor/read";
import { t } from "../locales";
import type { Begun } from "./pageBegun";
import { blockPlaced, placed } from "./pageBlocks";
import type { Caret } from "./pageCaret";
import { changeOf, type Edit } from "./pageEdits";
import type { Formats } from "./pageFormats";
import type { Laying } from "./pageLaying";
import { type Act, acted, gripped, type Moved, movedTo, roomy, topsOf, unitOf } from "./pageMoving";
import { elementOf, FIELDS, lineOfCaret } from "./pageNodes";
import { GHOST } from "./pageSpots";
import type { Writer } from "./pageWriter";

export type Beside = ReturnType<typeof besideOn>;

export const besideOn = (
  writer: Writer,
  caret: Caret,
  laying: Laying,
  begun: Begun,
  formats: Formats,
) => {
  const { page, hands } = writer;
  const { put } = laying;

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
    const tabled = elementOf(range?.startContainer)?.closest(".page > .table");
    if (!range || !sheet?.contains(range.startContainer) || field || tabled || !roomy(page)) {
      plus.hidden = true;
      return;
    }
    const ghost = page.querySelector(GHOST);
    let box = range.getBoundingClientRect();
    if (box.width === 0 && box.height === 0) {
      const line = ghost ?? lineOfCaret(range);
      if (!line) {
        plus.hidden = true;
        return;
      }
      box = line.getBoundingClientRect();
    }
    const view = page.defaultView;
    const zoom = Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
    const edge = sheet.getBoundingClientRect().left;
    plus.hidden = false;
    plus.style.top = `${(box.top + (view?.scrollY ?? 0)) / zoom}px`;
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

  const slashed = (one: Slashed, drawnWhole = false): Edit | null => {
    if (
      writer.kept?.lead !== undefined &&
      writer.kept.at > 0 &&
      hands.text()[writer.kept.at - 1] !== "\n"
    )
      begun.firstKey(writer.kept, "");
    const span = caret.caretSpan();
    if (!span) return null;
    const text = hands.text();
    const end = Math.max(span.to, caret.blockEnd(span.to));
    const how = placed(text, span.from, end, one, caret.paragraphStart());
    if ("wrap" in how) {
      formats.format(how.wrap, how.close);
      return null;
    }
    if ("edit" in how) {
      put(how.edit, how.caret, drawnWhole);
      return how.edit;
    }
    const first = !text.slice(0, span.from).trim();
    hands.ask(
      how.ask,
      caret.caretPoint(),
      (said) => {
        const now = hands.text();
        const at = Math.min(span.from, now.length);
        const laid = how.block
          ? blockPlaced(now, at, Math.min(end, now.length), said, said.length)
          : { edit: { from: at, to: at, insert: said }, caret: at + said.length };
        put(laid.edit, laid.caret);
        return true;
      },
      first,
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
    replaced(acted(hands.text(), tops, tops.indexOf(top), act));
  };

  const movedOnto = (top: Element, target: Element | null) => {
    const sheet = page.querySelector(".page");
    if (!sheet) return;
    const text = hands.text();
    const tops = topsOf(sheet);
    const unit = unitOf(text, tops, tops.indexOf(top));
    const at = spanned(target?.getAttribute("data-at") ?? null);
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
      plus.remove();
      gripping.stop();
    },
  };
};
