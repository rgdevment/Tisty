import type { Hosted } from "tisty-editor/edit";
import { charsAt, spanned } from "tisty-editor/read";
import { celledBox, diagrams, HUES, today } from "tisty-sheet";
import { docDrawn } from "../core";
import { locale } from "../locales";
import { diagramBar, picturesWithin } from "./dressing";
import { SAY } from "./saying";
import { dressedTable } from "./tabled";
import { darkNow } from "./theming";
import { gripsOn } from "./widening";
import { lend } from "./widgeting";

// A copy keeps no page bytes, or the page's handlers would act on a moved block; the frame's nodes fail instanceof.
const loose = (node: Node): Node => {
  if (node.nodeType !== Node.ELEMENT_NODE) return node;
  const element = node as Element;
  element.removeAttribute("data-at");
  for (const one of element.querySelectorAll("[data-at]")) one.removeAttribute("data-at");
  return node;
};

// The bar's Markdown button is gone here: the Markdown view is the one way to the words.
const barred = (into: HTMLElement) => {
  for (const one of into.querySelectorAll('[data-asked="diagramMarkdown"]')) one.remove();
  for (const figure of into.querySelectorAll<HTMLElement>("figure.diagram"))
    diagramBar(figure, { open: true, out: true, words: false });
};

const tableDressed = (page: Document, box: Element) => {
  dressedTable(page, box);
  celledBox(page, box, {
    today: today(),
    locale: locale(),
    hues: HUES,
    live: true,
    say: SAY,
  });
  gripsOn(page, box.querySelectorAll<HTMLElement>("thead th"));
};

const dressed = (into: HTMLElement, measured: () => void) => {
  const live = () => into.isConnected;
  picturesWithin(into, live);
  barred(into);
  for (const box of into.querySelectorAll(".table")) tableDressed(into.ownerDocument, box);
  for (const img of into.querySelectorAll("img")) img.addEventListener("load", measured);
  void diagrams(into, darkNow(), live).then(measured);
};

const within = (laid: readonly Element[], selector: string) =>
  laid.flatMap((one) => [
    ...(one.matches(selector) ? [one] : []),
    ...one.querySelectorAll(selector),
  ]);

// Blocks drawn again while the page is written on are dressed alone; a diagram already drawn is left as it is.
export const dressedLaid = (sheet: Element, laid: readonly Element[]) => {
  const live = () => sheet.isConnected;
  for (const one of laid) picturesWithin(one, live);
  for (const figure of within(laid, "figure.diagram"))
    diagramBar(figure as HTMLElement, { open: true, out: true, words: false });
  for (const box of within(laid, ".table")) tableDressed(sheet.ownerDocument, box);
  void diagrams(sheet, darkNow(), live);
};

const FRESH_AT_MOST = 32;
const WIDGET = /^(`{3,}|~{3,})widget[^\n]*\n([\s\S]*?)\n\1\s*$/;

// A live widget runs in its own frame, lent for as long as its block is on screen and handed back after.
const lentWidget = (into: HTMLElement, source: string, measured: () => void) => {
  const code = WIDGET.exec(source.replace(/\r\n/g, "\n").trim())?.[2];
  if (code === undefined) return undefined;
  const drawn = into.ownerDocument.createElement("figure");
  drawn.className = "widget";
  const held = into.ownerDocument.createElement("div");
  drawn.append(held);
  into.replaceChildren(drawn);
  const lent = lend(held, code, () => measured());
  measured();
  return lent.drop;
};

const keyOf = (source: string) => source.replace(/\r\n/g, "\n").trim();

const placed = (into: HTMLElement, html: string, measured: () => void) => {
  const held = into.ownerDocument.createElement("template");
  held.innerHTML = html;
  const blocks = Array.from(held.content.children).filter((one) => !one.matches(".front"));
  into.replaceChildren(...blocks.map(loose));
  into.classList.remove("cm-rich-waiting");
  measured();
  dressed(into, measured);
};

export const blockDrawer = (page: Document, body: string): NonNullable<Hosted["drawBlock"]> => {
  let onPage: Map<string, Element> | null = null;
  // A block drawn once by the core is kept, so scrolling it back into view asks for nothing.
  const fresh = new Map<string, string>();
  const known = () => {
    if (onPage) return onPage;
    onPage = new Map();
    for (const one of page.querySelectorAll<HTMLElement>(".page > [data-at]")) {
      const at = spanned(one.dataset.at ?? null);
      if (at) onPage.set(keyOf(body.slice(charsAt(body, at[0]), charsAt(body, at[1]))), one);
    }
    return onPage;
  };
  return (source, into, measured) => {
    const widget = lentWidget(into, source, measured);
    if (widget) return widget;
    const key = keyOf(source);
    const found = known().get(key);
    if (found) {
      into.replaceChildren(loose(found.cloneNode(true)));
      dressed(into, measured);
      return undefined;
    }
    const kept = fresh.get(key);
    if (kept !== undefined) {
      placed(into, kept, measured);
      return undefined;
    }
    into.textContent = source;
    into.classList.add("cm-rich-waiting");
    docDrawn(source)
      .then((html) => {
        fresh.set(key, html);
        if (fresh.size > FRESH_AT_MOST) fresh.delete(fresh.keys().next().value as string);
        if (into.isConnected) placed(into, html, measured);
      })
      .catch(() => {});
    return undefined;
  };
};
