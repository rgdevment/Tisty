import {
  BOXES,
  type Design,
  designNamed,
  diagramShot,
  isOwn,
  spanned,
  type TableAct,
  taskCardAt,
} from "tisty-editor/read";
import { t } from "../locales";
import { addressed, DESIGN, docAimed, TASK } from "../markdown";
import { type DayWent, dayPressed, dayTouched } from "./diaryPaging";
import type { Choice } from "./Menu";
import { placeOf } from "./pagePlaces";
import { tableClicked } from "./tabled";

export type Went =
  | { kind: "doc"; id: string; at?: string }
  | { kind: "task"; id: string }
  | { kind: "taskDone"; id: string; title: string }
  | { kind: "taskStep"; id: string; step: string; done: boolean }
  | { kind: "taskSteps"; nth: number }
  | { kind: "web"; href: string }
  | { kind: "here"; id: string }
  | { kind: "design"; name: Design }
  | { kind: "attachment"; reference: string }
  | { kind: "block"; at: [number, number] }
  | { kind: "code"; at: [number, number] }
  | { kind: "codeLang"; at: [number, number]; x: number; y: number; top: number; right: number }
  | { kind: "tag"; name: string }
  | { kind: "formula"; at: [number, number]; nth: number }
  | { kind: "tick"; at: [number, number] }
  | { kind: "part"; at: [number, number] }
  | { kind: "apart"; at: [number, number]; nth?: number }
  | {
      kind: "more";
      at: [number, number];
      target: string;
      x: number;
      y: number;
      paged: boolean;
    }
  | { kind: "end" }
  | { kind: "title" }
  | { kind: "hint"; at: [number, number]; seed: string; from: "hint" | "add-step" | "add-section" }
  | { kind: "section"; at: [number, number] }
  | { kind: "cut"; at: [number, number] }
  | { kind: "picture"; nth: number }
  | { kind: "diagram"; at: [number, number] }
  | { kind: "diagramExport"; at: [number, number] }
  | TableWent
  | DayWent
  | null;

/** A table's own click: `at` in bytes as the page says it, `row` by its place in the file, `to` where a dragged row lands. */
export type TableWent = {
  kind: "table";
  at: [number, number];
  act: TableAct;
  row: number;
  column: number;
  to?: number;
  x: number;
  y: number;
  top?: number;
  right?: number;
};

// A picture or a diagram, as the viewer shows it; only a kept file can be saved or opened.
export type Shot = { readonly src: string; readonly kept: string | null; readonly alt: string };

// What a click leaves alone, and so no picture inside it is one to look at.
const UNTOUCHED = "summary, figure.widget, .media, .col-grip, .picture-away, .table-input";

const SHOTS = ".page img[src], .page figure.diagram.drawn";

// A formula written inline is drawn whole; one the core could not draw shows its words as code.
const FORMULAS = ".page [data-src]:has(> math), .page code.math[data-src]";

const lookable = (all: Iterable<Element>): Element[] =>
  Array.from(all).filter((one) => !one.closest(`.doc-card, details:not([open]), ${UNTOUCHED}`));

const shotsIn = (page: Document): Element[] => lookable(page.querySelectorAll(SHOTS));

export const shotAt = (page: Document, target: Element | null): number | null => {
  const shot = target?.closest(SHOTS);
  const nth = shot ? shotsIn(page).indexOf(shot) : -1;
  return nth >= 0 ? nth : null;
};

export type PictureWays = {
  readonly look: (nth: number) => void;
  readonly save: (kept: string) => void;
  readonly open: (kept: string) => void;
};

// What a picture's own menu offers above writing it: a kept file can also leave or open.
export const pictureChoices = (shots: Shot[], nth: number | null, ways: PictureWays): Choice[] => {
  const kept = nth === null ? null : (shots[nth]?.kept ?? null);
  if (nth === null || !shots[nth]) return [];
  const look: Choice = { key: "look", label: t("pictureLook"), onPick: () => ways.look(nth) };
  if (!kept) return [look];
  return [
    look,
    { key: "save", label: t("pictureSave"), onPick: () => ways.save(kept) },
    { key: "open", label: t("pictureOpen"), onPick: () => ways.open(kept) },
  ];
};

const shotOf = (one: Element): Shot => {
  if (one.tagName !== "IMG") return { src: diagramShot(one) ?? "", kept: null, alt: "" };
  const img = one as HTMLImageElement;
  return { src: img.currentSrc || img.src, kept: img.dataset.kept ?? null, alt: img.alt };
};

