import { convertFileSrc } from "@tauri-apps/api/core";
import { glyph, piece } from "tisty-editor/read";
import { keptHere, type Lent, weighs } from "../core";
import { askAgain, serve, showing } from "../inUse";
import { t, type Word } from "../locales";
import { ending, family, KINDS, named, previewOf, weighed } from "../previews";
import { plainly } from "../refusal";
import { more } from "./carding";
import { type Lending, lendKept, locked } from "./widgeting";

const GLYPH: Record<string, string> = {
  pdf: "page",
  word: "page",
  text: "page",
  sheet: "sheet",
  data: "table",
  slides: "book",
  archive: "file-archive",
  page: "code",
  video: "file-video",
  audio: "file-audio",
};

export const lents = new WeakMap<Element, Lent>();

const alone = (img: Element): boolean => {
  const held = img.parentElement;
  return held?.tagName === "P" && held.children.length === 1 && !(held.textContent ?? "").trim();
};

export const pressed = (page: Document, className: string, said: string, run: () => void) => {
  const one = page.createElement("button");
  one.type = "button";
  one.className = className;
  one.textContent = said;
  one.addEventListener("click", run);
  return one;
};

// A sound or a film still in the cloud comes down only when asked for, and says so while it does.
const fed = (
  box: HTMLElement,
  player: HTMLMediaElement,
  at: string,
  live: () => boolean,
  held: (() => void)[],
) => {
  const page = box.ownerDocument;
  const state = piece(page, "media-state");
  box.append(state);
  const failed = (error_: unknown, play: boolean) => {
    box.dataset.state = "failed";
    const again = pressed(page, "media-again", t("tryAgain"), () => {
      askAgain(at);
      void bring(play);
    });
    state.replaceChildren(page.createTextNode(plainly(error_)), again);
  };
  const bring = async (play: boolean) => {
    box.dataset.state = "bringing";
    state.replaceChildren(t("mediaBringing"));
    let real: string;
    try {
      real = await serve(at);
    } catch (error_) {
      if (live()) failed(error_, play);
      return;
    }
    if (!live()) return;
    delete box.dataset.state;
    state.replaceChildren();
    player.src = convertFileSrc(real);
    if (!play) return;
    try {
      await player.play();
    } catch {
      return;
    }
  };
  const told = async () => {
    const here = await keptHere(at).catch(() => true);
    if (!live()) return;
    if (here) {
      held.push(showing(at));
      await bring(false);
      return;
    }
    box.dataset.state = "away";
    const ask = pressed(page, "media-bring", `▶ ${t("mediaBring")}`, () => void bring(true));
    state.replaceChildren(ask);
    const bytes = await weighs(at).catch(() => null);
    if (bytes !== null && live()) ask.textContent = `▶ ${t("mediaBring")} · ${weighed(bytes)}`;
  };
  void told();
};

const played = (
  page: Document,
  img: Element,
  as: "video" | "audio",
  at: string,
  live: () => boolean,
  held: (() => void)[],
  asking: boolean,
) => {
  const name = img.getAttribute("alt") || named(at);
  const box = page.createElement("span");
  box.className = `media is-${as}`;
  const player = page.createElement(as);
  player.controls = true;
  player.preload = as === "video" ? "metadata" : "none";
  player.setAttribute("aria-label", name);
  const said = piece(page, "media-name", name);
  if (as === "video") box.append(player, said);
  else box.append(said, player);
  if (asking) box.append(more(page, at));
  img.replaceWith(box);
  fed(box, player, at, live, held);
};

const labelled = (kind: string): string => {
  const word = KINDS[kind];
  return word ? t(word as Word) : kind.toUpperCase();
};

const filed = (page: Document, img: Element, at: string, live: () => boolean, asking: boolean) => {
  const name = img.getAttribute("alt") || named(at);
  const kind = ending(at);
  const kin = family(kind);
  if (!alone(img)) {
    const chip = page.createElement("a");
    chip.className = "card";
    chip.setAttribute("href", at);
    chip.dataset.family = kin;
    chip.textContent = name;
    img.replaceWith(chip);
    return;
  }
  const card = page.createElement("a");
  card.className = "doc-card is-file";
  card.setAttribute("href", at);
  card.dataset.family = kin;
  const tile = piece(page, "card-tile");
  tile.append(glyph(page, GLYPH[kin] ?? "page"));
  const meta = piece(page, "card-meta");
  const kindIs = piece(page, "kind", labelled(kind));
  meta.append(kindIs);
  if (asking) meta.append(more(page, at));
  meta.append(piece(page, "card-go", "›"));
  card.append(tile, piece(page, "card-name", name), piece(page, "card-blurb"), meta);
  img.replaceWith(card);
  weighs(at)
    .then((bytes) => {
      if (live()) kindIs.textContent = `${labelled(kind)} · ${weighed(bytes)}`;
    })
    .catch(() => undefined);
};

const paged = (
  page: Document,
  img: Element,
  at: string,
  apart: boolean,
  asking: boolean,
): Lending | null => {
  const name = img.getAttribute("alt") || named(at);
  const shown = page.createElement("figure");
  shown.className = "widget";
  lents.set(shown, { reference: at });
  const bar = page.createElement("div");
  bar.className = "widget-bar";
  bar.append(piece(page, "widget-name", name));
  if (apart) {
    const ask = page.createElement("button");
    ask.type = "button";
    ask.dataset.asked = "apart";
    ask.textContent = t("viewApart");
    bar.append(ask);
  }
  if (asking) bar.append(more(page, at));
  const held = page.createElement("div");
  shown.append(bar, held);
  img.replaceWith(shown);
  return lendKept(held, at);
};

export type Attaching = {
  readonly stranger: boolean;
  readonly apart: boolean;
  readonly more: boolean;
};

export const attachedAs = (
  page: Document,
  img: Element,
  at: string,
  live: () => boolean,
  held: (() => void)[],
  { stranger, apart, more: asking }: Attaching,
): Lending | null => {
  const seen = previewOf(at);
  if (seen?.as === "video" || seen?.as === "audio") {
    played(page, img, seen.as, at, live, held, asking);
    return null;
  }
  const drawnLive = ending(at) === "html" || ending(at) === "htm";
  if (drawnLive && (img as HTMLElement).dataset.as !== "file" && alone(img)) {
    if (stranger) {
      img.replaceWith(locked());
      return null;
    }
    return paged(page, img, at, apart, asking);
  }
  filed(page, img, at, live, asking);
  return null;
};
