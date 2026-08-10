import { useEffect, useRef, useState } from "react";
import { composed } from "../markdown";
import { t } from "../locales";
import Insert from "./Insert";

interface Props {
  value: string;
  hint: string;
  label: string;
  known: string[];
  beside?: boolean;
  rows?: number;
  /** Enter keeps the entry, as a journal line is written in one go. */
  submits?: boolean;
  onWrite: (text: string) => void;
}

/** Source where the cursor is, composed where it is not: no preview button, no mode to learn. */
export default function Prose({
  value,
  hint,
  label,
  known,
  beside,
  rows = 2,
  submits,
  onWrite,
}: Props) {
  const [text, setText] = useState(value);
  const [writing, setWriting] = useState(false);
  const [slash, setSlash] = useState<number | null>(null);
  const box = useRef<HTMLTextAreaElement>(null);
  const dropped = useRef(false);

  useEffect(() => setText(value), [value]);

  // React reuses the same div across both faces, so focus has to be placed
  // after the swap rather than in the handler that asked for it.
  useEffect(() => {
    if (writing) box.current?.focus();
  }, [writing]);

  const settle = () => {
    setWriting(false);
    setSlash(null);
    if (dropped.current) {
      dropped.current = false;
      setText(value);
      return;
    }
    if (text.trim() !== value.trim()) onWrite(text);
  };

  const put = (snippet: string) => {
    const at = slash ?? text.length;
    const next = `${text.slice(0, at)}${snippet}${text.slice(at + 1)}`;
    setSlash(null);
    setText(next);
    queueMicrotask(() => {
      box.current?.focus();
      box.current?.setSelectionRange(at + snippet.length, at + snippet.length);
    });
  };

  const source = (
    <div className="relative">
      <textarea
        ref={box}
        rows={rows}
        value={text}
        placeholder={hint}
        aria-label={label}
        onChange={(e) => {
          setText(e.target.value);
          const at = e.target.selectionStart - 1;
          setSlash(e.target.value[at] === "/" ? at : null);
        }}
        onFocus={() => setWriting(true)}
        onBlur={() => slash === null && settle()}
        onKeyDown={(e) => {
          if (e.key === "Escape" && slash === null) {
            dropped.current = true;
            e.currentTarget.blur();
          }
          if (e.key === "Enter" && !e.shiftKey && submits) {
            e.preventDefault();
            e.currentTarget.blur();
          }
        }}
        className="field-sizing-content w-full resize-none rounded-md bg-transparent px-1.5 py-1 font-mono text-[12.5px] leading-relaxed outline-none placeholder:text-faint hover:bg-hover focus:bg-hover"
      />
      {slash !== null && (
        <Insert
          known={known}
          onPut={put}
          onClose={() => {
            setSlash(null);
            box.current?.focus();
          }}
        />
      )}
    </div>
  );

  const read = (
    <div
      tabIndex={0}
      aria-label={label}
      onClick={() => setWriting(true)}
      onFocus={() => setWriting(true)}
      className="prose cursor-text rounded-md px-1.5 py-1 text-[13.5px] leading-relaxed outline-none hover:bg-hover"
      dangerouslySetInnerHTML={{ __html: text.trim() ? composed(text) : placeholder(hint) }}
    />
  );

  if (!writing) return read;
  if (!beside) return source;

  return (
    <div className="grid grid-cols-2 items-start gap-5">
      {source}
      <div
        className="prose px-1.5 py-1 text-[13.5px] leading-relaxed"
        aria-label={t("composed")}
        dangerouslySetInnerHTML={{ __html: composed(text) }}
      />
    </div>
  );
}

const placeholder = (hint: string): string =>
  `<p class="text-faint">${hint.replace(/[<>&]/g, "")}</p>`;
