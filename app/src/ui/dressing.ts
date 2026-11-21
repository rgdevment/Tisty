import { convertFileSrc } from "@tauri-apps/api/core";
import type { Widths } from "tisty-editor/edit";
import {
  type CardAsked,
  cardsPlaced,
  charsAt,
  type DrawnCards,
  readAs,
  sourceOf,
  unfolded,
} from "tisty-editor/read";
import { widgetsDrawn } from "tisty-sheet";
import { openLink } from "../core";
import type { Signing } from "../crediting";
import type { DayAt } from "../diary";
import { askAgain, serve, showing } from "../inUse";
import { locale, t } from "../locales";
import type { Held, PiecePlace } from "../pieceWindow";
import { pictured } from "../previews";
import { plainly } from "../refusal";
import { type Attaching, attachedAs, pressed } from "./attached";
import { boxKey, NOTE } from "./diaryPaging";
import { glancing } from "./glancing";
import { modKey } from "./macKeys";
import { placeCopied, placedAt, placeOf, placeSaid, placeWritten, samePlace } from "./pagePlaces";
import { type Shot, shotsOf, type TableWent, type Went, went, widgetsIn } from "./paging";
import { type Bar, type Formatting, type Menued, quickly } from "./quickly";
import { SAY } from "./saying";
import { signers } from "./signers";
import { offeredAsParts } from "./staging";
import { stickable, sticking } from "./sticking";
import { dragging, tabled } from "./tabled";
import { GLIMPSES } from "./webcarding";
import { gripped, widening } from "./widening";
import { type Lending, lend, notWidget } from "./widgeting";

/** Where a control asked from sits on screen: under it, its top and its right edge. */
export type Anchor = { x: number; y: number; top?: number; right?: number };

export type Reaching = {
  readonly body: string;
  readonly onDoc: (file: string, heading?: string) => void;
  readonly onTask?: (id: string) => void;
  readonly drawnCards?: DrawnCards;
  readonly onCardsMissing?: (asked: CardAsked[]) => void;
  readonly cardDays?: (box: Element) => void;
  readonly onTaskDone?: (id: string, title: string) => void;
  readonly onTaskStep?: (id: string, step: string, done: boolean) => void;
  readonly onOpen?: (reference: string) => void;
  readonly onTag?: (name: string) => void;
  // The page written in place: writing starts here (chars), with what the new line opens with; nothing is written until a key.
  readonly onBegin?: (at: number, seed: string, near: Element) => void;
  readonly onHintWords?: (at: [number, number], hint: HTMLElement) => void;
  // A piece drawn over its words opens in a window of its own, at its place in bytes; only a widget brings its words.
  readonly onPiece?: (held: Held, place?: PiecePlace, source?: string, code?: boolean) => void;
  // A code block's language, chosen from a menu that opens at this point of the page.
  readonly onLanguage?: (at: [number, number], near: Anchor) => void;
  // Bytes, as the window's own reader counts them, never converted; `nth` and what it says find it when the page is a save behind.
  readonly onDiagram?: (at: number, nth: number, source: string) => void;
  // Where on the window's own screen its choices open, under the bar of the figure.
  readonly onDiagramExport?: (nth: number, source: string, at: { x: number; y: number }) => void;
  readonly onMore?: (asked: Extract<Went, { kind: "more" }>) => void;
  readonly onPicture?: (shots: Shot[], nth: number) => void;
  readonly onHere?: (id: string) => void;
  readonly onAway?: () => void;
  readonly onTick?: (at: [number, number]) => void;
  readonly onStick?: (note: [number, number], to: number) => void;
  readonly onPart?: (at: [number, number]) => void;
  readonly onCut?: (at: [number, number]) => void;
  readonly stepDates?: Readonly<Record<string, string>>;
  readonly signing?: Signing;
  readonly onWiden?: (at: [number, number], widths: Widths) => void;
  readonly bar?: Bar;
  readonly onFormat?: (asked: Formatting) => void;
  readonly onMenu?: (asked: Menued) => void;
  readonly onTable?: (asked: TableWent) => void;
  readonly onDay?: (at: DayAt) => void;
  readonly onDayKeep?: () => void;
  readonly onDayCancel?: () => void;
  readonly onUnstick?: (at: DayAt, note: number) => void;
};

