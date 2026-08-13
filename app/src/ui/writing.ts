import StarterKit from "@tiptap/starter-kit";
import { Image } from "@tiptap/extension-image";
import { Table, TableRow, TableCell, TableHeader } from "@tiptap/extension-table";
import { TaskList } from "@tiptap/extension-task-list";
import { TaskItem } from "@tiptap/extension-task-item";
import { Text } from "@tiptap/extension-text";
import { Paragraph } from "@tiptap/extension-paragraph";
import { Heading } from "@tiptap/extension-heading";
import { TextAlign } from "@tiptap/extension-text-align";
import { Markdown } from "tiptap-markdown";
import type { Editor as Writing } from "@tiptap/core";

/// Its own serialiser never closes the block, so whatever follows an image ends
/// up glued to it — and a table glued to an image stops being a table on the
/// next read.
const Pictured = Image.extend({
  addStorage() {
    return {
      markdown: {
        serialize(
          state: { write: (text: string) => void; closeBlock: (node: unknown) => void },
          node: { attrs: Record<string, string> },
        ) {
          state.write(`![${node.attrs.alt ?? ""}](${node.attrs.src ?? ""})`);
          state.closeBlock(node);
        },
        parse: {},
      },
    };
  },
});

/// A pipe typed into a cell ends the column when the file is read back, and the
/// shifted table is what gets saved next. The library's own table serialiser
/// does not escape it, but it does raise `inTable`, so the escaping belongs
/// here, where the text is actually written.
const Barred = Text.extend({
  addStorage() {
    return {
      markdown: {
        serialize(
          state: { text: (value: string) => void; inTable?: boolean },
          node: { text?: string },
        ) {
          const text = node.text ?? "";
          state.text(state.inTable ? text.replace(/\|/g, "\\|") : text);
        },
        parse: {},
      },
    };
  },
});

/// Markdown has no syntax for alignment, so an aligned block is written as the
/// html it admits. An unaligned one must still come out as plain Markdown, or
/// every paragraph in every document would turn into a tag.
const leaning = (
  state: {
    write: (text: string) => void;
    renderInline: (node: unknown) => void;
    closeBlock: (node: unknown) => void;
  },
  node: { attrs: Record<string, string> },
  plain: () => void,
) => {
  const how = node.attrs.textAlign;
  if (!how || how === "left") return plain();
  state.write(`<p style="text-align: ${how}">`);
  state.renderInline(node);
  state.write("</p>");
  state.closeBlock(node);
};

const Leaning = Paragraph.extend({
  addStorage() {
    return {
      markdown: {
        serialize(state: never, node: never) {
          leaning(state, node, () => {
            const write = state as unknown as {
              renderInline: (n: unknown) => void;
              closeBlock: (n: unknown) => void;
            };
            write.renderInline(node);
            write.closeBlock(node);
          });
        },
        parse: {},
      },
    };
  },
});

const Titled = Heading.extend({
  addStorage() {
    return {
      markdown: {
        serialize(state: never, node: never) {
          leaning(state, node, () => {
            const write = state as unknown as {
              write: (t: string) => void;
              renderInline: (n: unknown) => void;
              closeBlock: (n: unknown) => void;
            };
            const deep = (node as unknown as { attrs: { level: number } }).attrs.level;
            write.write(`${"#".repeat(deep)} `);
            write.renderInline(node);
            write.closeBlock(node);
          });
        },
        parse: {},
      },
    };
  },
});

/// The one place the document's shape is decided. Tests build editors from this
/// same list, or they would be proving a copy of the configuration rather than
/// the editor a person types into.
export const written = () => [
  StarterKit.configure({ link: { openOnClick: false, autolink: true } }),
  Pictured,
  Table.configure({ resizable: false }),
  TableRow,
  TableHeader,
  TableCell,
  TaskList,
  TaskItem.configure({ nested: true }),
  Barred,
  Leaning,
  Titled,
  TextAlign.configure({ types: ["heading", "paragraph"] }),
  Markdown.configure({ html: true, linkify: true, breaks: true, transformPastedText: false }),
];

/// Null while the editor is being torn down: its storage is already gone.
export const asMarkdown = (editor: Writing): string | null => {
  if (editor.isDestroyed) return null;
  const kept = (editor.storage as unknown as { markdown?: { getMarkdown?: () => string } }).markdown;
  return typeof kept?.getMarkdown === "function" ? kept.getMarkdown() : null;
};

/// tiptap-markdown writes `[nodeName]` and drops the content when it cannot
/// serialise a node, so saving that would put the loss on disk for good. The
/// fallback always leaves it alone on its line, which is what keeps a person
/// writing the words `[table]` from being refused a save.
const RUINED = /^\s*\[(table|image|tableRow|tableCell|tableHeader)\]\s*$/m;

export const ruined = (markdown: string): boolean => RUINED.test(markdown);
