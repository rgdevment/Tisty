import { Extension } from "@tiptap/core";
import type { Node as Written } from "@tiptap/pm/model";
import type { EditorState, Transaction } from "@tiptap/pm/state";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";
import type { Glimpse } from "../core";
import { markup } from "../glyphs";
import { t, type Word } from "../locales";
import {
  ending,
  family,
  KINDS,
  named,
  type Preview,
  pictured,
  previewOf,
  weighed,
} from "../previews";

export interface Reach {
  url: (reference: string) => string | null;
  weight: (reference: string) => number | null;
  title: (id: string) => string | null | undefined;
  here?: string;
  blurb?: (id: string) => string | null;
  glimpse?: (at: string) => Glimpse | null;
  onGlimpse?: (at: string) => void;
  page?: (id: string) => number | null;
  gone?: (reference: string) => boolean;
  onDoc?: (id: string) => void;
  onWorld?: (at: string) => void;
  onMenu?: (
    at: { x: number; y: number },
    untie: (() => void) | undefined,
    drop: () => void,
    kept?: { at: string; name: string },
    leaf?: string,
  ) => void;
  onOpen?: (reference: string) => void;
  onAgain?: (reference: string) => void;
}

export const plugged = new PluginKey("previewing");

interface Spot {
  at: number;
  size: number;
  seen: Preview;
  href: string;
  label: string;
}

export const asCard = (node: Written): { href: string; seen: Preview; label: string } | null => {
  if (node.type.name !== "image") return null;
  const href = String(node.attrs.src ?? "");
  if (pictured(href)) return null;
  const seen = previewOf(href);
  if (!seen || seen.as === "web") return null;
  return { href, seen, label: String(node.attrs.alt ?? "") };
};

/// A link alone in its paragraph is a card; the same link inside a sentence stays a link. The
/// shape in the file is the whole answer, so nothing has to be remembered beside it.
export const asBookmark = (
  node: Written,
): { href: string; seen: Preview; label: string } | null => {
  if (node.type.name !== "paragraph" || node.childCount !== 1) return null;
  const only = node.firstChild;
  if (!only?.isText) return null;
  const tied = only.marks.find((one) => one.type.name === "link");
  const href = String(tied?.attrs.href ?? "");
  if (!href) return null;
  const seen = previewOf(href);
  return seen?.as === "web" ? { href, seen, label: only.text ?? "" } : null;
};

const found = (doc: Written): Spot[] => {
  const all: Spot[] = [];
  doc.descendants((node, at) => {
    const card = asCard(node);
    if (card) {
      all.push({ at, size: 1, seen: card.seen, href: card.href, label: card.label });
      return false;
    }
    const mark = asBookmark(node);
    if (!mark) return true;
    all.push({ at, size: node.nodeSize, seen: mark.seen, href: mark.href, label: mark.label });
    return false;
  });
  return all;
};

const frame = (kind: string): HTMLElement => {
  const box = document.createElement("span");
  box.className = kind;
  box.contentEditable = "false";
  return box;
};

const clocked = (secs: number): string => {
  const whole = Math.round(secs);
  const mins = Math.floor(whole / 60);
  return `${mins}:${String(whole % 60).padStart(2, "0")}`;
};

const played = (seen: Preview & { as: "video" }, reach: Reach, label: string): HTMLElement => {
  const box = frame("preview preview-video");

  const asks = () => {
    const seat = document.createElement("span");
    seat.className = "preview-seat";
    const glance = document.createElement("video");
    glance.preload = "metadata";
    glance.muted = true;
    glance.playsInline = true;
    glance.tabIndex = -1;
    const url = reach.url(seen.at);
    if (url) glance.src = `${url}#t=0.1`;
    glance.addEventListener("loadeddata", () => seat.classList.add("preview-lit"));
    glance.addEventListener("loadedmetadata", () => {
      if (glance.currentTime < 0.05 && glance.duration > 0.2) glance.currentTime = 0.1;
      if (Number.isFinite(glance.duration)) under.textContent = clocked(glance.duration);
    });

    const strip = document.createElement("span");
    strip.className = "preview-strip";
    const called = document.createElement("span");
    called.className = "preview-called";
    called.textContent = label || named(seen.at);
    const under = document.createElement("span");
    under.className = "preview-long";
    strip.append(called, under);

    const said = document.createElement("button");
    said.type = "button";
    said.className = "preview-play";
    said.setAttribute("aria-label", t("playIt"));
    said.title = t("playIt");
    said.addEventListener("click", shows);

    seat.append(glance, strip, said);
    box.replaceChildren(seat);
  };

  const shows = () => {
    const player = document.createElement("video");
    player.controls = true;
    player.autoplay = true;
    player.preload = "metadata";
    const url = reach.url(seen.at);
    if (url) player.src = url;

    const away = document.createElement("button");
    away.type = "button";
    away.className = "preview-fold";
    away.title = t("foldIt");
    away.setAttribute("aria-label", t("foldIt"));
    away.textContent = "×";
    away.addEventListener("click", () => {
      try {
        player.pause();
      } catch {
        void 0;
      }
      asks();
    });

    box.replaceChildren(player, away);
  };

  asks();
  return box;
};

