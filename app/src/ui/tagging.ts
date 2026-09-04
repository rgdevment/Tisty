import { Extension } from "@tiptap/core";
import type { Node as Written } from "@tiptap/pm/model";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";

export const tagged = new PluginKey("tagging");

/// The same reading the core does when it saves: a hash pinned to a letter or a digit, with a
/// separator in front of it. `# Heading` carries a space, and a colour or a web address carries
/// something that is not a separator.
const AT = /(^|[^\p{L}\p{N}#/\-_.:])#([\p{L}\p{N}][\p{L}\p{N}\-_]*)/gu;

const drawn = (state: { doc: Written; schema: { nodes: Record<string, unknown> } }) => {
  const codes = state.schema.nodes.codeBlock;
  return spots(state.doc).filter(({ from }) => {
    const at = state.doc.resolve(from);
    return !codes || at.parent.type !== codes;
  });
};

export const spots = (doc: Written): { from: number; to: number }[] => {
  const found: { from: number; to: number }[] = [];
  doc.descendants((node, at) => {
    if (!node.isText || !node.text) return;
    if (node.marks.some((mark) => mark.type.name === "code")) return;
    // A backtick pair marks code here too, the way the core reads it off the saved markdown.
    const said = node.text.replace(/`[^`]*`/g, (one) => " ".repeat(one.length));
    AT.lastIndex = 0;
    let hit = AT.exec(said);
    while (hit) {
      const from = at + hit.index + hit[1].length;
      found.push({ from, to: from + hit[2].length + 1 });
      hit = AT.exec(said);
    }
  });
  return found;
};

/// The same shape the core gives a tag — accents off, lowercase, underscores and spaces as dashes,
/// nothing else kept, and never two dashes in a row. A word that lands anywhere else opens a
/// screen with nothing on it.
export const named = (said: string): string =>
  said
    .normalize("NFD")
    .replace(/\p{M}/gu, "")
    .toLowerCase()
    .replace(/[\s_]/g, "-")
    .replace(/[^\p{L}\p{N}-]/gu, "")
    .replace(/-{2,}/g, "-")
    .replace(/^-+|-+$/g, "");

export const tagging = (onTag?: (tag: string) => void) =>
  Extension.create({
    name: "tagging",
    addProseMirrorPlugins() {
      return [
        new Plugin({
          key: tagged,
          props: {
            decorations(state) {
              return DecorationSet.create(
                state.doc,
                drawn(state).map(({ from, to }) =>
                  Decoration.inline(from, to, { class: "tag-said" }),
                ),
              );
            },
            handleClick(view, at) {
              if (!onTag) return false;
              const hit = drawn(view.state).find((one) => at > one.from && at < one.to);
              if (!hit) return false;
              onTag(named(view.state.doc.textBetween(hit.from + 1, hit.to)));
              return true;
            },
          },
        }),
      ];
    },
  });
