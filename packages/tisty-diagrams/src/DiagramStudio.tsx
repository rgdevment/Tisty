import { useMemo } from "react";
import Canvas from "./Canvas";
import type { Draw } from "./exporting";
import { HandsGiven, useSay } from "./handing";
import { useEditor } from "./useEditor";
import { sayIn } from "./words";

export interface StudioProps {
  readonly text: string;
  readonly onText: (text: string) => void;
  readonly dark: boolean;
  readonly locale: string;
  readonly draw: Draw;
  readonly copy: (text: string) => Promise<void>;
}

function Sides({ text, onText, dark }: Readonly<Pick<StudioProps, "text" | "onText" | "dark">>) {
  const t = useSay();
  const editor = useEditor(text, onText);
  return (
    <div className="flex min-h-0 flex-1">
      {editor?.board ?? <Canvas source={text} dark={dark} />}
      <aside
        aria-label={t("diagramProperties")}
        className="flex w-[340px] shrink-0 flex-col gap-2 border-l border-hair bg-panel p-4"
      >
        {editor?.panel}
        <div className="flex items-center">
          <label htmlFor="diagram-text" className="text-[12px] font-semibold text-soft">
            {t("diagramText")}
          </label>
          <span className="ml-auto text-[11px] text-accent">{t("diagramLive")}</span>
        </div>
        <textarea
          id="diagram-text"
          spellCheck={false}
          value={text}
          onChange={(e) => onText(e.target.value)}
          className="scroller min-h-0 flex-1 resize-none rounded-md border border-line bg-desk p-3 font-mono text-[12.5px] leading-relaxed text-ink outline-none focus:border-accent"
        />
        <small className="text-[12px] leading-snug text-faint">{t("diagramKeptAsWritten")}</small>
      </aside>
    </div>
  );
}

/** The drawing to work on beside the Mermaid text it is made of; the app around it saves the text. */
export function DiagramStudio({ text, onText, dark, locale, draw, copy }: Readonly<StudioProps>) {
  const hands = useMemo(() => ({ say: sayIn(locale), draw, copy }), [locale, draw, copy]);
  return (
    <HandsGiven value={hands}>
      <Sides text={text} onText={onText} dark={dark} />
    </HandsGiven>
  );
}
