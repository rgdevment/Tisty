import { fill, t } from "../locales";
import { drawn, piece } from "./carding";

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

// A part alone in its paragraph is drawn as its card, as a document's is; inside a sentence it stays a word.
export const partCarded = (page: Document, part: ((id: string) => PartCarded) | undefined) => {
  if (!part) return;
  for (const link of page.querySelectorAll<HTMLAnchorElement>(".page > p > a.card[data-task]")) {
    const held = link.parentElement;
    if (held?.children.length !== 1) continue;
    if ((held.textContent ?? "").trim() !== (link.textContent ?? "").trim()) continue;
    const id = link.dataset.task ?? "";
    const seen = part(id);
    const box = page.createElement("span");
    box.className = `doc-card is-part is-${seen.kind}`;
    box.dataset.task = id;
    box.tabIndex = 0;
    box.setAttribute("role", "link");
    box.addEventListener("keydown", (event) => {
      if (event.key !== "Enter" && event.key !== " ") return;
      event.preventDefault();
      box.click();
    });
    const tile = piece(page, "card-tile");
    tile.append(drawn(page, seen.kind === "closed" ? "done" : "todo"));
    const name =
      seen.kind === "loose" || seen.kind === "waiting" ? link.textContent || "" : seen.title;
    const when = seen.kind === "open" ? (seen.when ?? "") : "";
    const meta = piece(page, "card-meta");
    meta.append(chipOf(page, seen), piece(page, "card-go", "›"));
    box.append(tile, piece(page, "card-name", name), piece(page, "card-blurb", when), meta);
    link.replaceWith(box);
  }
};
