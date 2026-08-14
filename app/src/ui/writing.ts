import StarterKit from "@tiptap/starter-kit";
import { Image } from "@tiptap/extension-image";
import { Table, TableRow, TableCell, TableHeader } from "@tiptap/extension-table";
import { TaskList } from "@tiptap/extension-task-list";
import { TaskItem } from "@tiptap/extension-task-item";
import { Text } from "@tiptap/extension-text";
import { Markdown } from "tiptap-markdown";
import type { Editor as Writing } from "@tiptap/core";

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

export const written = () => [
  StarterKit.configure({ link: { openOnClick: false, autolink: true, protocols: ["tisty"] }, text: false }),
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

export const asMarkdown = (editor: Writing): string | null => {
  if (editor.isDestroyed) return null;
  const kept = (editor.storage as unknown as { markdown?: { getMarkdown?: () => string } }).markdown;
  return typeof kept?.getMarkdown === "function" ? kept.getMarkdown() : null;
};

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
