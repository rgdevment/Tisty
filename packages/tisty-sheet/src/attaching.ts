import { ending, fileCard, KINDS, playedAs } from "./files";
import { piece } from "./pieces";
import { weighed } from "./weighed";

export type Attaching = {
  readonly serve: (at: string) => Promise<string>;
  readonly url: (real: string) => string;
  readonly keptHere: (at: string) => Promise<boolean>;
  readonly weighs: (at: string) => Promise<number>;
  readonly again: (at: string) => void;
  readonly showing: (at: string) => () => void;
  readonly plainly: (problem: unknown) => string;
  readonly say: (word: string) => string;
  readonly named: (at: string) => string;
  readonly placed?: (img: Element, shown: HTMLElement) => void;
  readonly more?: (page: Document, at: string) => Node;
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
  host: Attaching,
) => {
  const page = box.ownerDocument;
  const state = piece(page, "media-state");
  box.append(state);
  const failed = (error_: unknown, play: boolean) => {
    box.dataset.state = "failed";
    const again = pressed(page, "media-again", host.say("tryAgain"), () => {
      host.again(at);
      void bring(play);
    });
    state.replaceChildren(page.createTextNode(host.plainly(error_)), again);
  };
  const bring = async (play: boolean) => {
    box.dataset.state = "bringing";
    state.replaceChildren(host.say("mediaBringing"));
    let real: string;
    try {
      real = await host.serve(at);
    } catch (error_) {
      if (live()) failed(error_, play);
      return;
    }
    if (!live()) return;
    delete box.dataset.state;
    state.replaceChildren();
    player.src = host.url(real);
    if (!play) return;
    try {
      await player.play();
    } catch {
      return;
    }
  };
  const told = async () => {
    const here = await host.keptHere(at).catch(() => true);
    if (!live()) return;
    if (here) {
      held.push(host.showing(at));
      await bring(false);
      return;
    }
    box.dataset.state = "away";
    const bringing = `▶ ${host.say("mediaBring")}`;
    const ask = pressed(page, "media-bring", bringing, () => void bring(true));
    state.replaceChildren(ask);
    const bytes = await host.weighs(at).catch(() => null);
    if (bytes !== null && live()) ask.textContent = `${bringing} · ${weighed(bytes)}`;
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
  host: Attaching,
) => {
  const name = img.getAttribute("alt") || host.named(at);
  const box = page.createElement("span");
  box.className = `media is-${as}`;
  box.dataset.file = at;
  box.dataset.name = img.getAttribute("alt") ?? "";
  const player = page.createElement(as);
  player.controls = true;
  player.preload = as === "video" ? "metadata" : "none";
  player.setAttribute("aria-label", name);
  const said = piece(page, "media-name", name);
  if (as === "video") box.append(player, said);
  else box.append(said, player);
  if (host.more) box.append(host.more(page, at));
  host.placed?.(img, box);
  img.replaceWith(box);
  fed(box, player, at, live, held, host);
};

const labelled = (kind: string, host: Attaching): string => {
  const word = KINDS[kind];
  return word ? host.say(word) : kind.toUpperCase();
};

const filed = (page: Document, img: Element, at: string, live: () => boolean, host: Attaching) => {
  const kind = ending(at);
  const named = img.getAttribute("alt") || host.named(at);
  const { card, meta, kindIs } = fileCard(page, img, kind, named, labelled(kind, host));
  card.setAttribute("href", at);
  host.placed?.(img, card);
  if (!meta || !kindIs) return;
  if (host.more) meta.append(host.more(page, at));
  meta.append(piece(page, "card-go", "›"));
  host
    .weighs(at)
    .then((bytes) => {
      if (live()) kindIs.textContent = `${labelled(kind, host)} · ${weighed(bytes)}`;
    })
    .catch(() => undefined);
};

export const attachedAs = (
  page: Document,
  img: Element,
  at: string,
  live: () => boolean,
  held: (() => void)[],
  host: Attaching,
): void => {
  const as = playedAs(ending(at));
  if (as) {
    played(page, img, as, at, live, held, host);
    return;
  }
  filed(page, img, at, live, host);
};
