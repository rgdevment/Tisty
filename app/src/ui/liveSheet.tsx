import { type ReactNode, type RefObject, useEffect, useState } from "react";
import { t } from "../locales";

type Framing = {
  readonly lively: boolean;
  readonly frame: () => Window | null;
  readonly putting: RefObject<number>;
  readonly onStuck: (stuck: boolean) => void;
  readonly onSeen: (at: number) => void;
  readonly ready: boolean;
  readonly turned: number;
};

export const useFrameScroll = ({
  lively,
  frame,
  putting,
  onStuck,
  onSeen,
  ready,
  turned,
}: Framing) => {
  // biome-ignore lint/correctness/useExhaustiveDependencies: each drawing may bring the frame anew, and the callbacks read refs.
  useEffect(() => {
    const inner = frame();
    inner?.document.documentElement.classList.toggle("lively", lively);
    if (!lively || !inner) return;
    const rolled = () => {
      const sheet = inner.document.scrollingElement;
      if (!sheet) return;
      onStuck(sheet.scrollTop > 8);
      if (putting.current >= 0 || sheet.scrollHeight <= sheet.clientHeight) return;
      onSeen(sheet.scrollTop);
    };
    inner.addEventListener("scroll", rolled, { passive: true });
    return () => inner.removeEventListener("scroll", rolled);
  }, [lively, ready, turned]);
};

type Below = {
  readonly children: ReactNode;
  readonly folds: boolean;
  readonly width: string;
};

export function LivelyBelow({ children, folds, width }: Below) {
  const [open, setOpen] = useState(false);
  if (!folds) return <div className={`mx-auto shrink-0 ${width}`}>{children}</div>;
  return (
    <div className={`mx-auto shrink-0 ${width}`}>
      <button
        type="button"
        aria-expanded={open}
        onClick={() => setOpen((was) => !was)}
        className="flex w-full items-center gap-1.5 px-2 py-1 text-left text-[11.5px] text-faint hover:text-ink"
      >
        <span aria-hidden>{open ? "▾" : "▸"}</span>
        <span>{t("theseLeaves")}</span>
      </button>
      {open && <div className="max-h-[45vh] overflow-y-auto">{children}</div>}
    </div>
  );
}

export type Viewing = {
  readonly markdown: boolean;
  readonly onTurn: (markdown: boolean) => void;
};

// Above the sheet, beside where it says where it sits: the page as drawn or as written.
export function ViewTabs({ markdown, onTurn }: Viewing) {
  const tab = (plainly: boolean, word: "viewPreview" | "viewMarkdown") => (
    <button
      type="button"
      aria-pressed={markdown === plainly}
      onMouseDown={(event) => event.preventDefault()}
      onClick={() => onTurn(plainly)}
      className="view-tab"
    >
      {t(word)}
    </button>
  );
  return (
    <div role="group" aria-label={t("viewOf")} className="view-tabs">
      {tab(false, "viewPreview")}
      {tab(true, "viewMarkdown")}
    </div>
  );
}

// Beside a task's page one quiet word turns the view, so nothing stands over the words.
export function ViewTurn({ markdown, onTurn }: Viewing) {
  return (
    <button
      type="button"
      aria-pressed={markdown}
      onClick={() => onTurn(!markdown)}
      className="-mr-1.5 ml-auto flex items-center rounded-md px-1.5 py-0.5 text-[11.5px] text-faint transition-colors hover:bg-hover hover:text-ink"
    >
      {t(markdown ? "viewAsPreview" : "viewAsMarkdown")}
    </button>
  );
}