const carded = (kind: string): HTMLElement => {
  const badge = document.createElement("span");
  badge.className = "card-badge";
  badge.dataset.kind = kind.slice(0, 4).toUpperCase();
  badge.dataset.family = kind === "doc" ? "doc" : family(kind);
  return badge;
};

const numbered = (n: number): HTMLElement => {
  const badge = document.createElement("span");
  badge.className = "card-num";
  badge.textContent = String(n);
  return badge;
};

const into = (): HTMLElement => {
  const held = document.createElement("span");
  held.className = "card-into";
  held.setAttribute("aria-hidden", "true");
  held.textContent = "›";
  return held;
};

const leaves = (box: HTMLElement) => {
  for (const which of ["card-leaf-one", "card-leaf-two"]) {
    const leaf = document.createElement("span");
    leaf.className = `card-leaf ${which}`;
    box.append(leaf);
  }
};

const papered = (): HTMLElement => {
  const held = document.createElement("span");
  held.className = "card-paper";
  const drawn = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  drawn.setAttribute("viewBox", "0 0 24 24");
  drawn.setAttribute("aria-hidden", "true");
  drawn.setAttribute("class", "glyph");
  drawn.innerHTML = markup("page") ?? "";
  held.append(drawn);
  return held;
};

const built = (
  seen: Preview,
  reach: Reach,
  label: string,
  back: () => void,
  drop: () => void,
  untie: () => void,
): HTMLElement => {
  const lost = seen.as !== "doc" && seen.as !== "web" && Boolean(reach.gone?.(seen.at));

  if (seen.as === "video" && !lost) return played(seen, reach, label);

  const box = frame("preview");
  box.addEventListener("keydown", (e) => {
    // The widget hides the key from the editor, and a document that cannot be written keeps it.
    if ((e.key === "Backspace" || e.key === "Delete") && reach.onMenu) {
      e.preventDefault();
      return drop();
    }
    if (e.key !== "Escape") return;
    e.preventDefault();
    try {
      back();
    } catch {
      void 0;
    }
  });

  if (seen.as === "audio" && !lost) {
    const player = document.createElement("audio");
    player.controls = true;
    player.preload = "metadata";
    const url = reach.url(seen.at);
    if (url) player.src = url;
    box.classList.add("preview-audio");
    box.append(player);
    return box;
  }

  box.classList.add("card");
  const said = document.createElement("span");
  said.className = "card-said";
  const name = document.createElement("span");
  name.className = "card-name";
  const under = document.createElement("span");
  under.className = "card-under";

  const itself = seen.as === "doc" && seen.id === reach.here;

  const leaf = seen.as === "doc" ? (reach.page?.(seen.id) ?? null) : null;
  const asPage = seen.as === "doc" && leaf !== null ? seen.id : undefined;

  if (seen.as === "doc") {
    const title = reach.title(seen.id);
    box.dataset.doc = seen.id;
    box.classList.add("card-doc");
    if (leaf === null) box.prepend(papered());
    else {
      box.classList.add("card-page");
      box.prepend(numbered(leaf));
    }
    if (title === undefined) {
      name.textContent = t("opening");
    } else if (title === null) {
      name.textContent = label || t("untitledDoc");
      under.textContent = t("goneDoc");
      box.classList.add("card-gone");
    } else {
      name.textContent = title;
      under.textContent = itself ? t("docItself") : (reach.blurb?.(seen.id) ?? "");
    }
  } else if (seen.as === "web") {
    const glimpse = reach.glimpse?.(seen.at) ?? null;
    box.classList.add("card-web");
    box.prepend(badged(seen.host));
    name.textContent = glimpse?.title || label.trim() || seen.host;
    under.textContent = glimpse?.said ?? "";
    under.classList.toggle("card-said-two", Boolean(glimpse?.said));
    const where = document.createElement("span");
    where.className = "card-where";
    where.textContent = seen.host;
    said.append(where);
    if (glimpse?.shot) box.append(shotted(glimpse.shot));
    else if (!glimpse && reach.onGlimpse) box.append(asking(seen.at, reach.onGlimpse));
  } else if (lost) {
    box.classList.add("card-gone");
    box.prepend(carded(ending(seen.at) || "?"));
    name.textContent = label || named(seen.at);
    under.textContent = t("lookAgain");
  } else {
    const kind = ending(seen.at);
    const bytes = reach.weight(seen.at);
    box.prepend(carded(kind || "?"));
    name.textContent = label || named(seen.at);
    const called = KINDS[kind];
    under.textContent = [
      called ? t(called as Word) : kind.toUpperCase(),
      bytes === null ? null : weighed(bytes),
    ]
      .filter(Boolean)
      .join(" · ");
  }

  said.append(name, under);
  box.append(said);
  if (leaf !== null) box.append(into());

  if (!reach.onMenu) {
    if (leaf !== null) leaves(box);
    return sayable(box, seen, reach, lost, itself);
  }

  const more = document.createElement("button");
  more.type = "button";
  more.className = "card-swap";
  more.textContent = "⋯";
  more.title = t("moreOnIt");
  more.setAttribute("aria-label", t("moreOnIt"));
  more.setAttribute("aria-haspopup", "menu");
  more.addEventListener("click", (e) => {
    e.stopPropagation();
    const box = more.getBoundingClientRect();
    const kept =
      seen.as === "doc" || seen.as === "web" || lost
        ? undefined
        : { at: seen.at, name: label || named(seen.at) };
    const back = seen.as === "web" ? undefined : untie;
    reach.onMenu?.({ x: box.left, y: box.bottom + 4 }, back, drop, kept, asPage);
  });
  box.append(more);
  if (leaf !== null) leaves(box);
  return sayable(box, seen, reach, lost, itself);
};

