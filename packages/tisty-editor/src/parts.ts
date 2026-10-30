import { cardBox, glyph, piece } from "./cards";
import type { Say } from "./words";

export type PartCarded =
  | {
      readonly kind: "open";
      readonly title: string;
      readonly done: number;
      readonly steps: number;
      readonly when?: string;
    }
  | { readonly kind: "closed"; readonly title: string; readonly dropped: boolean }
  | { readonly kind: "loose" }
  | { readonly kind: "waiting" };

export type PartCards = { readonly part: (id: string) => PartCarded; readonly say: Say };

const chipOf = (page: Document, seen: PartCarded, say: Say) => {
  if (seen.kind === "closed")
    return piece(page, "chip here", say(seen.dropped ? "partCardDropped" : "partCardDone"));
  if (seen.kind === "loose") return piece(page, "chip gone", say("partCardLoose"));
  if (seen.kind === "waiting") return piece(page, "kind", say("partCardKind"));
  if (seen.steps > 0)
    return piece(page, "chip", say("partCardSteps", `${seen.done}`, `${seen.steps}`));
  return piece(page, "kind", say("partCardKind"));
};

const carded = (page: Document, id: string, written: string, { part, say }: PartCards) => {
  const seen = part(id);
  const box = cardBox(page, `is-part is-${seen.kind}`, true);
  box.dataset.task = id;
  box.dataset.written = written;
  const tile = piece(page, "card-tile");
  tile.append(glyph(page, seen.kind === "closed" ? "done" : "todo"));
  const name = seen.kind === "loose" || seen.kind === "waiting" ? written : seen.title;
  const when = seen.kind === "open" ? (seen.when ?? "") : "";
  const meta = piece(page, "card-meta");
  meta.append(chipOf(page, seen, say), piece(page, "card-go", "›"));
  box.append(tile, piece(page, "card-name", name), piece(page, "card-blurb", when), meta);
  return box;
};

// A part alone in its paragraph is drawn as its card, as a document's is; inside a sentence it stays a word.
export const partCarded = (page: Document, parts: PartCards) => {
  for (const link of page.querySelectorAll<HTMLAnchorElement>(".page > p > a.card[data-task]")) {
    const held = link.parentElement;
    if (held?.children.length !== 1) continue;
    if ((held.textContent ?? "").trim() !== (link.textContent ?? "").trim()) continue;
    link.replaceWith(carded(page, link.dataset.task ?? "", link.textContent || "", parts));
  }
};

// The parts are asked for after the page is drawn, so their cards change in place rather than drawing it again.
export const recarded = (page: Document, parts: PartCards) => {
  for (const box of page.querySelectorAll<HTMLElement>(".page > p > .is-part[data-task]")) {
    const fresh = carded(page, box.dataset.task ?? "", box.dataset.written ?? "", parts);
    const focused = page.activeElement === box;
    box.replaceWith(fresh);
    if (focused) fresh.focus();
  }
};
