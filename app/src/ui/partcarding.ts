import { fill, t } from "../locales";
import { cardBox, drawn, piece } from "./carding";

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

const chipOf = (page: Document, seen: PartCarded) => {
  if (seen.kind === "closed")
    return piece(page, "chip here", t(seen.dropped ? "partCardDropped" : "partCardDone"));
  if (seen.kind === "loose") return piece(page, "chip gone", t("partCardLoose"));
  if (seen.kind === "waiting") return piece(page, "kind", t("partCardKind"));
  if (seen.steps > 0)
    return piece(page, "chip", fill("partCardSteps", `${seen.done}`, `${seen.steps}`));
  return piece(page, "kind", t("partCardKind"));
};

const carded = (page: Document, id: string, written: string, seen: PartCarded) => {
  const box = cardBox(page, `is-part is-${seen.kind}`, true);
  box.dataset.task = id;
  box.dataset.written = written;
  const tile = piece(page, "card-tile");
  tile.append(drawn(page, seen.kind === "closed" ? "done" : "todo"));
  const name = seen.kind === "loose" || seen.kind === "waiting" ? written : seen.title;
  const when = seen.kind === "open" ? (seen.when ?? "") : "";
  const meta = piece(page, "card-meta");
  meta.append(chipOf(page, seen), piece(page, "card-go", "›"));
  box.append(tile, piece(page, "card-name", name), piece(page, "card-blurb", when), meta);
  return box;
};

// A part alone in its paragraph is drawn as its card, as a document's is; inside a sentence it stays a word.
export const partCarded = (page: Document, part: ((id: string) => PartCarded) | undefined) => {
  if (!part) return;
  for (const link of page.querySelectorAll<HTMLAnchorElement>(".page > p > a.card[data-task]")) {
    const held = link.parentElement;
    if (held?.children.length !== 1) continue;
    if ((held.textContent ?? "").trim() !== (link.textContent ?? "").trim()) continue;
    const id = link.dataset.task ?? "";
    link.replaceWith(carded(page, id, link.textContent || "", part(id)));
  }
};

// The parts are asked for after the page is drawn, so their cards change in place rather than drawing it again.
export const recarded = (page: Document, part: ((id: string) => PartCarded) | undefined) => {
  if (!part) return;
  for (const box of page.querySelectorAll<HTMLElement>(".page > p > .is-part[data-task]")) {
    const id = box.dataset.task ?? "";
    box.replaceWith(carded(page, id, box.dataset.written ?? "", part(id)));
  }
};
