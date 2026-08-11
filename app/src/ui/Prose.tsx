import { useEffect, useRef, useState } from "react";
import { CATCHES, takesFiles } from "../dropped";
import { composed } from "../markdown";
import { t } from "../locales";
import Composed from "./Composed";
import Insert from "./Insert";

interface Props {
  value: string;
  hint: string;
  label: string;
  steps?: string[];
  beside?: boolean;
  rows?: number;
  onWrite: (text: string) => void;
  onError?: (problem: unknown) => void;
  /** Given only where there is somewhere roomier to send the reader. */
  onWhole?: () => void;
  /** True on the field a dropped file should land in. */
  catches?: boolean;
}

/** Source where the cursor is, composed where it is not: no preview button, no mode to learn. */
export default function Prose({
  value,
  hint,
  label,
  steps,
  beside,
  rows = 2,
  onWrite,
  onError,
  onWhole,
  catches,
}: Props) {
  const [text, setText] = useState(value);
  const [writing, setWriting] = useState(false);
  const [slash, setSlash] = useState<number | null>(null);
  const box = useRef<HTMLTextAreaElement>(null);
  const dropped = useRef(false);

  useEffect(() => setText(value), [value]);

  // The two faces are different elements, so focus goes after the swap.
  useEffect(() => {
    if (writing) box.current?.focus();
  }, [writing]);

  // The file arrives from outside React, so the field registers its element.
  const [mine, setMine] = useState<HTMLElement | null>(null);
  useEffect(() => {
    if (!mine || !catches) return;
    return takesFiles(mine, (written) => {
      setWriting(true);
      setText((held) => {
        const kept = held.replace(/\s+$/, "");
        return kept ? [kept, "", written].join("\n") : written;
      });
      dropped.current = false;
    });
  }, [mine, catches]);

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

  const caught = catches ? { [CATCHES]: "", ref: setMine } : {};

  const source = (
    <div className="relative" {...caught}>
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
        }}
        className="field-sizing-content max-h-[26rem] w-full resize-none overflow-y-auto rounded-md bg-transparent px-1.5 py-1 font-mono text-[12.5px] leading-relaxed outline-none placeholder:text-faint hover:bg-hover focus:bg-hover"
      />
      {slash !== null && (
        <Insert
          steps={steps}
          onPut={put}
          onError={onError}
          onClose={() => {
            setSlash(null);
            box.current?.focus();
          }}
        />
      )}
    </div>
  );

  const read = (
    <div {...caught}>
      <Composed
        tabIndex={0}
        label={label}
        html={text.trim() ? composed(text, steps) : placeholder(hint)}
        onEnter={() => setWriting(true)}
        onError={onError}
        onWhole={onWhole}
        className="prose cursor-text rounded-md px-1.5 py-1 text-[13.5px] leading-relaxed outline-none hover:bg-hover focus-visible:ring-2 focus-visible:ring-accent"
      />
    </div>
  );

  if (!writing) return read;
  if (!beside) return source;

  return (
    <div className="grid grid-cols-2 items-start gap-5">
      {source}
      <Composed
        label={t("composed")}
        onError={onError}
        html={composed(text, steps)}
        className="prose px-1.5 py-1 text-[13.5px] leading-relaxed"
      />
    </div>
  );
}

const placeholder = (hint: string): string =>
  `<p class="text-faint">${hint.replace(/[<>&]/g, "")}</p>`;
