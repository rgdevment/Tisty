import { piece } from "./cards";
import { HUES } from "./shell";
import type { Say } from "./words";

export type Glimpse = { readonly title?: string; readonly said?: string; readonly shot?: string };

// The sheet keeps no site's glimpse: the window brings the ones it kept and asks for one when told, the web has none.
export type Glimpses = {
  readonly kept: (at: string) => Promise<Glimpse | null>;
  readonly fetched: (at: string) => Promise<Glimpse | null>;
};

export const hosting = (href: string): string | null => {
  try {
    const said = new URL(href);
    if (said.protocol !== "http:" && said.protocol !== "https:") return null;
    return said.hostname.replace(/^www\./, "") || null;
  } catch {
    return null;
  }
};

const told = (said: Glimpse | null): said is Glimpse =>
  Boolean(said && (said.title || said.said || said.shot));

// A link that brings nothing of its own still reads apart from its neighbours: its site's letter in its site's colour.
const lettered = (page: Document, host: string): HTMLElement => {
  const tile = piece(page, "card-tile is-letter", host.charAt(0).toUpperCase());
  let sum = 0;
  for (const one of host) sum += one.charCodeAt(0);
  tile.style.setProperty("--hue", `var(--tisty-hue-${HUES[sum % HUES.length]})`);
  return tile;
};

const readable = (at: string, host: string): string => {
  try {
    const path = new URL(at).pathname.replace(/\/+$/, "");
    return `${host}${path}`;
  } catch {
    return host;
  }
};

type Parts = { box: HTMLElement; tile: HTMLElement; name: HTMLElement; blurb: HTMLElement };

const shown = (page: Document, parts: Parts, said: Glimpse) => {
  if (said.title) parts.name.textContent = said.title;
  if (said.said) parts.blurb.textContent = said.said;
  if (said.title || said.said) parts.box.classList.remove("is-bare");
  if (said.shot) {
    const shot = page.createElement("img");
    shot.className = "card-thumb";
    shot.alt = "";
    shot.loading = "lazy";
    shot.src = said.shot;
    parts.tile.remove();
    parts.box.classList.add("has-shot");
    parts.box.classList.remove("is-bare");
    parts.box.append(shot);
  }
};

const asking = (
  page: Document,
  at: string,
  parts: Parts,
  said: Say,
  glimpses: Glimpses,
  live: () => boolean,
) => {
  const ask = page.createElement("button");
  ask.type = "button";
  ask.className = "card-edit";
  ask.textContent = said("linkGlimpse");
  ask.hidden = true;
  ask.addEventListener("click", (event) => {
    event.preventDefault();
    event.stopPropagation();
    ask.disabled = true;
    glimpses
      .fetched(at)
      .then((brought) => {
        if (!live()) return;
        ask.hidden = told(brought);
        ask.disabled = false;
        if (told(brought)) shown(page, parts, brought);
      })
      .catch(() => {
        ask.disabled = false;
      });
  });
  glimpses
    .kept(at)
    .then((kept) => {
      if (!live()) return;
      if (told(kept)) shown(page, parts, kept);
      else ask.hidden = false;
    })
    .catch(() => {
      if (live()) ask.hidden = false;
    });
  return ask;
};

// A web link alone in its paragraph is drawn as a card of its site; inside a sentence it stays a link.
export const webCarded = (page: Document, say: Say, live: () => boolean, glimpses?: Glimpses) => {
  for (const link of page.querySelectorAll<HTMLAnchorElement>(".page > p > a[href]")) {
    const held = link.parentElement;
    const at = link.getAttribute("href") ?? "";
    const host = hosting(at);
    if (!held || !host || held.children.length !== 1 || link.children.length) continue;
    if ((held.textContent ?? "").trim() !== (link.textContent ?? "").trim()) continue;
    const box = page.createElement("a");
    box.className = "doc-card is-web is-bare";
    box.href = at;
    const tile = lettered(page, host);
    const words = link.textContent?.trim() ?? "";
    const name = piece(page, "card-name", !words || words === at ? readable(at, host) : words);
    const blurb = piece(page, "card-blurb");
    const meta = piece(page, "card-meta");
    meta.append(piece(page, "kind", host));
    if (glimpses) meta.append(asking(page, at, { box, tile, name, blurb }, say, glimpses, live));
    meta.append(piece(page, "card-go", "›"));
    box.append(tile, name, blurb, meta);
    link.replaceWith(box);
  }
};
