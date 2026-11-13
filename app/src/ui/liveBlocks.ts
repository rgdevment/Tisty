import type { Hosted } from "tisty-editor/edit";
import { charsAt, spanned } from "tisty-editor/read";
import { celledBox, diagrams, HUES, today } from "tisty-sheet";
import { docDrawn } from "../core";
import { locale } from "../locales";
import { picturesWithin } from "./dressing";
import { SAY } from "./saying";
import { dressedTable } from "./tabled";
import { darkNow } from "./theming";
import { gripsOn } from "./widening";

// A copy keeps no page bytes, or the page's own handlers would act on a block that moved.
const loose = (node: Node): Node => {
  if (node instanceof Element) {
    node.removeAttribute("data-at");
    for (const one of node.querySelectorAll("[data-at]")) one.removeAttribute("data-at");
  }
  return node;
};

const dressed = (into: HTMLElement, measured: () => void) => {
  const live = () => into.isConnected;
  picturesWithin(into, live);
  for (const box of into.querySelectorAll(".table")) {
    dressedTable(into.ownerDocument, box);
    celledBox(into.ownerDocument, box, {
      today: today(),
      locale: locale(),
      hues: HUES,
      live: true,
      say: SAY,
    });
    gripsOn(into.ownerDocument, box.querySelectorAll<HTMLElement>("thead th"));
  }
  for (const img of into.querySelectorAll("img")) img.addEventListener("load", measured);
  void diagrams(into, darkNow(), live).then(measured);
};

const drawnApart = (page: Document, source: string, into: HTMLElement, measured: () => void) => {
  into.textContent = source;
  into.classList.add("cm-rich-waiting");
  docDrawn(source)
    .then((html) => {
      if (!into.isConnected) return;
      const held = page.createElement("template");
      held.innerHTML = html;
      const blocks = Array.from(held.content.children).filter((one) => !one.matches(".front"));
      into.replaceChildren(...blocks.map(loose));
      into.classList.remove("cm-rich-waiting");
      measured();
      dressed(into, measured);
    })
    .catch(() => {});
};

export const blockDrawer = (page: Document, body: string): NonNullable<Hosted["drawBlock"]> => {
  let drawn: Map<string, Element> | null = null;
  const known = () => {
    if (drawn) return drawn;
    drawn = new Map();
    for (const one of page.querySelectorAll<HTMLElement>(".page > [data-at]")) {
      const at = spanned(one.dataset.at ?? null);
      if (at) drawn.set(body.slice(charsAt(body, at[0]), charsAt(body, at[1])).trim(), one);
    }
    return drawn;
  };
  return (source, into, measured) => {
    const found = known().get(source.trim());
    if (!found) return drawnApart(page, source, into, measured);
    into.replaceChildren(loose(found.cloneNode(true)));
    dressed(into, measured);
  };
};
