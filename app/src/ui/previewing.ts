import { Extension } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";
import type { EditorState, Transaction } from "@tiptap/pm/state";
import type { Node as Written } from "@tiptap/pm/model";
import { KINDS, ending, named, previewOf, weighed, type Preview } from "../previews";
import { t } from "../locales";

export interface Reach {
  url: (reference: string) => string | null;
  weight: (reference: string) => number | null;
  title: (id: string) => string | null | undefined;
  here?: string;
  blurb?: (id: string) => string | null;
  gone?: (reference: string) => boolean;
  onDoc?: (id: string) => void;
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

const linked = (node: Written): string | null => {
  if (!node.isText) return null;
  const link = node.marks.find((one) => one.type.name === "link");
  return link ? String(link.attrs.href ?? "") : null;
};

const found = (doc: Written): Spot[] => {
  const all: Spot[] = [];
  doc.descendants((block, at) => {
    if (!block.isTextblock) return true;
    const after = at + block.nodeSize;
    block.forEach((node, _offset, index) => {
      const href = linked(node);
      if (href === null) return;
      const next = block.maybeChild(index + 1);
      if (next && linked(next) === href) return;
      const seen = previewOf(href);
      if (!seen) return;
      all.push({ at: after, seen, href, label: node.text ?? "" });
    });
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

const played = (seen: Preview & { as: "video" }, reach: Reach): HTMLElement => {
  const box = frame("preview preview-video");

  const asks = () => {
    const seat = document.createElement("span");
    seat.className = "preview-seat";
    const glance = document.createElement("video");
    glance.preload = "metadata";
    glance.muted = true;
    glance.tabIndex = -1;
    const url = reach.url(seen.at);
    if (url) glance.src = `${url}#t=0.1`;

    const said = document.createElement("button");
    said.type = "button";
    said.className = "preview-play";
    said.setAttribute("aria-label", t("playIt"));
    said.title = t("playIt");
    said.addEventListener("click", shows);

    seat.append(glance, said);
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
  return badge;
};

const built = (
  seen: Preview,
  reach: Reach,
  label: string,
  back: () => void,
  drop: () => void,
): HTMLElement => {
  const lost = seen.as !== "doc" && Boolean(reach.gone?.(seen.at));

  if (seen.as === "video" && !lost) return played(seen, reach);

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
    box.prepend(carded("doc"));
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
    under.textContent = [KINDS[kind] ?? kind.toUpperCase(), bytes === null ? null : weighed(bytes)]
      .filter(Boolean)
      .join(" · ");
  }

  said.append(name, under);
  box.append(said);
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
  const $at = view.state.doc.resolve(Math.max(1, at - 1));
  if (!$at.depth) return;
  view.dispatch(view.state.tr.delete($at.before($at.depth), $at.after($at.depth)));
};

export const alone = (block: Written): boolean => {
  let held = false;
  let more = false;
  block.forEach((child) => {
    const link = child.isText
      ? child.marks.find((one) => one.type.name === "link")
      : undefined;
    if (link && previewOf(String(link.attrs.href ?? ""))) {
      if (held) more = true;
      held = true;
      return;
    }
    if (!child.isText || child.text?.trim()) more = true;
  });
  return held && !more;
};

export const eats = (key: string, at: number, room: number, whole: boolean): boolean => {
  if (whole) return true;
  if (key === "Backspace") return at > 0;
  if (key === "Delete") return at < room;
  return false;
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
            handleKeyDown(view, event) {
              if (event.key !== "Backspace" && event.key !== "Delete") return false;
              const { $from, $to, empty } = view.state.selection;
              if (!$from.depth || !$from.sameParent($to) || !alone($from.parent)) return false;
              if (!eats(event.key, $from.parentOffset, $from.parent.content.size, !empty)) {
                return false;
              }
              view.dispatch(
                view.state.tr.delete($from.before($from.depth), $from.after($from.depth)),
              );
              return true;
            },
            decorations(state) {
              const now = reach();
              return DecorationSet.create(
                state.doc,
                scan(state.doc).map((one) =>
                  Decoration.widget(
                    one.at,
                    (view, getPos) =>
                      built(
                        one.seen,
                        now,
                        one.label,
                        () => view.focus(),
                        () => shed(view, getPos()),
                      ),
                    {
                      key: `${one.seen.as}:${one.href}:${settled(one.seen, now)}`,
                      side: 1,
                      ignoreSelection: true,
                      stopEvent: () => true,
                    },
                  ),
                ),
              );
            },
          },
        }),
      ];
    },
  });
};
