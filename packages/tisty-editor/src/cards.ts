import { markup } from "./glyphs";

export type Carded =
  | { readonly kind: "doc"; readonly title: string }
  | { readonly kind: "page"; readonly title: string; readonly at: number; readonly of: number }
  | { readonly kind: "away"; readonly title: string }
  | { readonly kind: "self"; readonly title: string }
  | { readonly kind: "gone" };

export type CardWord =
  | "cardAway"
  | "cardHere"
  | "cardGone"
  | "kindDoc"
  | "untitledDoc"
  | "docItself"
  | "goneDoc"
  | "leafOfMany";

// The sheet holds no documents: whoever shows it says what each card names and in which language.
export type Cards = {
  readonly card: (file: string) => Carded;
  readonly blurb?: (file: string) => Promise<string>;
  readonly say: (word: CardWord, ...filled: string[]) => string;
  readonly controls?: (file: string) => Node[];
};

export const piece = (page: Document, kind: string, said = "") => {
  const one = page.createElement("span");
  one.className = kind;
  one.textContent = said;
  return one;
};

export const glyph = (page: Document, name: string) => {
  const drawn = page.createElementNS("http://www.w3.org/2000/svg", "svg");
  drawn.setAttribute("class", "glyph");
  drawn.setAttribute("viewBox", "0 0 24 24");
  drawn.setAttribute("aria-hidden", "true");
  drawn.innerHTML = markup(name) ?? "";
  return drawn;
};

const GLYPH = { doc: "page", page: "book", away: "archive", self: "page", gone: "page" } as const;

const metaOf = (page: Document, seen: Carded, say: Cards["say"]) => {
  if (seen.kind === "page")
    return piece(page, "chip", say("leafOfMany", `${seen.at}`, `${seen.of}`));
  if (seen.kind === "away") return piece(page, "chip away", say("cardAway"));
  if (seen.kind === "self") return piece(page, "chip here", say("cardHere"));
  if (seen.kind === "gone") return piece(page, "chip gone", say("cardGone"));
  return piece(page, "kind", say("kindDoc"));
};

export const cardBox = (page: Document, kind: string, open: boolean) => {
  const box = page.createElement("span");
  box.className = `doc-card ${kind}`;
  if (!open) return box;
  box.tabIndex = 0;
  box.setAttribute("role", "link");
  box.addEventListener("keydown", (event) => {
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    box.click();
  });
  return box;
};

// A card alone in its paragraph is drawn whole; one inside a sentence stays a word of it.
export const carded = (page: Document, cards: Cards, live: () => boolean) => {
  for (const link of page.querySelectorAll<HTMLAnchorElement>(".page > p > a.card[data-doc]")) {
    const held = link.parentElement;
    if (!held || held.children.length !== 1) continue;
    if ((held.textContent ?? "").trim() !== (link.textContent ?? "").trim()) continue;
    const file = link.dataset.doc ?? "";
    const seen = cards.card(file);
    const open = seen.kind !== "self" && seen.kind !== "gone";
    const box = cardBox(page, `is-${seen.kind}`, open);
    if (open) box.dataset.doc = file;
    const tile = piece(page, "card-tile");
    tile.append(glyph(page, GLYPH[seen.kind]));
    const name = seen.kind === "gone" ? link.textContent || cards.say("untitledDoc") : seen.title;
    const blurb = piece(page, "card-blurb");
    if (seen.kind === "self") blurb.textContent = cards.say("docItself");
    if (seen.kind === "gone") blurb.textContent = cards.say("goneDoc");
    const meta = piece(page, "card-meta");
    meta.append(metaOf(page, seen, cards.say), ...(cards.controls?.(file) ?? []));
    if (open) meta.append(piece(page, "card-go", "›"));
    box.append(tile, piece(page, "card-name", name), blurb, meta);
    link.replaceWith(box);
    if (open && cards.blurb)
      cards
        .blurb(file)
        .then((said) => {
          if (live()) blurb.textContent = said;
        })
        .catch(() => {});
  }
};