const sayable = (
  box: HTMLElement,
  seen: Preview,
  reach: Reach,
  lost: boolean,
  itself: boolean,
): HTMLElement => {
  if (itself) {
    box.classList.add("card-itself");
    return box;
  }
  box.setAttribute("role", "button");
  box.setAttribute("tabindex", "0");
  const go = () => {
    if (seen.as === "doc") return reach.onDoc?.(seen.id);
    if (seen.as === "web") return reach.onWorld?.(seen.at);
    if (lost) return reach.onAgain?.(seen.at);
    reach.onOpen?.(seen.at);
  };
  box.addEventListener("click", go);
  box.addEventListener("keydown", (e) => {
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    go();
  });
  return box;
};

const shed = (
  view: { state: EditorState; dispatch: (tr: Transaction) => void },
  at: number | undefined,
  size: number,
) => {
  if (at === undefined) return;
  view.dispatch(view.state.tr.delete(at, at + size));
};

const untied = (
  view: { state: EditorState; dispatch: (tr: Transaction) => void },
  at: number | undefined,
  href: string,
  label: string,
) => {
  if (at === undefined) return;
  const { schema } = view.state;
  const said = label.trim() || named(href);
  const words = schema.text(said, [schema.marks.link.create({ href })]);
  view.dispatch(view.state.tr.replaceWith(at, at + 1, schema.nodes.paragraph.create(null, words)));
};

const HOSTS: Record<string, string> = {
  "github.com":
    "M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z",
};

