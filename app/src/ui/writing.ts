import StarterKit from "@tiptap/starter-kit";
import { Image } from "@tiptap/extension-image";
import { Table, TableRow, TableCell, TableHeader } from "@tiptap/extension-table";
import { TaskList } from "@tiptap/extension-task-list";
import { TaskItem } from "@tiptap/extension-task-item";
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