// Each code block on a page written in place says its language, and a press on it offers the others.
export const languaged = (page: Document) => {
  for (const figure of page.querySelectorAll<HTMLElement>(".page figure.code[data-at]")) {
    if (figure.querySelector(":scope > .code-lang")) continue;
    const tongue = /language-(\S+)/.exec(figure.querySelector("pre > code")?.className ?? "")?.[1];
    const button = asking(page, "codeLang", tongue ?? t("codeLangNone"));
    button.className = "code-lang";
    button.title = t("codeLanguage");
    button.setAttribute("contenteditable", "false");
    figure.append(button);
  }
};

const asking = (page: Document, asked: string, said: string): HTMLButtonElement => {
  const one = page.createElement("button");
  one.type = "button";
  one.dataset.asked = asked;
  one.textContent = said;
  return one;
};

export type Worn = { lent: Lending[]; kept: (() => void)[] };

export const fitted = (page: Document, wide?: number, zoom?: number) => {
  const root = page.documentElement;
  if (wide) root.style.setProperty("--wide", `${wide}px`);
  root.style.setProperty("zoom", String(zoom ?? 1));
};

const STRANGER: Attaching = { more: false };
const sources = new WeakMap<Element, string>();

export const pictures = (
  page: Document,
  live: () => boolean,
  attaching: Attaching = STRANGER,
): (() => void)[] => {
  const held: (() => void)[] = [];
  for (const img of page.querySelectorAll<HTMLImageElement>(".page img[src]")) {
    const at = img.getAttribute("src") ?? "";
    if (!at || /^(https?|data|asset|blob|file|tauri):/i.test(at)) continue;
    if (!pictured(at)) {
      attachedAs(page, img, at, live, held, attaching);
      continue;
    }
    img.dataset.kept = at;
    img.removeAttribute("src");
    held.push(showing(at));
    fetched(img, at, live);
  }
  return held;
};

export const picturesWithin = (root: ParentNode, live: () => boolean) => {
  for (const img of root.querySelectorAll<HTMLImageElement>("img[data-kept], img[src]")) {
    const at = img.getAttribute("src") || img.dataset.kept || "";
    if (/^(https?|data|asset|blob|file|tauri):/i.test(at) || !pictured(at)) continue;
    img.dataset.kept = at;
    img.removeAttribute("src");
    fetched(img, at, live);
  }
};

// A picture that cannot be brought says why where it would be, instead of leaving a hole.
const fetched = (img: HTMLImageElement, at: string, live: () => boolean) => {
  serve(at)
    .then((real) => {
      if (!live()) return;
      img.hidden = false;
      img.setAttribute("src", convertFileSrc(real));
    })
    .catch((error_: unknown) => {
      if (!live()) return;
      const page = img.ownerDocument;
      const note = page.createElement("span");
      note.className = "picture-away";
      const again = pressed(page, "media-again", t("tryAgain"), () => {
        askAgain(at);
        note.remove();
        fetched(img, at, live);
      });
      note.append(page.createTextNode(plainly(error_)), again);
      img.hidden = true;
      img.after(note);
    });
};

export const widgets = (page: Document, reach?: Reaching): Lending[] =>
  widgetsDrawn(page, { refused: notWidget, lent: lend }, (drawn, figure, source, bar) => {
    placeCopied(figure, drawn);
    sources.set(drawn, source);
    if (reach?.onPiece) {
      bar.append(asking(page, "code", t("codeShow")), asking(page, "apart", t("viewApart")));
    }
  });

// A comment alone under its heading is the template's word on what goes there, shown until something does.
const hinted = (page: Document, body: string) => {
  const heading = (one: Element | null) =>
    !one || /^H[1-6]$/.test(one.tagName) || one.matches(".front");
  const bytes = new TextEncoder().encode(body);
  const decoder = new TextDecoder();
  for (const one of page.querySelectorAll<HTMLElement>(".page > div[data-at]")) {
    const at = placeOf(one);
    if (!at || !one.previousElementSibling?.matches("h1, h2, h3, h4, h5, h6")) continue;
    const source = decoder.decode(bytes.subarray(at[0], at[1])).trim();
    const said = /^<!--([\s\S]*?)-->$/.exec(source);
    if (!said) continue;
    const alone = heading(one.nextElementSibling);
    one.classList.add(alone ? "hint" : "hint-gone");
    const box = /^-\s*\[ \]\s*/.exec(said[1].trim());
    one.dataset.hint = said[1].trim().slice(box?.[0].length ?? 0);
    if (!alone) continue;
    if (box) {
      one.classList.add("step-hint");
      one.dataset.seed = "- [ ] ";
    }
  }
  stepped(page, bytes);
  sectioned(page, bytes.length);
};

