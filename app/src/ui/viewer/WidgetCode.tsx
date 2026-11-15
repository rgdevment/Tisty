import { useEffect, useRef, useState } from "react";
import { t } from "../../locales";
import { button } from "./Glyphs";

type Props = {
  readonly source: string;
  readonly writable: boolean;
  readonly onDraft: (now: string) => void;
  readonly onClose: () => void;
};

export default function WidgetCode({ source, writable, onDraft, onClose }: Props) {
  const [text, setText] = useState(source);
  const field = useRef<HTMLTextAreaElement>(null);
  useEffect(() => field.current?.focus(), []);

  const written = (next: string) => {
    setText(next);
    onDraft(next);
  };

  return (
    <div className="flex min-h-0 w-1/2 min-w-[300px] shrink-0 flex-col border-r border-hair bg-desk">
      <div className="flex h-10 shrink-0 items-center border-b border-hair bg-panel pr-2.5 pl-4 text-[12.5px]">
        <b className="font-semibold">{t("viewCode")}</b>
        <button
          type="button"
          className={`ml-auto ${button} border-transparent text-soft`}
          onClick={onClose}
        >
          {t("viewClose")}
        </button>
      </div>
      <textarea
        ref={field}
        value={text}
        readOnly={!writable}
        spellCheck={false}
        aria-label={t("viewCode")}
        onChange={(e) => written(e.target.value)}
        onKeyDown={(e) => {
          if (e.key !== "Tab" || e.shiftKey || !writable) return;
          e.preventDefault();
          const box = e.currentTarget;
          const from = box.selectionStart;
          written(`${text.slice(0, from)}  ${text.slice(box.selectionEnd)}`);
          requestAnimationFrame(() => box.setSelectionRange(from + 2, from + 2));
        }}
        className="scroller m-0 min-h-0 flex-1 resize-none bg-transparent px-6 py-5 font-mono text-[12.5px] leading-relaxed text-ink outline-none"
      />
    </div>
  );
}
