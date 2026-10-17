import type { Mend } from "../core";
import { t } from "../locales";
import { charsAt } from "./paging";

// Said under the block and never inside it, so its words are not read as the block's own.
export const mendable = (page: Document, mends: readonly Mend[], mend: (one: Mend) => void) => {
  for (const said of page.querySelectorAll(".mend-said")) said.remove();
  for (const one of mends) {
    const block = page.querySelector<HTMLElement>(`.page > [data-at^="${one.from}-"]`);
    if (!block) continue;
    const said = page.createElement("div");
    said.className = "mend-said";
    said.append(page.createTextNode(t("unspoken")));
    const button = page.createElement("button");
    button.type = "button";
    button.textContent = t("mendIt");
    button.addEventListener("click", () => mend(one));
    said.append(button);
    block.after(said);
  }
};

// Mended from the end, so each one still names the bytes it was found at.
export const mended = (text: string, mends: readonly Mend[]): string =>
  [...mends]
    .sort((a, b) => b.from - a.from)
    .reduce(
      (now, one) =>
        `${now.slice(0, charsAt(text, one.from))}${one.fixed}${now.slice(charsAt(text, one.to))}`,
      text,
    );
