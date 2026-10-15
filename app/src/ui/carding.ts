import { docRead, type Filed } from "../core";
import { markup } from "../glyphs";
import { fill, t } from "../locales";
import { inTextOrder, pagesOf } from "../paging";

export type Carded =
  | { readonly kind: "doc"; readonly title: string }
  | { readonly kind: "page"; readonly title: string; readonly at: number; readonly of: number }
  | { readonly kind: "away"; readonly title: string }
  | { readonly kind: "self"; readonly title: string }
  | { readonly kind: "gone" };

const spanless = (line: string): string =>
  line.replace(/<span[^>]*data-ico="([^"]*)"[^>]*>[\s\S]*?<\/span>/g, ":$1:");

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
};

const piece = (page: Document, kind: string, said = "") => {
  const one = page.createElement("span");
  one.className = kind;
  one.textContent = said;
  return one;
};

const drawn = (page: Document, name: string) => {
  const glyph = page.createElementNS("http://www.w3.org/2000/svg", "svg");
  glyph.setAttribute("class", "glyph");
  glyph.setAttribute("viewBox", "0 0 24 24");
  glyph.setAttribute("aria-hidden", "true");
  glyph.innerHTML = markup(name) ?? "";
  return glyph;
};

const GLYPH = { doc: "page", page: "book", away: "archive", self: "page", gone: "page" } as const;

const metaOf = (page: Document, seen: Carded) => {
  if (seen.kind === "page")
    return piece(page, "chip", fill("leafOfMany", `${seen.at}`, `${seen.of}`));
  if (seen.kind === "away") return piece(page, "chip away", t("cardAway"));
  if (seen.kind === "self") return piece(page, "chip here", t("cardHere"));
  if (seen.kind === "gone") return piece(page, "chip gone", t("cardGone"));
  return piece(page, "kind", t("kindDoc"));
};

// A card alone in its paragraph is drawn whole; one inside a sentence stays a word of it.
export const carded = (page: Document, reach: Carding, live: () => boolean) => {
  if (!reach.card) return;
  for (const link of page.querySelectorAll<HTMLAnchorElement>(".page > p > a.card[data-doc]")) {
    const held = link.parentElement;
    if (!held || held.children.length !== 1) continue;
    if ((held.textContent ?? "").trim() !== (link.textContent ?? "").trim()) continue;
    const file = link.dataset.doc ?? "";
    const seen = reach.card(file);
    const open = seen.kind !== "self" && seen.kind !== "gone";
    const box = page.createElement("span");
    box.className = `doc-card is-${seen.kind}`;
    if (open) {
      box.dataset.doc = file;
      box.tabIndex = 0;
      box.setAttribute("role", "link");
      box.addEventListener("keydown", (event) => {
        if (event.key !== "Enter" && event.key !== " ") return;
        event.preventDefault();
        box.click();
      });
    }
    const tile = piece(page, "card-tile");
    tile.append(drawn(page, GLYPH[seen.kind]));
    const name = seen.kind === "gone" ? link.textContent || t("untitledDoc") : seen.title;
    const blurb = piece(page, "card-blurb");
    if (seen.kind === "self") blurb.textContent = t("docItself");
    if (seen.kind === "gone") blurb.textContent = t("goneDoc");
    const meta = piece(page, "card-meta");
    meta.append(metaOf(page, seen));
    if (reach.onBlock) {
      const edit = page.createElement("button");
      edit.type = "button";
      edit.className = "card-edit";
      edit.dataset.asked = "code";
      edit.textContent = t("cardEdit");
      meta.append(edit);
    }
    if (open) meta.append(piece(page, "card-go", "›"));
    box.append(tile, piece(page, "card-name", name), blurb, meta);
    link.replaceWith(box);
    if (open && reach.blurb)
      reach
        .blurb(file)
        .then((said) => {
          if (live()) blurb.textContent = said;
        })
        .catch(() => {});
  }
};
