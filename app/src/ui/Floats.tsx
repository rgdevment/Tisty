import type { Editor as Writing } from "@tiptap/core";
import { useEffect, useRef, useState } from "react";
import { addressed } from "../linking";
import { t } from "../locales";
import { named, pictured, previewOf } from "../previews";

interface Props {
  editor: Writing;
  at: { x: number; y: number };
  asking?: boolean;
  onDone?: () => void;
}

type Span = { from: number; to: number; words: string };

const within = (y: number, tall: number): number =>
  Math.max(8, Math.min(y, window.innerHeight - tall - 8));

const spanOf = (editor: Writing): Span => {
  const { from, to } = editor.state.selection;
  return { from, to, words: editor.state.doc.textBetween(from, to) };
};

export default function Floats({ editor, at, asking, onDone }: Props) {
  const live = spanOf(editor);
  const [linking, setLinking] = useState<{ words: string; where: string; held: Span } | null>(
    asking ? { words: live.words, where: "", held: live } : null,
  );
  const [wrong, setWrong] = useState(false);
  const [reached, setReached] = useState(0);
  const card = useRef<HTMLDivElement | null>(null);

  const locked = editor.isActive("codeBlock");
  const href = String(editor.getAttributes("link").href ?? "");
  const cardable = Boolean(href) && !pictured(href) && Boolean(previewOf(href));

  const asCard = () => {
    const said = live.words.trim() || named(href);
    editor
      .chain()
      .focus()
      .setTextSelection(live)
      .deleteSelection()
      .insertContent({ type: "image", attrs: { src: href, alt: said } })
      .run();
    onDone?.();
  };

  const marks = [
    { key: "bold", glyph: "B", name: t("bold"), weight: "font-bold" },
    { key: "italic", glyph: "I", name: t("italic"), weight: "italic font-serif" },
    { key: "underline", glyph: "U", name: t("underlined"), weight: "underline" },
    { key: "strike", glyph: "S", name: t("struck"), weight: "line-through" },
    { key: "code", glyph: "‹›", name: t("codeSpan"), weight: "font-mono" },
    { key: "pen", glyph: "A", name: t("penIt"), weight: "rounded-[3px] bg-pen-yellow px-1" },
  ] as const;

  const kinds = ["note", "tip", "important", "warning", "caution"] as const;
  const said = String(editor.getAttributes("callout").kind ?? "");

  const pens = [
    { key: "green", name: t("penGreen"), paint: "bg-pen-green" },
    { key: "blue", name: t("penBlue"), paint: "bg-pen-blue" },
    { key: "pink", name: t("penPink"), paint: "bg-pen-pink" },
  ] as const;

  const inked = (pen: string) => {
    const chain = editor.chain().focus().setTextSelection(live);
    if (editor.isActive("highlight", { color: pen })) return chain.unsetHighlight().run();
    chain.setHighlight({ color: pen }).run();
  };

  const turn = (key: string) => {
    const chain = editor.chain().focus().setTextSelection(live);
    if (key === "bold") return chain.toggleBold().run();
    if (key === "italic") return chain.toggleItalic().run();
    if (key === "underline") return chain.toggleUnderline().run();
    if (key === "strike") return chain.toggleStrike().run();
    if (key === "pen") return chain.toggleHighlight().run();
    chain.toggleCode().run();
  };

  const leave = (held: Span) => {
    setLinking(null);
    editor.chain().focus().setTextSelection(held).run();
    onDone?.();
  };

  const tie = (words: string, where: string, held: Span) => {
    const target = where.trim();
    if (!target) {
      editor.chain().focus().setTextSelection(held).unsetLink().run();
      setLinking(null);
      return onDone?.();
    }
    const full = addressed(target);
    if (!full) return setWrong(true);

    const lead = held.words.length - held.words.trimStart().length;
    const tail = held.words.length - held.words.trimEnd().length;
    const tight: Span = {
      from: held.from + lead,
      to: held.to - tail,
      words: held.words.trim(),
    };

    const said = words.trim() || (tight.from === tight.to ? target : "");
    const chain = editor.chain().focus().setTextSelection(tight);
    if (said && said !== tight.words) {
      chain
        .insertContentAt(tight, { type: "text", text: said })
        .setTextSelection({ from: tight.from, to: tight.from + said.length });
    }
    if (!chain.setLink({ href: full }).run()) return setWrong(true);
    setLinking(null);
    onDone?.();
  };

  useEffect(() => {
    if (!linking) return;
    const away = (e: MouseEvent) => {
      if (!card.current?.contains(e.target as Node)) leave(linking.held);
    };
    document.addEventListener("mousedown", away);
    return () => document.removeEventListener("mousedown", away);
  });

  const walk = (by: number) => {
    const all = [...(card.current?.querySelectorAll<HTMLElement>("[data-tool]") ?? [])];
    for (let step = 1; step <= all.length; step += 1) {
      const next = (reached + by * step + all.length * step) % all.length;
      if ((all[next] as HTMLButtonElement).disabled) continue;
      setReached(next);
      return all[next].focus();
    }
  };

  if (linking !== null) {
    return (
      <form
        ref={card as unknown as React.RefObject<HTMLFormElement>}
        style={{
          left: Math.max(8, Math.min(at.x, window.innerWidth - 300)),
          top: within(at.y - 76, 190),
        }}
        onSubmit={(e) => {
          e.preventDefault();
          tie(linking.words, linking.where, linking.held);
        }}
        onKeyDown={(e) => e.key === "Escape" && leave(linking.held)}
        className="fixed z-40 flex w-[276px] flex-col gap-1 rounded-[10px] border border-hair bg-rail p-1.5 shadow-xl"
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
            onChange={(e) => {
              setWrong(false);
              setLinking({ ...linking, where: e.target.value });
            }}
            placeholder={t("linkTo")}
            aria-label={t("linkTo")}
            aria-invalid={wrong || undefined}
            className={`min-w-0 flex-1 rounded-md px-2 py-1 text-[12.5px] outline-none placeholder:text-faint ${
              wrong ? "bg-urgent/15 text-urgent" : "bg-hover"
            }`}
          />
          <button
            type="submit"
            aria-label={t("linkIt")}
            className="grid h-7 w-7 shrink-0 place-items-center rounded-md text-[12px] text-accent hover:bg-hover"
          >
            ↵
          </button>
        </div>
        {wrong && <p className="px-1 text-[11px] leading-tight text-urgent">{t("notAnAddress")}</p>}
      </form>
    );
  }

  return (
    <div
      ref={card}
      role="toolbar"
      aria-label={t("formatting")}
      style={{
        left: Math.max(8, Math.min(at.x, window.innerWidth - 190)),
        top: within(at.y - 44, 44),
      }}
      onKeyDown={(e) => {
        if (e.key !== "ArrowRight" && e.key !== "ArrowLeft") return;
        e.preventDefault();
        walk(e.key === "ArrowRight" ? 1 : -1);
      }}
      className="fixed z-40 flex items-center gap-0.5 rounded-[10px] border border-hair bg-rail p-1 shadow-xl"
    >
      {said && (
        <select
          data-tool
          tabIndex={-1}
          aria-label={t("calloutKind")}
          title={t("calloutKind")}
          value={said}
          onMouseDown={(e) => e.stopPropagation()}
          onChange={(e) =>
            editor
              .chain()
              .focus()
              .setTextSelection(live)
              .updateAttributes("callout", { kind: e.target.value })
              .run()
          }
          className="mr-0.5 h-7 rounded-md border-0 bg-transparent px-1 text-[12px] text-soft hover:bg-hover"
        >
          {kinds.map((one) => (
            <option key={one} value={one}>
              {t(`said${one}` as Parameters<typeof t>[0])}
            </option>
          ))}
        </select>
      )}

      {marks.map((one, i) => {
        const on = editor.isActive(one.key);
        return (
          <button
            key={one.key}
            type="button"
            data-tool
            tabIndex={reached === i ? 0 : -1}
            disabled={locked && one.key !== "code"}
            aria-label={one.name}
            aria-pressed={on}
            title={one.name}
            onMouseDown={(e) => e.preventDefault()}
            onFocus={() => setReached(i)}
            onClick={() => turn(one.key)}
            className={`grid h-7 w-7 place-items-center rounded-md text-[12.5px] ${one.weight} ${
              on ? "bg-accent-soft text-accent" : "text-soft hover:bg-hover hover:text-ink"
            } disabled:cursor-not-allowed disabled:opacity-35 disabled:hover:bg-transparent`}
          >
            {one.glyph}
          </button>
        );
      })}

      {pens.map((one) => (
        <button
          key={one.key}
          type="button"
          data-tool
          tabIndex={-1}
          disabled={locked}
          aria-label={one.name}
          aria-pressed={editor.isActive("highlight", { color: one.key })}
          title={one.name}
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => inked(one.key)}
          className={`grid h-7 w-5 place-items-center rounded-md hover:bg-hover disabled:cursor-not-allowed disabled:opacity-35 disabled:hover:bg-transparent ${
            editor.isActive("highlight", { color: one.key }) ? "bg-accent-soft" : ""
          }`}
        >
          <span className={`h-3 w-3 rounded-[3px] border border-line ${one.paint}`} />
        </button>
      ))}

      <button
        type="button"
        data-tool
        tabIndex={reached === marks.length ? 0 : -1}
        disabled={locked}
        aria-label={t("linkIt")}
        aria-pressed={editor.isActive("link")}
        aria-haspopup="dialog"
        title={t("linkIt")}
        onMouseDown={(e) => e.preventDefault()}
        onFocus={() => setReached(marks.length)}
        onClick={() =>
          setLinking({
            words: live.words,
            where: String(editor.getAttributes("link").href ?? ""),
            held: live,
          })
        }
        className={`grid h-7 w-7 place-items-center rounded-md text-[12px] ${
          editor.isActive("link")
            ? "bg-accent-soft text-accent"
            : "text-soft hover:bg-hover hover:text-ink"
        } disabled:cursor-not-allowed disabled:opacity-35 disabled:hover:bg-transparent`}
      >
        ⚭
      </button>

      {cardable && (
        <button
          type="button"
          data-tool
          tabIndex={reached === marks.length + 2 ? 0 : -1}
          disabled={locked}
          aria-label={t("showAsCard")}
          title={t("showAsCard")}
          onMouseDown={(e) => e.preventDefault()}
          onFocus={() => setReached(marks.length + 2)}
          onClick={asCard}
          className="grid h-7 w-7 place-items-center rounded-md text-[12px] text-soft hover:bg-hover hover:text-ink disabled:cursor-not-allowed disabled:opacity-35"
        >
          ▤
        </button>
      )}
    </div>
  );
}