// The template grows at its end, shown only while it is edited; each label can be taken out.
const sectioned = (page: Document, size: number) => {
  const sheet = page.querySelector(".page");
  if (!sheet) return;
  // An empty page still takes its first section, written after whatever the file holds.
  const last = Array.from(sheet.querySelectorAll(":scope > [data-at]")).pop();
  const add = page.createElement("button");
  add.type = "button";
  add.className = "add-section";
  add.dataset.after = placeSaid(last) ?? `${size}-${size}`;
  add.dataset.seed = "## ";
  add.textContent = `+ ${t("addSection")}`;
  sheet.append(add);
  for (const label of sheet.querySelectorAll<HTMLElement>(":scope > :is(h1, h2, h3, h4, h5, h6)")) {
    const from = placeOf(label);
    if (!from) continue;
    const hint = label.nextElementSibling?.matches(".hint, .hint-gone")
      ? placeOf(label.nextElementSibling)
      : null;
    const drop = page.createElement("button");
    drop.type = "button";
    drop.className = "drop-section";
    placeWritten(drop, [from[0], (hint ?? from)[1]]);
    drop.setAttribute("aria-label", t("dropSection"));
    drop.title = t("dropSection");
    label.append(drop);
  }
};

const stepped = (page: Document, bytes: Uint8Array) => {
  const decoder = new TextDecoder();
  for (const list of page.querySelectorAll<HTMLElement>(".page > :is(ul, ol)[data-at]")) {
    const boxes = list.querySelectorAll(":scope > li.task > .check");
    if (!boxes.length) continue;
    const done = list.querySelectorAll(':scope > li.task > .check[data-done="true"]').length;
    list.style.setProperty("--done", `${Math.round((done / boxes.length) * 100)}%`);
    const add = page.createElement("button");
    add.type = "button";
    add.className = "add-step";
    add.dataset.after = placeSaid(list) ?? "";
    // A step added under a list takes its mark, or it would start a list of its own.
    const at = placeOf(list);
    const said = at ? decoder.decode(bytes.subarray(at[0], at[1])) : "";
    add.dataset.seed = `${/^[ \t>]*([-*+]|\d+[.)])/.exec(said)?.[1] ?? "-"} [ ] `;
    add.textContent = `+ ${t("addStep")}`;
    list.after(add);
  }
};

const titled = (page: Document) => {
  const sheet = page.querySelector<HTMLElement>(".page");
  if (!sheet) return;
  sheet.setAttribute("data-empty", t("pageEmpty"));
  if (sheet.firstElementChild || page.documentElement.classList.contains("untitled")) return;
  const slot = page.createElement("h1");
  slot.dataset.title = t("titleEmpty");
  sheet.prepend(slot);
};

export const worn = (page: Document, live: () => boolean, reach?: Reaching): Worn => {
  const writes = Boolean(reach?.onBegin);
  if (writes) titled(page);
  if (reach && writes && page.documentElement.classList.contains("untitled"))
    hinted(page, reach.body);
  datedSteps(page, reach?.stepDates);
  signers(page, reach?.signing);
  page.documentElement.classList.toggle("ticking", !!reach?.onTick);
  if (reach?.onStick) stickable(page);
  if (reach?.onWiden) gripped(page);
  if (reach?.onLanguage) languaged(page);
  const writable = Boolean(reach?.onTable);
  if (writable) tabled(page);
  if (reach?.onDiagram || reach?.onDiagramExport)
    diagramBars(page, { open: Boolean(reach.onDiagram), out: Boolean(reach.onDiagramExport) });
  if (reach?.onPiece) mathBars(page);
  if (reach?.onUnstick) unstickable(page);
  if (reach) glancing(page);
  let held: (() => void)[] = [];
  let lent: Lending[] = [];
  const read = readAs(page, {
    say: SAY,
    locale: locale(),
    live,
    writable,
    brought: () => {
      held = pictures(page, live, { more: Boolean(reach?.onMore) });
      lent = widgets(page, reach);
    },
    sites: reach && { glimpses: GLIMPSES },
  });
  if (reach) placedCards(page, reach);
  if (reach?.onPart) offeredAsParts(page, t("stepToPart"), reach.body);
  return { lent, kept: [...held, ...read.kept] };
};