const ANY =
  "M8 0a8 8 0 1 0 0 16A8 8 0 0 0 8 0Zm5.5 5H11.3a12.6 12.6 0 0 0-1-2.5A6.5 6.5 0 0 1 13.5 5ZM8 1.6c.5.7 1 1.8 1.4 3.4H6.6C7 3.4 7.5 2.3 8 1.6ZM1.7 9.5A6.6 6.6 0 0 1 1.5 8c0-.5.1-1 .2-1.5h2.5a15 15 0 0 0 0 3H1.7Zm.8 1.5h2.2c.2 1 .6 1.8 1 2.5A6.5 6.5 0 0 1 2.5 11Zm2.2-6H2.5a6.5 6.5 0 0 1 3.2-2.5c-.4.7-.8 1.5-1 2.5ZM8 14.4c-.5-.7-1-1.8-1.4-3.4h2.8c-.4 1.6-.9 2.7-1.4 3.4Zm1.7-4.9H6.3a13.6 13.6 0 0 1 0-3h3.4a13.6 13.6 0 0 1 0 3Zm.6 4.1c.4-.7.8-1.5 1-2.6h2.2a6.5 6.5 0 0 1-3.2 2.6Zm1.2-4.1a15 15 0 0 0 0-3h2.5c.1.5.2 1 .2 1.5s-.1 1-.2 1.5h-2.5Z";

const badged = (host: string): HTMLElement => {
  const box = document.createElement("span");
  box.className = "card-host";
  const glyph = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  glyph.setAttribute("viewBox", "0 0 16 16");
  glyph.setAttribute("aria-hidden", "true");
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  path.setAttribute("fill", "currentColor");
  path.setAttribute("d", HOSTS[host] ?? ANY);
  glyph.append(path);
  box.append(glyph);
  return box;
};

const shotted = (src: string): HTMLElement => {
  const box = document.createElement("span");
  box.className = "card-shot";
  const shot = document.createElement("img");
  shot.src = src;
  shot.alt = "";
  shot.loading = "lazy";
  box.append(shot);
  return box;
};

const asking = (at: string, ask: (at: string) => void): HTMLElement => {
  const said = document.createElement("button");
  said.type = "button";
  said.className = "card-pull";
  said.textContent = t("linkGlimpse");
  said.addEventListener("click", (e) => {
    e.stopPropagation();
    ask(at);
  });
  return said;
};

const settled = (seen: Preview, reach: Reach): string => {
  if (seen.as === "web") {
    const one = reach.glimpse?.(seen.at);
    return one
      ? `${seen.host}:${one.title ?? ""}:${one.said ?? ""}:${one.shot ? "shot" : ""}`
      : seen.host;
  }
  if (seen.as === "doc") {
    if (seen.id === reach.here) return "itself";
    const title = reach.title(seen.id);
    const leaf = reach.page?.(seen.id) ?? 0;
    const blurb = reach.blurb?.(seen.id) ?? "";
    return title === undefined ? "" : `${leaf}:${title ?? "gone"}:${blurb}`;
  }
  if (reach.gone?.(seen.at)) return "gone";
  if (seen.as === "file") return String(reach.weight(seen.at) ?? "");
  return reach.url(seen.at) ?? "";
};

export const previewing = (reach: () => Reach) => {
  let read: Written | null = null;
  let spots: Spot[] = [];
  const scan = (doc: Written): Spot[] => {
    if (doc === read) return spots;
    read = doc;
    spots = found(doc);
    return spots;
  };

  return Extension.create({
    name: "previewing",
    addProseMirrorPlugins() {
      return [
        new Plugin({
          key: plugged,
          props: {
            decorations(state) {
              const now = reach();
              return DecorationSet.create(
                state.doc,
                scan(state.doc).flatMap((one) => [
                  Decoration.node(one.at, one.at + one.size, { class: "card-source" }),
                  Decoration.widget(
                    one.at,
                    (view, getPos) =>
                      built(
                        one.seen,
                        now,
                        one.label,
                        () => view.focus(),
                        () => shed(view, getPos(), one.size),
                        () => untied(view, getPos(), one.href, one.label),
                      ),
                    {
                      key: `${one.seen.as}:${one.href}:${settled(one.seen, now)}`,
                      side: 1,
                      ignoreSelection: true,
                      stopEvent: () => true,
                    },
                  ),
                ]),
              );
            },
          },
        }),
      ];
    },
  });
};
