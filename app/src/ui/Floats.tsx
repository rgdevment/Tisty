import type { Editor as Writing } from "@tiptap/core";
import { t } from "../locales";

interface Props {
  editor: Writing;
  at: { x: number; y: number };
}

export default function Floats({ editor, at }: Props) {
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

  const leans = [
    { key: "left", glyph: "≡", name: t("alignLeft") },
    { key: "center", glyph: "≡", name: t("alignCentre") },
    { key: "right", glyph: "≡", name: t("alignRight") },
    { key: "justify", glyph: "≡", name: t("alignBoth") },
  ] as const;

  const lean = (key: string) => editor.chain().focus().setTextAlign(key).run();

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

      <span aria-hidden className="mx-0.5 h-4 w-px bg-hair" />

      {leans.map((one) => {
        const on = editor.isActive({ textAlign: one.key });
        return (
          <button
            key={one.key}
            type="button"
            aria-label={one.name}
            aria-pressed={on}
            title={one.name}
            onMouseDown={(e) => {
              e.preventDefault();
              lean(one.key);
            }}
            className={`grid h-7 w-7 place-items-center rounded-md text-[11px] ${
              one.key === "center"
                ? "[text-align:center]"
                : one.key === "right"
                  ? "[text-align:right]"
                  : ""
            } ${on ? "bg-accent-soft text-accent" : "text-soft hover:bg-hover hover:text-ink"}`}
          >
            <span className={one.key === "justify" ? "tracking-tighter" : ""}>{one.glyph}</span>
          </button>
        );
      })}
    </div>
  );
}