export const placedCards = (page: Document, reach: Reaching) => {
  if (!reach.drawnCards) return;
  const missing = cardsPlaced(page, reach.drawnCards, undefined, reach.cardDays);
  if (missing.length > 0) reach.onCardsMissing?.(missing);
};

type BarWays = { readonly open: boolean; readonly out: boolean };

export const diagramBar = (figure: HTMLElement, ways: BarWays) => {
  if (figure.querySelector(":scope > .diagram-bar")) return;
  const page = figure.ownerDocument;
  const bar = page.createElement("div");
  bar.className = "diagram-bar";
  // The words live in an attribute the stylesheet shows, so finding and copying never meet them.
  const button = (asked: string, said: string) => {
    const one = asking(page, asked, "");
    one.setAttribute("aria-label", said);
    one.title = said;
    bar.append(one);
  };
  if (ways.out) button("diagramExport", t("diagramExport"));
  if (ways.open) button("diagram", t("diagramOpen"));
  figure.append(bar);
};

// A formula is drawn over its words, so a quiet bar beside it opens them, as a widget's does.
export const mathBars = (page: Document) => {
  for (const formula of page.querySelectorAll<HTMLElement>("div.math[data-at]")) {
    if (formula.querySelector(":scope > .math-bar")) continue;
    const bar = page.createElement("div");
    bar.className = "math-bar";
    const one = asking(page, "code", "");
    one.setAttribute("aria-label", t("codeShow"));
    one.title = t("codeShow");
    bar.append(one);
    formula.append(bar);
  }
};

const unstickable = (page: Document) => {
  for (const note of page.querySelectorAll(`.diary-words > ${NOTE}`)) {
    if (note.querySelector(":scope > .note-off")) continue;
    const off = asking(page, "unstick", "");
    off.className = "note-off";
    off.setAttribute("aria-label", t("diaryUnstick"));
    off.title = t("diaryUnstick");
    note.append(off);
  }
};

export const diagramBars = (page: Document, ways: BarWays = { open: true, out: false }) => {
  for (const figure of page.querySelectorAll<HTMLElement>("figure.diagram[data-at]"))
    diagramBar(figure, ways);
};

const figureAt = (page: Document, at: [number, number]) => {
  const figures = Array.from(page.querySelectorAll<HTMLElement>("figure.diagram[data-at]"));
  const nth = figures.findIndex((one) => samePlace(one, at));
  return nth >= 0 ? { nth, figure: figures[nth] } : null;
};

// The page sits in a frame; a menu of the window opens where the bar shows on the window's screen.
const under = (one: Element) => {
  const box = one.getBoundingClientRect();
  const frame = one.ownerDocument.defaultView?.frameElement?.getBoundingClientRect();
  return { x: box.left + (frame?.left ?? 0), y: box.bottom + 4 + (frame?.top ?? 0) };
};

const underBar = (figure: HTMLElement) => under(figure.querySelector(".diagram-bar") ?? figure);