export const shotsOf = (page: Document): Shot[] => shotsIn(page).map(shotOf);

// What a label holds runs to the next label, so writing under it starts past the last of it.
const sectionEnd = (label: Element): [number, number] | null => {
  let last = placeOf(label);
  for (let one = label.nextElementSibling; one; one = one.nextElementSibling) {
    if (/^H[1-6]$/.test(one.tagName)) break;
    last = placeOf(one) ?? last;
  }
  return last;
};

const linked = (link: HTMLAnchorElement): Went => {
  const doc = link.dataset.doc;
  if (doc) return { kind: "doc", id: doc };
  const task = link.dataset.task;
  if (task) return { kind: "task", id: task };
  const href = (link.getAttribute("href") ?? "").trim();
  const aimed = docAimed(href);
  if (aimed?.file) return { kind: "doc", id: aimed.file, at: aimed.heading };
  if (href.startsWith(TASK) && href.length > TASK.length)
    return { kind: "task", id: href.slice(TASK.length) };
  if (href.startsWith(DESIGN)) {
    const asked = href.slice(DESIGN.length);
    const name = designNamed(asked);
    return isOwn(name) || name === asked ? { kind: "design", name } : null;
  }
  if (/^(https?:\/\/|mailto:)\S+$/i.test(href) || addressed(href)) return { kind: "web", href };
  if (href.startsWith("#")) return href.length > 1 ? { kind: "here", id: href.slice(1) } : null;
  if (href && !/^[a-z][a-z0-9+.-]*:/i.test(href)) return { kind: "attachment", reference: href };
  return null;
};

export const widgetsIn = (block: Element | null): Element[] =>
  block?.matches("figure.widget")
    ? [block]
    : Array.from(block?.querySelectorAll("figure.widget") ?? []);

const tablePressed = (asked: HTMLElement): Went => {
  const at = placeOf(asked.closest<HTMLElement>(BOXES));
  const box = asked.getBoundingClientRect();
  const place = (said?: string) => (said === undefined ? -1 : Number(said));
  return at
    ? {
        kind: "table",
        ...(asked.dataset.to === undefined ? {} : { to: Number(asked.dataset.to) }),
        at,
        act: asked.dataset.act as TableAct,
        row: place(asked.dataset.row),
        column: place(asked.dataset.column),
        x: box.left,
        y: box.bottom,
        top: box.top,
        right: box.right,
      }
    : null;
};

const taskPressed = (asked: HTMLElement): Went | undefined => {
  const id = asked.dataset.task ?? "";
  if (asked.dataset.asked === "taskSteps") return { kind: "taskSteps", nth: taskCardAt(asked) };
  if (asked.dataset.asked === "taskDone") {
    const title = asked.closest(".is-task")?.querySelector(".card-name")?.textContent ?? "";
    return { kind: "taskDone", id, title };
  }
  if (asked.dataset.asked !== "taskStep") return undefined;
  const done = asked.getAttribute("aria-checked") !== "true";
  return { kind: "taskStep", id, step: asked.dataset.step ?? "", done };
};

const pressedIn = (asked: HTMLElement): Went => {
  const day = dayPressed(asked);
  if (day !== undefined) return day;
  const task = taskPressed(asked);
  if (task !== undefined) return task;
  if (asked.dataset.asked === "table") return tablePressed(asked);
  // A diagram inside a quote or a list is named by its own place, not by the block around it.
  const diagram = asked.dataset.asked;
  if (diagram === "diagram" || diagram === "diagramExport") {
    const at = placeOf(asked.closest<HTMLElement>("figure.diagram[data-at]"));
    return at ? { kind: diagram, at } : null;
  }
  if (asked.dataset.asked === "part") {
    const step = placeOf(asked.closest<HTMLElement>("li.task[data-at]"));
    return step ? { kind: "part", at: step } : null;
  }
  const at = placeOf(asked.closest<HTMLElement>(".page > [data-at]"));
  if (!at) return null;
  if (asked.dataset.asked === "more") {
    const box = asked.getBoundingClientRect();
    const target = asked.dataset.target ?? "";
    const paged = asked.closest(".doc-card")?.classList.contains("is-page") ?? false;
    return { kind: "more", at, target, x: box.left, y: box.bottom, paged };
  }
  if (asked.dataset.asked === "codeLang") {
    const fenced = placeOf(asked.closest<HTMLElement>("figure.code[data-at]"));
    const box = asked.getBoundingClientRect();
    return fenced
      ? { kind: "codeLang", at: fenced, x: box.left, y: box.bottom, top: box.top, right: box.right }
      : null;
  }
  if (asked.dataset.asked === "code") {
    const widget = placeOf(asked.closest("figure.widget[data-at], div.math[data-at]"));
    return { kind: "code", at: widget ?? at };
  }
  if (asked.dataset.asked !== "apart") return { kind: "block", at };
  const nth = widgetsIn(asked.closest(".page > [data-at]")).indexOf(
    asked.closest("figure.widget") as Element,
  );
  return nth > 0 ? { kind: "apart", at, nth } : { kind: "apart", at };
};

