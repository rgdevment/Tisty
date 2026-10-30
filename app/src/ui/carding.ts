import { type Carded, type CardWord, carded as drawCards } from "tisty-editor/read";
import { docRead, type Filed } from "../core";
import { fill, t } from "../locales";
import { DOC } from "../markdown";
import { inTextOrder, pagesOf } from "../paging";

const said = (word: CardWord, ...filled: string[]): string =>
  filled.length ? fill(word, filled[0], filled[1]) : t(word);

const spanless = (line: string): string => line.replace(/::([a-z0-9-]+)(?: [a-z]+)?::/g, ":$1:");

export const glimpsed = (body: string): string =>
  body
    .split("\n")
    .slice(1)
    .map((line) =>
      spanless(line)
        .replace(/^[#>\s*+-]+/, "")
        .trim(),
    )
    .filter(Boolean)
    .join(" · ")
    .slice(0, 160);

export const cardsOf = (known: Filed[], here: string | undefined, told: string[]) => {
  const pages = inTextOrder(pagesOf(known, here), told).filter((one) => !one.archived && !one.away);
  return (file: string): Carded => {
    const one = known.find((doc) => doc.file === file);
    if (!one || one.gone) return { kind: "gone" };
    const title = one.title.trim() || t("untitledDoc");
    if (file === here) return { kind: "self", title };
    if (one.archived || one.away) return { kind: "away", title };
    const at = pages.findIndex((page) => page.file === file);
    return at < 0 ? { kind: "doc", title } : { kind: "page", title, at: at + 1, of: pages.length };
  };
};

// A blurb is read once per version of its document, however many cards or redraws ask for it.
export const blurbing = () => {
  const kept = new Map<string, { wrote: string; said: Promise<string> }>();
  return (one: Filed | undefined): Promise<string> => {
    if (!one) return Promise.resolve("");
    const wrote = one.wrote ?? (one.bytes == null ? null : `${one.bytes}`);
    const was = kept.get(one.file);
    if (was && wrote !== null && was.wrote === wrote) return was.said;
    const said = docRead(one.file)
      .then(glimpsed)
      .catch(() => "");
    if (wrote !== null) kept.set(one.file, { wrote, said });
    return said;
  };
};

export type Carding = {
  readonly card?: (file: string) => Carded;
  readonly blurb?: (file: string) => Promise<string>;
  readonly onBlock?: unknown;
  readonly onMore?: unknown;
};

export const more = (page: Document, target: string) => {
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

// The sheet draws the cards; the window says what each names, in its language, and adds the editor's own controls.
export const carded = (page: Document, reach: Carding, live: () => boolean) => {
  if (!reach.card) return;
  const controls = (file: string): Node[] => {
    const made: Node[] = [];
    if (reach.onBlock) {
      const edit = page.createElement("button");
      edit.type = "button";
      edit.className = "card-edit";
      edit.dataset.asked = "code";
      edit.textContent = t("cardEdit");
      made.push(edit);
    }
    if (reach.onMore) made.push(more(page, `${DOC}${file}`));
    return made;
  };
  drawCards(page, { card: reach.card, blurb: reach.blurb, say: said, controls }, live);
};