export const answered = (page: Document, asked: Went, reach: Reaching) => {
  if (!asked) return;
  if (asked.kind === "table") reach.onTable?.(asked);
  if (asked.kind === "diagram") {
    const found = figureAt(page, asked.at);
    if (found) reach.onDiagram?.(asked.at[0], found.nth, sourceOf(found.figure));
  }
  if (asked.kind === "diagramExport") {
    const found = figureAt(page, asked.at);
    if (found) reach.onDiagramExport?.(found.nth, sourceOf(found.figure), underBar(found.figure));
  }
  if (asked.kind === "doc") reach.onDoc(asked.id, asked.at);
  if (asked.kind === "task") reach.onTask?.(asked.id);
  if (asked.kind === "taskDone") reach.onTaskDone?.(asked.id, asked.title);
  if (asked.kind === "taskStep") reach.onTaskStep?.(asked.id, asked.step, asked.done);
  if (asked.kind === "taskSteps") unfolded(page, asked.nth);
  if (asked.kind === "day") reach.onDay?.(asked.at);
  if (asked.kind === "dayKeep") reach.onDayKeep?.();
  if (asked.kind === "dayCancel") reach.onDayCancel?.();
  if (asked.kind === "unstick") reach.onUnstick?.(asked.at, asked.note);
  if (asked.kind === "part")
    reach.onPart?.([charsAt(reach.body, asked.at[0]), charsAt(reach.body, asked.at[1])]);
  if (asked.kind === "web") openLink(asked.href).catch(() => {});
  if (asked.kind === "here") {
    const at = page.getElementById(asked.id);
    if (at) at.scrollIntoView({ block: "start" });
    else reach.onHere?.(asked.id);
  }
  if (asked.kind === "attachment") reach.onOpen?.(asked.reference);
  if (asked.kind === "tag") reach.onTag?.(asked.name);
  if (asked.kind === "picture") {
    reach.onAway?.();
    reach.onPicture?.(shotsOf(page), asked.nth);
  }
  if (asked.kind === "tick")
    reach.onTick?.([charsAt(reach.body, asked.at[0]), charsAt(reach.body, asked.at[1])]);
  if (asked.kind === "hint") {
    const held =
      asked.from === "hint"
        ? placedAt<HTMLElement>(page, ".page > .hint", asked.at)
        : page.querySelector<HTMLElement>(
            `.page > .${asked.from}[data-after="${asked.at[0]}-${asked.at[1]}"]`,
          );
    const end = charsAt(reach.body, asked.at[1]);
    if (held && reach.onBegin) reach.onBegin(end, asked.seed, held);
  }
  if (asked.kind === "cut")
    reach.onCut?.([charsAt(reach.body, asked.at[0]), charsAt(reach.body, asked.at[1])]);
  if (asked.kind === "section" && reach.onBegin) {
    const last = placedAt(page, ".page > *", asked.at);
    const add = last?.nextElementSibling;
    const near = add?.matches(".add-step") ? add : last;
    if (near) reach.onBegin(charsAt(reach.body, asked.at[1]), "", near);
  }
  if (asked.kind === "title" && reach.onBegin) {
    const slot = page.querySelector(".page > h1[data-title]");
    if (slot) reach.onBegin(0, "# ", slot);
  }
  if (asked.kind === "end" && reach.onBegin) {
    const last = Array.from(page.querySelectorAll(".page > [data-at]")).pop();
    reach.onBegin(reach.body.length, "", last ?? page.querySelector(".page") ?? page.body);
  }
  // The properties are kept out of the page, so asking for them, as the column's Edit does, opens their window.
  if (asked.kind === "block") {
    const front = placedAt(page, ".page > .front", asked.at);
    // An agent's card and its file's own properties open only while the page is written, never while it is held.
    const opens = front?.matches(".agent-card, .agent-own") ? Boolean(reach.onBegin) : true;
    const nth = front?.matches(".agent-own") ? 1 : 0;
    if (front && opens) reach.onPiece?.("front", { at: asked.at[0], nth });
  }
  if (asked.kind === "block" && reach.onHintWords) {
    const hint = placedAt<HTMLElement>(page, ".page > :is(.hint, .hint-gone)", asked.at);
    if (hint) reach.onHintWords(asked.at, hint);
  }
  if (asked.kind === "more")
    reach.onMore?.({
      ...asked,
      at: [charsAt(reach.body, asked.at[0]), charsAt(reach.body, asked.at[1])],
    });
  const widget = asked.kind === "code" ? widgetAt(page, asked.at) : null;
  if (widget) reach.onPiece?.("widget", widget.place, widget.source, true);
  else if (asked.kind === "code")
    reach.onPiece?.("formula", {
      at: asked.at[0],
      nth: nthAt(page, "div.math[data-at]", asked.at),
    });
  if (asked.kind === "formula")
    reach.onPiece?.("inlineFormula", { at: asked.at[0], nth: asked.nth });
  if (asked.kind === "codeLang")
    reach.onLanguage?.([charsAt(reach.body, asked.at[0]), charsAt(reach.body, asked.at[1])], {
      x: asked.x,
      y: asked.y,
      top: asked.top,
      right: asked.right,
    });
  if (asked.kind === "apart") {
    const block = placedAt(page, ".page > *", asked.at);
    const drawn = widgetsIn(block)[asked.nth ?? 0];
    const source = drawn ? sources.get(drawn) : undefined;
    if (drawn && source) reach.onPiece?.("widget", widgetPlace(page, drawn), source, false);
  }
};

