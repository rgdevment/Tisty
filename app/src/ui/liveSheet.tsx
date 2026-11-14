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
        className="flex w-full items-center gap-2 px-4 py-2 text-left text-[12.5px] text-faint hover:text-ink"
      >
        <span aria-hidden>{open ? "▾" : "▸"}</span>
        <span>{t("theseLeaves")}</span>
      </button>
      {open && <div className="max-h-[45vh] overflow-y-auto">{children}</div>}
    </div>
  );
}

type Turning = {
  readonly markdown: boolean;
  readonly onTurn: (markdown: boolean) => void;
  /** On a task's page it stands on a row of its own, since nothing above the words leaves it room. */
  readonly inRow?: boolean;
};

export function ViewSwitch({ markdown, onTurn, inRow = false }: Turning) {
  const tab = (plainly: boolean, word: "viewPreview" | "viewMarkdown") => (
    <button
      type="button"
      aria-pressed={markdown === plainly}
      onMouseDown={(event) => event.preventDefault()}
      onClick={() => onTurn(plainly)}
      className={`rounded-full px-3 py-1 font-medium text-[12.5px] transition-colors ${
        markdown === plainly ? "bg-sheet text-ink shadow-sm" : "text-faint hover:text-ink"
      }`}
    >
      {t(word)}
    </button>
  );
  return (
    <div
      className={`${inRow ? "mb-2 ml-auto w-fit" : "absolute top-3 right-4 z-10"} flex gap-0.5 rounded-full border border-hair bg-hover p-0.5`}
    >
      {tab(false, "viewPreview")}
      {tab(true, "viewMarkdown")}
    </div>
  );
}
