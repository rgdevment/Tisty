import { useEffect, useRef, useState } from "react";
import { EditorContent, useEditor } from "@tiptap/react";
import type { Editor as Writing } from "@tiptap/core";
import { asMarkdown, written } from "./writing";
import { convertFileSrc } from "@tauri-apps/api/core";
import { served, weighs, type Filed } from "../core";
import { previewing, type Reach } from "./previewing";
import { docLink } from "../markdown";
import { CATCHES, takesFiles } from "../dropped";
import { t } from "../locales";
import Slash, { asked, narrowed, type Block } from "./Slash";
import Floats from "./Floats";
import Papers from "./Papers";

export const stripped = (html: string): string =>
  html.replace(/<img\b[^>]*>/gi, (tag) =>
    /\bsrc\s*=\s*["'](?!https?:|attachments\/)/i.test(tag) ? "" : tag,
  );

const middle = (editor: Writing, from: number, to: number) => {
  const a = caret(editor, from);
  const b = caret(editor, to);
  return { x: (a.x + b.x) / 2, y: Math.min(a.y, b.y) };
};

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
  papers?: Filed[];
  onAttach?: () => Promise<string | null>;
  onOpen?: (reference: string) => void;
  onDoc?: (id: string) => void;
  onWrite: (text: string) => void;
}

export default function Editor({
  value,
  taking,
  label,
  papers,
  onAttach,
  onOpen,
  onDoc,
  onWrite,
}: Props) {
  const [asking, setAsking] = useState<{ at: { x: number; y: number }; word: string } | null>(null);
  const [active, setActive] = useState(0);
  const [picked, setPicked] = useState<{ at: { x: number; y: number } } | null>(null);
  const [tying, setTying] = useState<{ x: number; y: number } | null>(null);
  const [choosing, setChoosing] = useState<{ x: number; y: number } | null>(null);
  const urls = useRef(new Map<string, string>());
  const weights = useRef(new Map<string, number>());
  const pending = useRef(new Set<string>());
  const nudge = useRef(() => {});
  const reach = useRef<Reach>({
    url: () => null,
    weight: () => null,
    title: () => null,
  });
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
    extensions: [...written(), previewing(() => reach.current)],
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
      transformPastedHTML: stripped,
      handleClick: (view, pos) => {
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
          key: "link",
          label: t("linkIt"),
          hint: "[ ]( )",
          icon: "⚭",
          run: () => setTying(caret(editor, editor.state.selection.from)),
        },
        {
          key: "paper",
          label: t("insertDoc"),
          hint: "\u25a4",
          icon: "\u25a4",
          run: () => setChoosing(caret(editor, editor.state.selection.from)),
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

  nudge.current = () => {
    if (!editor || editor.isDestroyed) return;
    editor.view.dispatch(editor.state.tr.setMeta("preview", true));
  };

  const fetches = (reference: string, get: () => Promise<void>) => {
    if (pending.current.has(reference)) return;
    pending.current.add(reference);
    get()
      .then(() => nudge.current())
      .catch(() => undefined);
  };

  reach.current = {
    onDoc,
    url: (reference) => {
      const held = urls.current.get(reference);
      if (held) return held;
      fetches(`url:${reference}`, () =>
        served(reference).then((real) => {
          urls.current.set(reference, convertFileSrc(real));
        }),
      );
      return null;
    },
    weight: (reference) => {
      const held = weights.current.get(reference);
      if (held !== undefined) return held;
      fetches(`weight:${reference}`, () =>
        weighs(reference).then((bytes) => {
          weights.current.set(reference, bytes);
        }),
      );
      return null;
    },
    title: (id) => {
      const one = papers?.find((paper) => paper.file === id);
      return one ? one.title.trim() || t("untitledDoc") : null;
    },
  };


  const opened = Boolean(asking) && shown.length > 0;
  const current = Math.min(active, shown.length - 1);

  useEffect(() => {
    const dom = editor && !editor.isDestroyed ? editor.view.dom : null;
    if (!dom) return;
    dom.setAttribute("aria-expanded", String(opened));
    if (!opened) {
      dom.removeAttribute("aria-controls");
      return dom.removeAttribute("aria-activedescendant");
    }
    dom.setAttribute("aria-controls", "slash-menu");
    dom.setAttribute("aria-activedescendant", `slash-${current}`);
  }, [editor, opened, current]);

  useEffect(() => {
    if (!editor || editor.isDestroyed) return;
    return takesFiles(editor.view.dom, (put) => {
      editor.chain().focus().insertContent(put).run();
    });
  }, [editor]);

  useEffect(() => {
    if (!editor || editor.isDestroyed) return;
    let live = true;
    const dress = () =>
      requestAnimationFrame(() => {
        if (editor.isDestroyed) return;
        editor.view.dom.querySelectorAll<HTMLImageElement>("img[src]").forEach((img) => {
          const at = img.getAttribute("src") ?? "";
          if (!at || /^(https?|data|asset|blob|file):/i.test(at)) return;
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
      });

    dress();
    editor.on("transaction", dress);
    return () => {
      live = false;
      editor.off("transaction", dress);
    };
  }, [editor, value]);

  return (
    <>
      <EditorContent editor={editor} className="scroller min-h-0 flex-1" />
      {asking && shown.length > 0 && (
        <Slash at={asking.at} blocks={shown} active={active} onPick={take} />
      )}
      {picked && editor && !asking && !tying && <Floats editor={editor} at={picked.at} />}
      {tying && editor && (
        <Floats editor={editor} at={tying} asking onDone={() => setTying(null)} />
      )}
      {choosing && editor && (
        <div
          style={{
            left: Math.max(8, Math.min(choosing.x, window.innerWidth - 290)),
            top: Math.max(8, choosing.y + 6),
          }}
          className="fixed z-40 w-[272px] rounded-[10px] border border-hair bg-rail p-1.5 shadow-xl"
        >
          <Papers
            all={papers}
            onPick={(paper) => {
              setChoosing(null);
              editor.chain().focus().insertContent(docLink(paper.file, paper.title)).run();
            }}
          />
        </div>
      )}
    </>
  );
}