export const went = (target: Element, selecting: boolean): Went => {
  const asked = target.closest<HTMLElement>("button[data-asked]");
  if (asked) return pressedIn(asked);
  const link = target.closest("a");
  if (link) return linked(link);
  const card = target.closest<HTMLElement>(".doc-card[data-doc]")?.dataset.doc;
  if (card) return { kind: "doc", id: card };
  // A task that is not here has nothing to open, so its card is only the words it was written with.
  const task = target.closest<HTMLElement>(".doc-card[data-task]:not(.is-gone)")?.dataset.task;
  if (task) return { kind: "task", id: task };
  const ref = target.closest<HTMLElement>(".ref");
  if (ref?.dataset.stage) return { kind: "here", id: ref.dataset.stage };
  if (ref?.dataset.ref) return null;
  const tag = target.closest<HTMLElement>(".tag")?.dataset.tag;
  if (tag) return { kind: "tag", name: tag };
  if (selecting || target.closest(UNTOUCHED)) return null;
  const day = dayTouched(target);
  if (day !== undefined) return day;
  const formula = target.closest(FORMULAS);
  const written = placeOf(formula, "data-src");
  if (formula && written) {
    const nth = Array.from(target.ownerDocument.querySelectorAll(FORMULAS)).indexOf(formula);
    return { kind: "formula", at: written, nth };
  }
  // A picture or a diagram is looked at; it is written only from the block's own menu.
  const shot = shotAt(target.ownerDocument, target);
  if (shot !== null) return { kind: "picture", nth: shot };
  const tabled = tableClicked(target);
  if (tabled) {
    const at = placeOf(target.closest<HTMLElement>(BOXES));
    const box = target.closest("td, th")?.getBoundingClientRect();
    return at ? { kind: "table", at, ...tabled, x: box?.left ?? 0, y: box?.bottom ?? 0 } : null;
  }
  // A table written in place opens its Markdown only from its menu, never from a click around its cells.
  if (target.closest(`${BOXES}.dressed`)) return null;
  // A stage's box ticks where it is read; the words beside it still open the block.
  const box = target.closest(".check")?.closest<HTMLElement>("li.task[data-at]");
  const ticked = placeOf(box);
  if (ticked) return { kind: "tick", at: ticked };
  if (target.closest(".page > h1[data-title]")) return { kind: "title" };
  const look = target.ownerDocument.documentElement.classList;
  const shaping = look.contains("untitled") && look.contains("shaping");
  const drop = placeOf(target.closest<HTMLElement>(".drop-section"));
  if (drop) return shaping ? { kind: "cut", at: drop } : null;
  const hinted = target.closest<HTMLElement>(
    ".page > :is(.hint, .hint-gone, .add-step, .add-section)",
  );
  const hint = placeOf(hinted) ?? spanned(hinted?.dataset.after ?? null);
  // While the template is edited, a hint is its words to mend, not a place to write.
  if (hint && shaping && hinted?.matches(".hint, .hint-gone")) return { kind: "block", at: hint };
  if (shaping && hinted?.matches(".add-step")) return null;
  if (hint) {
    const from = hinted?.classList.contains("add-section")
      ? "add-section"
      : hinted?.classList.contains("add-step")
        ? "add-step"
        : "hint";
    return { kind: "hint", at: hint, seed: hinted?.dataset.seed ?? "", from };
  }
  // A task's sections are filled in, not renamed, until its structure is asked for; an empty one is still mended.
  const label = target.closest(".page > :is(h1, h2, h3, h4, h5, h6)");
  if (look.contains("untitled") && !look.contains("shaping") && label?.textContent?.trim()) {
    const end = sectionEnd(label);
    return end ? { kind: "section", at: end } : null;
  }
  if (shaping && !label) return null;
  const at = placeOf(target.closest<HTMLElement>(".page > [data-at]"));
  return at ? { kind: "block", at } : null;
};