// A widget is named to its window by where its block starts and which it is from the top, as a diagram is.
const widgetPlace = (page: Document, drawn: Element): PiecePlace | undefined => {
  const at = placeOf(drawn)?.[0];
  const nth = Array.from(page.querySelectorAll("figure.widget")).indexOf(drawn);
  return at === undefined || nth < 0 ? undefined : { at, nth };
};

export const nthAt = (page: Document, drawn: string, at: [number, number]) =>
  Math.max(
    0,
    Array.from(page.querySelectorAll(drawn)).findIndex((one) => samePlace(one, at)),
  );

const widgetAt = (page: Document, at: [number, number]) => {
  const drawn = placedAt(page, "figure.widget", at);
  const source = drawn ? sources.get(drawn) : undefined;
  return drawn && source ? { source, place: widgetPlace(page, drawn) } : null;
};

const below = (page: Document, target: Element, event: MouseEvent): boolean => {
  const sheet = page.querySelector(".page");
  if (!sheet || (target !== sheet && target !== page.body && target !== page.documentElement))
    return false;
  const blocks = sheet.querySelectorAll(":scope > :not(.cm-dock, .add-section)");
  const last = blocks[blocks.length - 1];
  return !last || event.clientY > last.getBoundingClientRect().bottom;
};

// Long enough for a second press to arrive, short enough that looking at a diagram still feels at once.
export const DOUBLE_MS = 300;

// Formats, links, undo and redo belong to the words written on the page, never to the window's own keys.
const WRITTEN_KEYS = new Set(["b", "i", "u", "k", "z", "y"]);
const WRITTEN_ON = '.page[contenteditable="true"], .diary-box';

