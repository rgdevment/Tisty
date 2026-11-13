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
