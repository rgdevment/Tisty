import { useEffect } from "react";
import { EditorContent, useEditor } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { Link } from "@tiptap/extension-link";
import { Image } from "@tiptap/extension-image";
import { Table, TableRow, TableCell, TableHeader } from "@tiptap/extension-table";
import { TaskList } from "@tiptap/extension-task-list";
import { TaskItem } from "@tiptap/extension-task-item";
import { Markdown } from "tiptap-markdown";
import type { Editor as Writing } from "@tiptap/core";

// Null while the editor is being torn down: its storage is already gone.
const asMarkdown = (editor: Writing): string | null => {
  if (editor.isDestroyed) return null;
  const kept = (editor.storage as unknown as { markdown?: { getMarkdown?: () => string } }).markdown;
  return typeof kept?.getMarkdown === "function" ? kept.getMarkdown() : null;
};

interface Props {
  value: string;
  taking?: boolean;
  label?: string;
  onWrite: (text: string) => void;
}

export default function Editor({ value, taking, label, onWrite }: Props) {
  const editor = useEditor({
    autofocus: taking,
    extensions: [
      StarterKit,
      Link.configure({ openOnClick: false, autolink: true }),
      Image,
      Table.configure({ resizable: false }),
      TableRow,
      TableHeader,
      TableCell,
      TaskList,
      TaskItem.configure({ nested: true }),
      Markdown.configure({ html: false, linkify: true, breaks: true, transformPastedText: true }),
    ],
    content: value,
    editorProps: {
      attributes: {
        class: "tisty-doc",
        role: "textbox",
        "aria-multiline": "true",
        ...(label ? { "aria-label": label } : {}),
      },
    },
    onUpdate: ({ editor }) => {
      const text = asMarkdown(editor);
      if (text !== null) onWrite(text);
    },
  });

  useEffect(() => {
    if (!editor || editor.isDestroyed || asMarkdown(editor) === value) return;
    editor.commands.setContent(value, { emitUpdate: false });
  }, [editor, value]);

  return <EditorContent editor={editor} className="scroller min-h-0 flex-1" />;
}
