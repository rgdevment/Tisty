import { useEffect, useRef, useState } from "react";
import { EditorContent, useEditor } from "@tiptap/react";
import type { Editor as Writing } from "@tiptap/core";
import { asMarkdown, ruined, written } from "./writing";
import { convertFileSrc } from "@tauri-apps/api/core";
import { served } from "../core";
import { CATCHES, takesFiles } from "../dropped";
import { t } from "../locales";
import Slash, { asked, narrowed, type Block } from "./Slash";
import Floats from "./Floats";

const middle = (editor: Writing, from: number, to: number) => {
  const a = caret(editor, from);
  const b = caret(editor, to);
  return { x: (a.x + b.x) / 2, y: Math.min(a.y, b.y) };
};

/// Where the menu should hang. Only cosmetic, and it throws in environments
/// with no layout, so a miss must not take the menu down with it.
const caret = (editor: Writing, at: number) => {
  try {
    const spot = editor.view.coordsAtPos(at);
    return { x: spot.left, y: spot.bottom };
  } catch {
    return { x: 0, y: 0 };
  }
};

interface Props {
  value: string;
  taking?: boolean;
  label?: string;
  onAttach?: () => Promise<string | null>;
  onRuin?: () => void;
  onOpen?: (reference: string) => void;
  onWrite: (text: string) => void;
}

