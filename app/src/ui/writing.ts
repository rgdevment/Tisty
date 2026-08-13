import StarterKit from "@tiptap/starter-kit";
import { Image } from "@tiptap/extension-image";
import { Table, TableRow, TableCell, TableHeader } from "@tiptap/extension-table";
import { TaskList } from "@tiptap/extension-task-list";
import { TaskItem } from "@tiptap/extension-task-item";
import { Text } from "@tiptap/extension-text";
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

/// The one place the document's shape is decided. Tests build editors from this
/// same list, or they would be proving a copy of the configuration rather than
/// the editor a person types into.
export const written = () => [
  StarterKit.configure({ link: { openOnClick: false, autolink: true }, text: false }),
  Pictured,
  Table.configure({ resizable: false }),
  TableRow,
  TableHeader,
  TableCell,
  TaskList,
  TaskItem.configure({ nested: true }),
  Barred,
  Markdown.configure({ html: true, linkify: true, breaks: true, transformPastedText: false }),
];

/// Null while the editor is being torn down: its storage is already gone.
export const asMarkdown = (editor: Writing): string | null => {
  if (editor.isDestroyed) return null;
  const kept = (editor.storage as unknown as { markdown?: { getMarkdown?: () => string } }).markdown;
  return typeof kept?.getMarkdown === "function" ? kept.getMarkdown() : null;
};

/// Strict Markdown, for pasting where inline html is not welcome. A declared
/// loss: reading it back with html off escapes the tags instead of dropping
/// them, so the tags are removed here rather than reinterpreted.
export const bared = (markdown: string): string => {
  const fence = /^\s*(?:```|~~~)/;
  let inside = false;
  return markdown
    .split("\n")
    .map((line) => {
      if (fence.test(line)) {
        inside = !inside;
        return line;
      }
      if (inside) return line;
      return line.replace(/<\/?u\b[^>]*>/gi, "");
    })
    .join("\n");
};
