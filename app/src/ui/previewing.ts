import { Extension } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";
import type { EditorState } from "@tiptap/pm/state";
import type { Node as Written } from "@tiptap/pm/model";
import { ending, named, previewOf, weighed, type Preview } from "../previews";
import { t } from "../locales";

export interface Reach {
  url: (reference: string) => string | null;
  weight: (reference: string) => number | null;
  title: (id: string) => string | null;
  gone?: (reference: string) => boolean;
  onDoc?: (id: string) => void;
  onOpen?: (reference: string) => void;
  onAgain?: (reference: string) => void;
}

const key = new PluginKey("previewing");

const ends = (doc: Written, pos: number, node: Written, href: string): boolean => {
  const after = doc.resolve(pos + node.nodeSize).nodeAfter;
  if (!after?.isText) return true;
  return !after.marks.some((one) => one.type.name === "link" && one.attrs.href === href);
};

const found = (state: EditorState): { at: number; seen: Preview; href: string }[] => {
  const all: { at: number; seen: Preview; href: string }[] = [];
  state.doc.descendants((node, pos) => {
    if (!node.isText) return;
    const link = node.marks.find((one) => one.type.name === "link");
    if (!link) return;
    const href = String(link.attrs.href ?? "");
    if (!ends(state.doc, pos, node, href)) return;
    const seen = previewOf(href);
    if (!seen) return;
    const at = state.doc.resolve(pos);
    all.push({ at: at.depth ? at.after(at.depth) : pos + node.nodeSize, seen, href });
  });
  return all;
};

const frame = (): HTMLElement => {
  const box = document.createElement("span");
  box.className = "preview";
  box.contentEditable = "false";
  return box;
};

const built = (seen: Preview, reach: Reach): HTMLElement => {
  const box = frame();

  if (seen.as === "doc") {
    box.classList.add("preview-doc");
    const title = reach.title(seen.id);
    const name = document.createElement("span");
    name.className = "preview-name";
    name.textContent = title ?? t("opening");
    box.append(name);
    if (title === null) box.classList.add("preview-gone");
    box.setAttribute("role", "button");
    box.setAttribute("tabindex", "0");
    const go = () => reach.onDoc?.(seen.id);
    box.addEventListener("click", go);
    box.addEventListener("keydown", (e) => {
      if (e.key !== "Enter" && e.key !== " ") return;
      e.preventDefault();
      go();
    });
    return box;
  }

  const lost = Boolean(reach.gone?.(seen.at));

  if (!lost && (seen.as === "video" || seen.as === "audio")) {
    const player = document.createElement(seen.as);
    player.controls = true;
    player.preload = "metadata";
    const url = reach.url(seen.at);
    if (url) player.src = url;
    box.classList.add(`preview-${seen.as}`);
    box.append(player);
    return box;
  }

  box.classList.add("preview-file");
  const name = document.createElement("span");
  name.className = "preview-name";
  name.textContent = named(seen.at);
  const said = document.createElement("span");
  said.className = "preview-said";
  if (lost) {
    box.classList.add("preview-gone");
    said.textContent = t("lookAgain");
    box.title = t("goneFile");
    box.setAttribute("role", "button");
    box.setAttribute("tabindex", "0");
    const again = () => reach.onAgain?.(seen.at);
    box.addEventListener("click", again);
    box.addEventListener("keydown", (e) => {
      if (e.key !== "Enter" && e.key !== " ") return;
      e.preventDefault();
      again();
    });
  } else {
    const bytes = reach.weight(seen.at);
    said.textContent =
      bytes === null ? ending(seen.at).toUpperCase() : `${ending(seen.at).toUpperCase()} · ${weighed(bytes)}`;
    box.setAttribute("role", "button");
    box.setAttribute("tabindex", "0");
    const go = () => reach.onOpen?.(seen.at);
    box.addEventListener("click", go);
    box.addEventListener("keydown", (e) => {
      if (e.key !== "Enter" && e.key !== " ") return;
      e.preventDefault();
      go();
    });
  }
  box.append(name, said);
  return box;
};

const settled = (seen: Preview, reach: Reach): string => {
  if (seen.as === "doc") return reach.title(seen.id) ?? "";
  if (reach.gone?.(seen.at)) return "gone";
  if (seen.as === "file") return String(reach.weight(seen.at) ?? "");
  return reach.url(seen.at) ?? "";
};

export const previewing = (reach: () => Reach) =>
  Extension.create({
    name: "previewing",
    addProseMirrorPlugins() {
      return [
        new Plugin({
          key,
          props: {
            decorations(state) {
              const now = reach();
              return DecorationSet.create(
                state.doc,
                found(state).map((one) =>
                  Decoration.widget(one.at, () => built(one.seen, now), {
                    key: `${one.seen.as}:${one.href}:${settled(one.seen, now)}`,
                    side: 1,
                    ignoreSelection: true,
                  }),
                ),
              );
            },
          },
        }),
      ];
    },
  });
