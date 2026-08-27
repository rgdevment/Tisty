import { Extension } from "@tiptap/core";
import type { Node as Written } from "@tiptap/pm/model";
import type { EditorState, Transaction } from "@tiptap/pm/state";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";
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
  gone?: (reference: string) => boolean;
  onDoc?: (id: string) => void;
  onMenu?: (
    at: { x: number; y: number },
    untie: () => void,
    drop: () => void,
    kept?: { at: string; name: string },
  ) => void;
  onOpen?: (reference: string) => void;
  onAgain?: (reference: string) => void;
}

export const plugged = new PluginKey("previewing");

interface Spot {
  at: number;
  seen: Preview;
  href: string;
  label: string;
}

export const asCard = (node: Written): { href: string; seen: Preview; label: string } | null => {
  if (node.type.name !== "image") return null;
  const href = String(node.attrs.src ?? "");
  if (pictured(href)) return null;
  const seen = previewOf(href);
  return seen ? { href, seen, label: String(node.attrs.alt ?? "") } : null;
};

const found = (doc: Written): Spot[] => {
  const all: Spot[] = [];
  doc.descendants((node, at) => {
    const card = asCard(node);
    if (!card) return true;
    all.push({ at, seen: card.seen, href: card.href, label: card.label });
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
  const lost = seen.as !== "doc" && Boolean(reach.gone?.(seen.at));

  if (seen.as === "video" && !lost) return played(seen, reach, label);

  const box = frame("preview");
  box.addEventListener("keydown", (e) => {
    if (e.key === "Backspace" || e.key === "Delete") {
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

  if (seen.as === "doc") {
    const title = reach.title(seen.id);
    box.classList.add("card-doc");
    box.prepend(papered());
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
      seen.as === "doc" || lost ? undefined : { at: seen.at, name: label || named(seen.at) };
    reach.onMenu?.({ x: box.left, y: box.bottom + 4 }, untie, drop, kept);
  });
  box.append(more);
  if (itself) {
    box.classList.add("card-itself");
    return box;
  }
  box.setAttribute("role", "button");
  box.setAttribute("tabindex", "0");
  const go = () => {
    if (seen.as === "doc") return reach.onDoc?.(seen.id);
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
) => {
  if (at === undefined) return;
  view.dispatch(view.state.tr.delete(at, at + 1));
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

const settled = (seen: Preview, reach: Reach): string => {
  if (seen.as === "doc") {
    if (seen.id === reach.here) return "itself";
    const title = reach.title(seen.id);
    return title === undefined ? "" : (title ?? "gone");
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
                  Decoration.node(one.at, one.at + 1, { class: "card-source" }),
                  Decoration.widget(
                    one.at,
                    (view, getPos) =>
                      built(
                        one.seen,
                        now,
                        one.label,
                        () => view.focus(),
                        () => shed(view, getPos()),
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
