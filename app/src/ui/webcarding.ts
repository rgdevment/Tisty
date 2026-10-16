import { type Glimpse, glimpseFetch, glimpseKept } from "../core";
import { markup } from "../glyphs";
import { t } from "../locales";
import { hosting } from "../previews";

const piece = (page: Document, kind: string, said = "") => {
  const one = page.createElement("span");
  one.className = kind;
  one.textContent = said;
  return one;
};

const globe = (page: Document) => {
  const glyph = page.createElementNS("http://www.w3.org/2000/svg", "svg");
  glyph.setAttribute("class", "glyph");
  glyph.setAttribute("viewBox", "0 0 24 24");
  glyph.setAttribute("aria-hidden", "true");
  glyph.innerHTML = markup("globe-earth") ?? "";
  return glyph;
};

const told = (said: Glimpse | null): said is Glimpse =>
  Boolean(said && (said.title || said.said || said.shot));

const shown = (
  page: Document,
  parts: { tile: HTMLElement; name: HTMLElement; blurb: HTMLElement },
  said: Glimpse,
) => {
  if (said.title) parts.name.textContent = said.title;
  if (said.said) parts.blurb.textContent = said.said;
  if (said.shot) {
    const shot = page.createElement("img");
    shot.className = "card-shot";
    shot.alt = "";
    shot.src = said.shot;
    parts.tile.replaceChildren(shot);
  }
};

// A web link alone in its paragraph is a card; its preview comes only from the network when asked.
export const webCarded = (page: Document, live: () => boolean) => {
  for (const link of page.querySelectorAll<HTMLAnchorElement>(".page > p > a[href]")) {
    const held = link.parentElement;
    const at = link.getAttribute("href") ?? "";
    const host = hosting(at);
    if (!held || !host || held.children.length !== 1) continue;
    if ((held.textContent ?? "").trim() !== (link.textContent ?? "").trim()) continue;
    const box = page.createElement("a");
    box.className = "doc-card is-web";
    box.href = at;
    const tile = piece(page, "card-tile");
    tile.append(globe(page));
    const name = piece(page, "card-name", link.textContent?.trim() || host);
    const blurb = piece(page, "card-blurb");
    const meta = piece(page, "card-meta");
    meta.append(piece(page, "kind", host));
    const ask = page.createElement("button");
    ask.type = "button";
    ask.className = "card-edit";
    ask.textContent = t("linkGlimpse");
    ask.hidden = true;
    ask.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      ask.disabled = true;
      glimpseFetch(at)
        .then((said) => {
          if (!live()) return;
          ask.hidden = told(said);
          ask.disabled = false;
          if (told(said)) shown(page, { tile, name, blurb }, said);
        })
        .catch(() => {
          ask.disabled = false;
        });
    });
    meta.append(ask, piece(page, "card-go", "›"));
    box.append(tile, name, blurb, meta);
    link.replaceWith(box);
    glimpseKept(at)
      .then((said) => {
        if (!live()) return;
        if (told(said)) shown(page, { tile, name, blurb }, said);
        else ask.hidden = false;
      })
      .catch(() => {
        if (live()) ask.hidden = false;
      });
  }
};
