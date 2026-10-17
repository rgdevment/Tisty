import { type Glimpse, glimpseFetch, glimpseKept } from "../core";
import { t } from "../locales";
import { hosting } from "../previews";
import { piece } from "./carding";
import { HUES } from "./paging";

// One question to the core per link and window, however many times the page is drawn again.
const asked = new Map<string, Promise<Glimpse | null>>();

const keptFor = (at: string) => {
  const was = asked.get(at);
  if (was) return was;
  const said = glimpseKept(at).catch(() => null);
  asked.set(at, said);
  return said;
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

const shown = (
  page: Document,
  parts: { box: HTMLElement; tile: HTMLElement; name: HTMLElement; blurb: HTMLElement },
  said: Glimpse,
) => {
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

export const webCarded = (page: Document, live: () => boolean) => {
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
          if (told(said)) {
            asked.set(at, Promise.resolve(said));
            shown(page, { box, tile, name, blurb }, said);
          }
        })
        .catch(() => {
          ask.disabled = false;
        });
    });
    meta.append(ask, piece(page, "card-go", "›"));
    box.append(tile, name, blurb, meta);
    link.replaceWith(box);
    keptFor(at)
      .then((said) => {
        if (!live()) return;
        if (told(said)) shown(page, { box, tile, name, blurb }, said);
        else ask.hidden = false;
      })
      .catch(() => {
        if (live()) ask.hidden = false;
      });
  }
};