export const wired = (page: Document, reach: () => Reaching) => {
  const refused = (event: Event) => event.preventDefault();
  let looking: ReturnType<typeof setTimeout> | undefined;
  // A selection dragged out of the block being written still belongs to it, wherever it is let go.
  let pressedIn = false;
  page.addEventListener(
    "mousedown",
    (event) => {
      pressedIn = Boolean((event.target as Element | null)?.closest?.(".editing"));
    },
    true,
  );
  page.addEventListener("click", (event) => {
    const target = event.target as Element | null;
    if (event.button !== 0 || typeof target?.closest !== "function") return;
    clearTimeout(looking);
    if (pressedIn) {
      pressedIn = false;
      return;
    }
    if (target.closest(".quick-bar, .tag-list, .mend-said, .cm-dock")) return;
    // The whole document is the only thing written: a press beside or under it goes to its end.
    const whole = page.querySelector<HTMLElement>(".page.whole .cm-content");
    if (whole && !target.closest(".editing")) {
      const end = page.createRange();
      end.selectNodeContents(whole);
      end.collapse(false);
      page.getSelection()?.removeAllRanges();
      page.getSelection()?.addRange(end);
      whole.focus();
      return;
    }
    const selecting = !(page.getSelection()?.isCollapsed ?? true);
    const link = target.closest("a");
    if (link) event.preventDefault();
    // A link among words being written takes the caret like them, and opens with Ctrl or Cmd held, as in a terminal.
    const amongWords =
      link?.closest('[contenteditable="true"]') &&
      !link.closest('[contenteditable="false"]') &&
      !link.getAttribute("href")?.startsWith("#");
    const shaping = page.documentElement.classList.contains("shaping");
    const asked =
      amongWords && !modKey(event)
        ? null
        : (went(target, selecting) ??
          (!shaping && below(page, target, event) ? { kind: "end" } : null));
    // The second press of a double one on a diagram is the window's, not a press away from the block being written.
    const second = event.detail >= 2 && target.closest("figure.diagram") && reach().onDiagram;
    if (!asked && !second && !target.closest(".editing")) reach().onAway?.();
    const diagram = asked?.kind === "picture" && target.closest<HTMLElement>("figure.diagram");
    const opening = diagram ? placeOf(diagram) : null;
    // A diagram that opens in its window on a double press waits a moment before it is looked at.
    if (opening && diagram && reach().onDiagram) {
      if (event.detail < 2)
        looking = setTimeout(() => {
          if (diagram.isConnected) answered(page, asked, reach());
        }, DOUBLE_MS);
      return;
    }
    answered(page, asked, reach());
  });
  // Its own event, since a double press on a label selects a word and the click then reads as a selection.
  page.addEventListener("dblclick", (event) => {
    const target = event.target as Element | null;
    const figure = target?.closest?.<HTMLElement>("figure.diagram.drawn[data-at]");
    const at = placeOf(figure);
    if (!at || !reach().onDiagram || target?.closest(".diagram-bar")) return;
    clearTimeout(looking);
    page.getSelection()?.removeAllRanges();
    answered(page, { kind: "diagram", at }, reach());
  });
  dragging(page, (box, from, to) => {
    const at = placeOf(box);
    if (at)
      answered(
        page,
        { kind: "table", at, act: "row", row: from, column: -1, to, x: 0, y: 0 },
        reach(),
      );
  });
  sticking(page, () => {
    const now = reach();
    const stick = now.onStick;
    if (!stick) return undefined;
    return (note, to) =>
      stick([charsAt(now.body, note[0]), charsAt(now.body, note[1])], charsAt(now.body, to));
  });
  widening(page, () => {
    const now = reach();
    const widen = now.onWiden;
    if (!widen) return undefined;
    return (at, widths) => widen([charsAt(now.body, at[0]), charsAt(now.body, at[1])], widths);
  });
  quickly(page, () => {
    const now = reach();
    const chars = (at: [number, number]): [number, number] => [
      charsAt(now.body, at[0]),
      charsAt(now.body, at[1]),
    ];
    const { bar, onFormat, onMenu } = now;
    return {
      bar,
      onFormat: onFormat && ((asked) => onFormat({ ...asked, at: chars(asked.at) })),
      onMenu:
        onMenu &&
        ((asked) =>
          onMenu({
            ...asked,
            picked: asked.picked && { ...asked.picked, at: chars(asked.picked.at) },
            block: asked.block && { ...asked.block, at: chars(asked.block.at) },
          })),
    };
  });
  for (const kind of ["auxclick", "submit", "dragstart"]) {
    page.addEventListener(kind, refused);
  }
  page.addEventListener(
    "keydown",
    (event) => {
      const asked = boxKey(event);
      if (asked) answered(page, asked, reach());
    },
    true,
  );
  // The window's own shortcuts listen on the app, which never hears a key pressed in the page.
  page.addEventListener("keydown", (event) => {
    const target = event.target as Element | null;
    const writing = target?.closest?.(".editing");
    const asked = modKey(event) || event.key === "F1";
    if (writing && (event.defaultPrevented || !asked)) return;
    if (asked && WRITTEN_KEYS.has(event.key.toLowerCase()) && target?.closest?.(WRITTEN_ON)) return;
    const passed = new KeyboardEvent("keydown", event);
    window.dispatchEvent(passed);
    if (passed.defaultPrevented) event.preventDefault();
  });
};

// A step is kept as written, marks and all, and cut past its length; the page shows only its words.
const bare = (said: string): string =>
  said
    .replace(/…$/, "")
    .replace(/[^\p{L}\p{N}]+/gu, " ")
    .trim()
    .toLowerCase();

export const datedSteps = (page: Document, dates?: Readonly<Record<string, string>>) => {
  const known = Object.entries(dates ?? {}).map(
    ([said, line]) => [bare(said), line, said.endsWith("…")] as const,
  );
  if (known.length === 0) {
    for (const one of page.querySelectorAll(".page li.task[title]")) one.removeAttribute("title");
    return;
  }
  for (const one of page.querySelectorAll<HTMLElement>(".page li.task")) {
    const own = one.cloneNode(true) as HTMLElement;
    for (const inner of own.querySelectorAll("ul, ol")) inner.remove();
    const words = bare(own.textContent ?? "");
    const said = known.find(
      ([key, , cut]) => key && (words === key || (cut && words.startsWith(key))),
    )?.[1];
    if (said && one.title !== said) one.title = said;
    else if (!said) one.removeAttribute("title");
  }
};
