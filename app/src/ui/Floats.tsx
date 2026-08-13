import { useState } from "react";
import type { Editor as Writing } from "@tiptap/core";
import { t } from "../locales";

interface Props {
  editor: Writing;
  at: { x: number; y: number };
  asking?: boolean;
  onDone?: () => void;
}

export default function Floats({ editor, at, asking, onDone }: Props) {
  const { from, to } = editor.state.selection;
  const [linking, setLinking] = useState<{ words: string; where: string } | null>(
    asking ? { words: editor.state.doc.textBetween(from, to), where: "" } : null,
  );
  const marks = [
    { key: "bold", glyph: "B", name: t("bold"), weight: "font-bold" },
    { key: "italic", glyph: "I", name: t("italic"), weight: "italic font-serif" },
    { key: "underline", glyph: "U", name: t("underlined"), weight: "underline" },
    { key: "strike", glyph: "S", name: t("struck"), weight: "line-through" },
    { key: "code", glyph: "‹›", name: t("codeSpan"), weight: "font-mono" },
  ] as const;

  const turn = (key: string) => {
    const chain = editor.chain().focus();
    if (key === "bold") return chain.toggleBold().run();
    if (key === "italic") return chain.toggleItalic().run();
    if (key === "underline") return chain.toggleUnderline().run();
    if (key === "strike") return chain.toggleStrike().run();
    chain.toggleCode().run();
  };

  const tie = (words: string, where: string) => {
    const target = where.trim();
    setLinking(null);
    onDone?.();
    if (!target) return editor.chain().focus().unsetLink().run();
    const full = /^[a-z][a-z0-9+.-]*:/i.test(target) ? target : `https://${target}`;
    const said = words.trim();
    const chain = editor.chain().focus();
    if (said && said !== editor.state.doc.textBetween(from, to)) {
      chain.insertContent(said).setTextSelection({ from, to: from + said.length });
    }
    chain.setLink({ href: full }).run();
  };

  if (linking !== null) {
    return (
      <form
        style={{
          left: Math.max(8, Math.min(at.x, window.innerWidth - 300)),
          top: Math.max(8, at.y - 76),
        }}
        onSubmit={(e) => {
          e.preventDefault();
          tie(linking.words, linking.where);
        }}
        onKeyDown={(e) => {
          if (e.key !== "Escape") return;
          setLinking(null);
          onDone?.();
        }}
        className="fixed z-[70] flex w-[276px] flex-col gap-1 rounded-[10px] border border-hair bg-rail p-1.5 shadow-xl"
      >
        <input
          autoFocus
          value={linking.words}
          onChange={(e) => setLinking({ ...linking, words: e.target.value })}
          placeholder={t("linkWords")}
          aria-label={t("linkWords")}
          className="rounded-md bg-hover px-2 py-1 text-[12.5px] outline-none placeholder:text-faint"
        />
        <div className="flex items-center gap-1">
          <input
            value={linking.where}
            onChange={(e) => setLinking({ ...linking, where: e.target.value })}
            placeholder={t("linkTo")}
            aria-label={t("linkTo")}
            className="min-w-0 flex-1 rounded-md bg-hover px-2 py-1 text-[12.5px] outline-none placeholder:text-faint"
          />
          <button
            type="submit"
            aria-label={t("linkIt")}
            className="grid h-7 w-7 shrink-0 place-items-center rounded-md text-[12px] text-accent hover:bg-hover"
          >
            ↵
          </button>
        </div>
      </form>
    );
  }

  return (
    <div
      role="toolbar"
      aria-label={t("formatting")}
      style={{
        left: Math.max(8, Math.min(at.x, window.innerWidth - 190)),
        top: Math.max(8, at.y - 44),
      }}
      className="fixed z-[70] flex items-center gap-0.5 rounded-[10px] border border-hair bg-rail p-1 shadow-xl"
    >
      {marks.map((one) => {
        const on = editor.isActive(one.key);
        return (
          <button
            key={one.key}
            type="button"
            aria-label={one.name}
            aria-pressed={on}
            title={one.name}
            onMouseDown={(e) => {
              e.preventDefault();
              turn(one.key);
            }}
            className={`grid h-7 w-7 place-items-center rounded-md text-[12.5px] ${one.weight} ${
              on ? "bg-accent-soft text-accent" : "text-soft hover:bg-hover hover:text-ink"
            }`}
          >
            {one.glyph}
          </button>
        );
      })}

      <button
        type="button"
        aria-label={t("linkIt")}
        aria-pressed={editor.isActive("link")}
        title={t("linkIt")}
        onMouseDown={(e) => {
          e.preventDefault();
          setLinking({
            words: editor.state.doc.textBetween(from, to),
            where: String(editor.getAttributes("link").href ?? ""),
          });
        }}
        className={`grid h-7 w-7 place-items-center rounded-md text-[12px] ${
          editor.isActive("link")
            ? "bg-accent-soft text-accent"
            : "text-soft hover:bg-hover hover:text-ink"
        }`}
      >
        ⚭
      </button>

    </div>
  );
}
