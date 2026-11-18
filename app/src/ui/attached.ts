import { convertFileSrc } from "@tauri-apps/api/core";
import { attachedAs as drawnAs } from "tisty-sheet";
import { keptHere, weighs } from "../core";
import { askAgain, serve, showing } from "../inUse";
import { t, type Word } from "../locales";
import { named } from "../previews";
import { plainly } from "../refusal";
import { placeCopied } from "./pagePlaces";

export { pressed } from "tisty-sheet";

const more = (page: Document, target: string) => {
  const one = page.createElement("button");
  one.type = "button";
  one.className = "card-more";
  one.dataset.asked = "more";
  one.dataset.target = target;
  one.setAttribute("aria-label", t("cardMore"));
  one.title = t("cardMore");
  one.textContent = "⋯";
  return one;
};

export type Attaching = {
  readonly more: boolean;
};

export const attachedAs = (
  page: Document,
  img: Element,
  at: string,
  live: () => boolean,
  held: (() => void)[],
  { more: asking }: Attaching,
): void =>
  drawnAs(page, img, at, live, held, {
    serve,
    url: convertFileSrc,
    keptHere,
    weighs,
    again: askAgain,
    showing,
    plainly,
    say: (word) => t(word as Word),
    named,
    placed: (from, shown) => placeCopied(from, shown, "data-src", "data-placed"),
    more: asking ? more : undefined,
  });
