import { convertFileSrc } from "@tauri-apps/api/core";
import { type Lent, openLink } from "../core";
import { markup } from "../glyphs";
import { askAgain, serve, showing } from "../inUse";
import { t } from "../locales";
import { pictured } from "../previews";
import { plainly } from "../refusal";
import { type Attaching, attachedAs, lents, pressed } from "./attached";
import { type Carding, carded } from "./carding";
import { charsAt, colored, HUES, saidKinds, spanned, type Went, went, widgetsIn } from "./paging";
import { type PartCarded, partCarded } from "./partcarding";
import { type Formatting, type Menued, quickly } from "./quickly";
import { offeredAsParts, staged } from "./staging";
import { webCarded } from "./webcarding";
import { gripped, widening } from "./widening";
import { type Lending, lend, locked } from "./widgeting";

export type Reaching = Carding & {
  readonly body: string;
  readonly onDoc: (file: string) => void;
  readonly onTask?: (id: string) => void;
  readonly part?: (id: string) => PartCarded;
  readonly onOpen?: (reference: string) => void;
  readonly onTag?: (name: string) => void;
  readonly onBlock?: (
    at: [number, number],
    block: HTMLElement,
    seed?: string,
    read?: string,
  ) => void;
  readonly onApart?: (lent: Lent) => void;
  readonly onMore?: (asked: Extract<Went, { kind: "more" }>) => void;
  readonly onHere?: (id: string) => void;
  readonly onAway?: () => void;
  readonly onTick?: (at: [number, number]) => void;
  readonly onPart?: (at: [number, number]) => void;
  readonly onCut?: (at: [number, number]) => void;
  readonly stepDates?: Readonly<Record<string, string>>;
  readonly onWiden?: (at: [number, number], column: number, dashes: number) => void;
  readonly onFormat?: (asked: Formatting) => void;
  readonly onMenu?: (asked: Menued) => void;
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

export const iconed = (page: Document | HTMLElement) => {
  const made = "createElementNS" in page ? page : page.ownerDocument;
  for (const one of page.querySelectorAll<HTMLElement>("span.ico[data-ico]")) {
    const drawn = markup(one.dataset.ico ?? "");
    if (!drawn) continue;
    const hue = one.dataset.hue ?? "";
    if (HUES.includes(hue)) one.classList.add(`ico-${hue}`);
    const glyph = made.createElementNS("http://www.w3.org/2000/svg", "svg");
    glyph.setAttribute("class", "glyph");
    glyph.setAttribute("viewBox", "0 0 24 24");
    glyph.setAttribute("aria-hidden", "true");
    glyph.innerHTML = drawn;
    one.replaceChildren(glyph);
  }
};

const STRANGER: Attaching = { stranger: true, apart: false, more: false };

export const pictures = (
  page: Document,
  live: () => boolean,
  attaching: Attaching = STRANGER,
): { held: (() => void)[]; lent: Lending[] } => {
  const held: (() => void)[] = [];
  const lent: Lending[] = [];
  for (const img of page.querySelectorAll<HTMLImageElement>(".page img[src]")) {
    const at = img.getAttribute("src") ?? "";
    if (!at || /^(https?|data|asset|blob|file|tauri):/i.test(at)) continue;
    if (!pictured(at)) {
      const drawnLive = attachedAs(page, img, at, live, held, attaching);
      if (drawnLive) lent.push(drawnLive);
      continue;
    }
    img.removeAttribute("src");
    held.push(showing(at));
    fetched(img, at, live);
  }
  return { held, lent };
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

export const widgets = (page: Document, stranger: boolean, reach?: Reaching): Lending[] => {
  const lent: Lending[] = [];
  for (const code of page.querySelectorAll<HTMLElement>(
    "figure.code > pre > code.language-widget",
  )) {
    const source = code.textContent ?? "";
    const figure = code.closest("figure");
    if (!figure || !source.trim()) continue;
    if (stranger) {
      figure.before(locked());
      continue;
    }
    const drawn = page.createElement("figure");
    drawn.className = "widget";
    drawn.dataset.at = figure.dataset.at ?? "";
    lents.set(drawn, { source });
    const bar = page.createElement("div");
    bar.className = "widget-bar";
    const name = page.createElement("span");
    name.className = "widget-name";
    name.textContent = figure.querySelector("figcaption")?.textContent ?? "";
    bar.append(name);
    if (reach?.onBlock) bar.append(asking(page, "code", t("codeShow")));
    if (reach?.onApart) bar.append(asking(page, "apart", t("viewApart")));
    const held = page.createElement("div");
    drawn.append(bar, held);
    figure.replaceWith(drawn);
    lent.push(lend(held, source));
  }
  return lent;
};

// A comment alone under its heading is the template's word on what goes there, shown until something does.
const hinted = (page: Document, body: string) => {
  const heading = (one: Element | null) =>
    !one || /^H[1-6]$/.test(one.tagName) || one.matches(".front");
  const bytes = new TextEncoder().encode(body);
  const decoder = new TextDecoder();
  for (const one of page.querySelectorAll<HTMLElement>(".page > div[data-at]")) {
    const at = spanned(one.dataset.at ?? null);
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
  add.dataset.after = last?.getAttribute("data-at") ?? `${size}-${size}`;
  add.dataset.seed = "## ";
  add.textContent = `+ ${t("addSection")}`;
  sheet.append(add);
  for (const label of sheet.querySelectorAll<HTMLElement>(":scope > :is(h1, h2, h3, h4, h5, h6)")) {
    const from = spanned(label.dataset.at ?? null);
    if (!from) continue;
    const hint = label.nextElementSibling?.matches(".hint, .hint-gone")
      ? spanned(label.nextElementSibling.getAttribute("data-at"))
      : null;
    const drop = page.createElement("button");
    drop.type = "button";
    drop.className = "drop-section";
    drop.dataset.at = `${from[0]}-${(hint ?? from)[1]}`;
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
    add.dataset.after = list.dataset.at;
    // A step added under a list takes its mark, or it would start a list of its own.
    const at = spanned(list.dataset.at ?? null);
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

export const worn = (
  page: Document,
  stranger: boolean,
  live: () => boolean,
  reach?: Reaching,
): Worn => {
  saidKinds(page, (kind) => t(`said${kind}` as Parameters<typeof t>[0]));
  if (reach?.onBlock) titled(page);
  if (reach?.onBlock && page.documentElement.classList.contains("untitled"))
    hinted(page, reach.body);
  datedSteps(page, reach?.stepDates);
  page.documentElement.classList.toggle("ticking", !!reach?.onTick);
  iconed(page);
  staged(page);
  if (reach?.onPart && !stranger) offeredAsParts(page, t("stepToPart"), reach.body);
  if (reach?.onWiden && !stranger) gripped(page);
  const shown = pictures(page, live, {
    stranger,
    apart: Boolean(reach?.onApart),
    more: Boolean(reach?.onMore) && !stranger,
  });
  const kept = shown.held;
  const lent = widgets(page, stranger, reach).concat(shown.lent);
  if (reach && !stranger) {
    carded(page, reach, live);
    partCarded(page, reach.part);
    webCarded(page, live);
  }
  colored(page, live);
  return { lent, kept };
};

export const answered = (page: Document, asked: Went, reach: Reaching, read = "") => {
  if (!asked) return;
  if (asked.kind === "doc") reach.onDoc(asked.id);
  if (asked.kind === "task") reach.onTask?.(asked.id);
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
  if (asked.kind === "tick")
    reach.onTick?.([charsAt(reach.body, asked.at[0]), charsAt(reach.body, asked.at[1])]);
  if (asked.kind === "block") {
    const block = page.querySelector<HTMLElement>(
      `.page > [data-at="${asked.at[0]}-${asked.at[1]}"]`,
    );
    const at: [number, number] = [
      charsAt(reach.body, asked.at[0]),
      charsAt(reach.body, asked.at[1]),
    ];
    if (block) reach.onBlock?.(at, block, "", read);
  }
  if (asked.kind === "hint") {
    const named = `${asked.at[0]}-${asked.at[1]}`;
    const held = page.querySelector<HTMLElement>(
      asked.from === "hint"
        ? `.page > .hint[data-at="${named}"]`
        : `.page > .${asked.from}[data-after="${named}"]`,
    );
    const end = charsAt(reach.body, asked.at[1]);
    if (held) reach.onBlock?.([end, end], held, asked.seed);
  }
  if (asked.kind === "cut")
    reach.onCut?.([charsAt(reach.body, asked.at[0]), charsAt(reach.body, asked.at[1])]);
  if (asked.kind === "section" && reach.onBlock) {
    const last = page.querySelector(`.page > [data-at="${asked.at[0]}-${asked.at[1]}"]`);
    const fresh = page.createElement("p");
    const add = last?.nextElementSibling;
    (add?.matches(".add-step") ? add : last)?.after(fresh);
    const end = charsAt(reach.body, asked.at[1]);
    reach.onBlock([end, end], fresh);
    if (fresh.isConnected) fresh.remove();
  }
  if (asked.kind === "title" && reach.onBlock) {
    const slot = page.querySelector<HTMLElement>(".page > h1[data-title]");
    if (slot) reach.onBlock([0, 0], slot, "# ");
  }
  if (asked.kind === "end" && reach.onBlock) {
    const sheet = page.querySelector(".page");
    if (!sheet) return;
    const fresh = page.createElement("p");
    sheet.append(fresh);
    reach.onBlock([reach.body.length, reach.body.length], fresh);
    if (fresh.isConnected) fresh.remove();
  }
  if (asked.kind === "more")
    reach.onMore?.({
      ...asked,
      at: [charsAt(reach.body, asked.at[0]), charsAt(reach.body, asked.at[1])],
    });
  if (asked.kind === "apart") {
    const block = page.querySelector(`.page > [data-at="${asked.at[0]}-${asked.at[1]}"]`);
    const drawn = widgetsIn(block)[asked.nth ?? 0];
    const lent = drawn ? lents.get(drawn) : undefined;
    if (lent) reach.onApart?.(lent);
  }
};

const below = (page: Document, target: Element, event: MouseEvent): boolean => {
  const sheet = page.querySelector(".page");
  if (!sheet || (target !== sheet && target !== page.body && target !== page.documentElement))
    return false;
  const blocks = sheet.querySelectorAll(":scope > :not(.cm-dock, .add-section)");
  const last = blocks[blocks.length - 1];
  return !last || event.clientY > last.getBoundingClientRect().bottom;
};

const readTo = (page: Document, event: MouseEvent): string => {
  const block = (event.target as Element | null)?.closest?.(".page > [data-at]");
  const caret = page.caretRangeFromPoint?.(event.clientX, event.clientY);
  if (!block || !caret || !block.contains(caret.startContainer)) return "";
  const range = page.createRange();
  range.setStart(block, 0);
  range.setEnd(caret.startContainer, caret.startOffset);
  return range.toString();
};

export const wired = (page: Document, reach: () => Reaching) => {
  const refused = (event: Event) => event.preventDefault();
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
    if (pressedIn) {
      pressedIn = false;
      return;
    }
    if (target.closest(".quick-bar, .mend-said, .cm-dock")) return;
    const selecting = !(page.getSelection()?.isCollapsed ?? true);
    if (target.closest("a")) event.preventDefault();
    const shaping = page.documentElement.classList.contains("shaping");
    const asked =
      went(target, selecting) ?? (!shaping && below(page, target, event) ? { kind: "end" } : null);
    if (!asked && !target.closest(".editing")) reach().onAway?.();
    answered(page, asked, reach(), asked?.kind === "block" ? readTo(page, event) : "");
  });
  widening(page, () => {
    const now = reach();
    const widen = now.onWiden;
    if (!widen) return undefined;
    return (at, column, dashes) =>
      widen([charsAt(now.body, at[0]), charsAt(now.body, at[1])], column, dashes);
  });
  quickly(page, () => {
    const now = reach();
    const chars = (at: [number, number]): [number, number] => [
      charsAt(now.body, at[0]),
      charsAt(now.body, at[1]),
    ];
    const { onFormat, onMenu } = now;
    return {
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
  // The window's own shortcuts listen on the app, which never hears a key pressed in the page.
  page.addEventListener("keydown", (event) => {
    const writing = (event.target as Element | null)?.closest?.(".editing");
    const asked = event.ctrlKey || event.metaKey || event.key === "F1";
    if (writing && (event.defaultPrevented || !asked)) return;
    const passed = new KeyboardEvent("keydown", event);
    window.dispatchEvent(passed);
    if (passed.defaultPrevented) event.preventDefault();
  });
  page.addEventListener("widgetdoc", (event) => {
    const id = (event as CustomEvent<string>).detail;
    if (id) reach().onDoc(id);
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
  for (const one of page.querySelectorAll<HTMLElement>(".page li.task")) {
    const own = one.cloneNode(true) as HTMLElement;
    for (const inner of own.querySelectorAll("ul, ol")) inner.remove();
    const words = bare(own.textContent ?? "");
    const said = known.find(
      ([key, , cut]) => key && (words === key || (cut && words.startsWith(key))),
    )?.[1];
    if (said) one.title = said;
    else one.removeAttribute("title");
  }
};
