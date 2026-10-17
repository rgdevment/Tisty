import type { Mend } from "../core";
import { fill, t } from "../locales";
import { charsAt } from "./paging";

export type Said = { readonly code?: string; readonly words?: string };

const SHOWN_AT_MOST = 60;

const short = (said: string) =>
  said.length > SHOWN_AT_MOST ? `${said.slice(0, SHOWN_AT_MOST - 1)}…` : said;

export const told = (mends: readonly Mend[]): { parts: Said[]; all: string } => {
  const pieces = mends.flatMap((one) => one.pieces);
  const all = pieces.map((one) => `${one.said} → ${one.fixed}`).join("\n");
  const [one] = pieces;
  if (pieces.length > 1)
    return { parts: [{ words: fill("unspokenMany", String(pieces.length)) }], all };
  if (!one) return { parts: [{ words: t("unspoken") }], all };
  const written = one.fixed ? [{ words: ` · ${t("writtenAs")} ` }, { code: short(one.fixed) }] : [];
  return {
    parts: [{ code: short(one.said) }, { words: ` ${t("unspokenIs")}` }, ...written],
    all: "",
  };
};

// Said under the block and never inside it, so its words are not read as the block's own.
export const mendable = (page: Document, mends: readonly Mend[], mend: (one: Mend) => void) => {
  for (const said of page.querySelectorAll(".mend-said")) said.remove();
  for (const one of mends) {
    const block = page.querySelector<HTMLElement>(`.page > [data-at^="${one.from}-"]`);
    if (!block) continue;
    const said = page.createElement("div");
    said.className = "mend-said";
    const { parts, all } = told([one]);
    const words = page.createElement("span");
    for (const part of parts) {
      if (part.code === undefined) words.append(page.createTextNode(part.words ?? ""));
      else {
        const code = page.createElement("code");
        code.textContent = part.code;
        words.append(code);
      }
    }
    if (all) words.title = all;
    const button = page.createElement("button");
    button.type = "button";
    button.textContent = t("mendIt");
    button.addEventListener("click", () => mend(one));
    said.append(words, button);
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