export default function Editor({
  value,
  taking,
  label,
  onAttach,
  onRuin,
  onOpen,
  onWrite,
}: Props) {
  const [asking, setAsking] = useState<{ at: { x: number; y: number }; word: string } | null>(null);
  const [active, setActive] = useState(0);
  const [picked, setPicked] = useState<{ at: { x: number; y: number } } | null>(null);
  const urls = useRef(new Map<string, string>());
  const now = useRef<{ open: boolean; count: number; take: () => void }>({
    open: false,
    count: 0,
    take: () => {},
  });

  const look = (editor: Writing) => {
    const { $from, $to, empty } = editor.state.selection;
    setPicked(
      empty || editor.isActive("codeBlock") ? null : { at: middle(editor, $from.pos, $to.pos) },
    );
    // A slash inside code is a path or a division, never a command.
    if (!empty || editor.isActive("codeBlock") || editor.isActive("code")) {
      return setAsking(null);
    }
    const word = asked($from.parent.textBetween(0, $from.parentOffset, undefined, " "));
    if (word === null) return setAsking(null);
    setAsking({ at: caret(editor, $from.pos), word });
    setActive(0);
  };

  const editor = useEditor({
    autofocus: taking,
    extensions: written(),
    content: value,
    editorProps: {
      attributes: {
        class: "tisty-doc",
        [CATCHES]: "",
        role: "textbox",
        "aria-multiline": "true",
        spellcheck: "true",
        ...(label ? { "aria-label": label } : {}),
      },
      transformPastedHTML: (html) =>
        // A pasted picture carries the path it came from — often an absolute
        // one with the person's name — and that would be written into a file
        // that travels between machines.
        html.replace(/<img\b[^>]*>/gi, (tag) =>
          /\bsrc\s*=\s*["'](?!https?:|attachments\/)/i.test(tag) ? "" : tag,
        ),
      handleClick: (view, pos) => {
        // Read from the document, never from the DOM: dressing a mark's node
        // would propagate into the saved markdown, unlike an image, which is a
        // leaf ProseMirror refuses to read back.
        const at = view.state.doc.resolve(pos);
        const picture = view.state.doc.nodeAt(pos);
        const src = picture?.type.name === "image" ? String(picture.attrs.src ?? "") : "";
        const link = at
          .marks()
          .find((one) => one.type.name === "link")
          ?.attrs.href;
        const target = src || String(link ?? "");
        if (!target || /^(https?|mailto|tel):/i.test(target)) return false;
        onOpen?.(target);
        return true;
      },
      handleKeyDown: (_, e) => {
        if (!now.current.open) return false;
        const count = now.current.count;
        if (e.key === "ArrowDown") {
          setActive((was) => (was + 1) % count);
          return true;
        }
        if (e.key === "ArrowUp") {
          setActive((was) => (was - 1 + count) % count);
          return true;
        }
        if (e.key === "Enter" || e.key === "Tab") {
          now.current.take();
          return true;
        }
        if (e.key === "Escape") {
          setAsking(null);
          return true;
        }
        return false;
      },
    },
    onUpdate: ({ editor }) => {
      const text = asMarkdown(editor);
      if (text !== null && ruined(text)) {
        onRuin?.();
        look(editor);
        return;
      }
      if (text !== null) onWrite(text);
      look(editor);
    },
    onSelectionUpdate: ({ editor }) => look(editor),
  });

  const blocks: Block[] = editor
    ? [
        {
          key: "h1",
          label: t("bigTitle"),
          hint: "#",
          icon: "H",
          run: () => editor.chain().focus().toggleHeading({ level: 1 }).run(),
        },
        {
          key: "h2",
          label: t("midTitle"),
          hint: "##",
          icon: "H",
          run: () => editor.chain().focus().toggleHeading({ level: 2 }).run(),
        },
        {
          key: "bullets",
          label: t("bullets"),
          hint: "-",
          icon: "•",
          run: () => editor.chain().focus().toggleBulletList().run(),
        },
        {
          key: "numbers",
          label: t("numbers"),
          hint: "1.",
          icon: "1",
          run: () => editor.chain().focus().toggleOrderedList().run(),
        },
        {
          key: "todo",
          label: t("checks"),
          hint: "[ ]",
          icon: "☑",
          run: () => editor.chain().focus().toggleTaskList().run(),
        },
        {
          key: "quote",
          label: t("quote"),
          hint: ">",
          icon: "❝",
          run: () => editor.chain().focus().toggleBlockquote().run(),
        },
        {
          key: "code",
          label: t("codeBlock"),
          hint: "```",
          icon: "⌗",
          run: () => editor.chain().focus().toggleCodeBlock().run(),
        },
        {
          key: "table",
          label: t("table"),
          hint: "|",
          icon: "⊞",
          run: () =>
            editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(),
        },
        {
          key: "rule",
          label: t("divider"),
          hint: "---",
          icon: "—",
          run: () => editor.chain().focus().setHorizontalRule().run(),
        },
        ...(onAttach
          ? [
              {
                key: "attach",
                label: t("attachment"),
                hint: "↧",
                icon: "◫",
                run: () => {
                  onAttach().then((markdown) => {
                    if (markdown) editor.chain().focus().insertContent(markdown).run();
                  });
                },
              },
            ]
          : []),
      ]
    : [];

  const shown = asking ? narrowed(blocks, asking.word) : [];

  const take = (block: Block) => {
    if (!editor || !asking) return;
    const to = editor.state.selection.$from.pos;
    editor
      .chain()
      .focus()
      .deleteRange({ from: to - asking.word.length - 1, to })
      .run();
    setAsking(null);
    block.run();
  };

  now.current = {
    open: Boolean(asking) && shown.length > 0,
    count: shown.length,
    take: () => take(shown[Math.min(active, shown.length - 1)]),
  };

  useEffect(() => {
    if (!editor || editor.isDestroyed || asMarkdown(editor) === value) return;
    editor.commands.setContent(value, { emitUpdate: false });
  }, [editor, value]);

  // The file lands outside React: the window hands it to whatever element under
  // the cursor registered itself, the same way a task's field does.
  useEffect(() => {
    if (!editor || editor.isDestroyed) return;
    return takesFiles(editor.view.dom, (put) => {
      editor.chain().focus().insertContent(put).run();
    });
  }, [editor]);

  // The document keeps the reference a person can read; only the pixels on
  // screen need the servable url, so the markdown is never touched.
  useEffect(() => {
    if (!editor || editor.isDestroyed) return;
    let live = true;
    const dress = () => {
      editor.view.dom.querySelectorAll<HTMLImageElement>("img[src]").forEach((img) => {
        const at = img.getAttribute("src") ?? "";
        if (!at || at.startsWith("http") || at.startsWith("data:")) return;
        const cached = urls.current.get(at);
        if (cached) return void img.setAttribute("src", cached);
        served(at)
          .then((real) => {
            const url = convertFileSrc(real);
            urls.current.set(at, url);
            if (live) img.setAttribute("src", url);
          })
          .catch(() => undefined);
      });
    };
    dress();
    editor.on("update", dress);
    return () => {
      live = false;
      editor.off("update", dress);
    };
  }, [editor, value]);

  return (
    <>
      <EditorContent editor={editor} className="scroller min-h-0 flex-1" />
      {asking && shown.length > 0 && (
        <Slash at={asking.at} blocks={shown} active={active} onPick={take} />
      )}
      {picked && editor && !asking && <Floats editor={editor} at={picked.at} />}
    </>
  );
}
