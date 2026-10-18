import type { KeyboardEvent, ReactNode, Ref } from "react";
import { t } from "../locales";

interface Props {
  label: string;
  apart?: string;
  held: Ref<HTMLElement>;
  onKeyDown: (event: KeyboardEvent) => void;
  onExpand?: () => void;
  onClose: () => void;
  footer: ReactNode;
  children: ReactNode;
}

export default function Panel({
  label,
  apart,
  held,
  onKeyDown,
  onExpand,
  onClose,
  footer,
  children,
}: Props) {
  return (
    <aside
      ref={held}
      tabIndex={-1}
      onKeyDown={onKeyDown}
      aria-label={label}
      className={`absolute top-11 bottom-3 z-20 flex w-[380px] flex-col overflow-hidden rounded-[10px] border border-hair bg-sheet shadow-lift [&_.gutter]:px-0 outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-inset ${apart ?? "right-3"}`}
    >
      <div className="flex items-center justify-between gap-1 px-3 pt-2.5 text-[13px] text-faint">
        <button
          type="button"
          onClick={onExpand}
          className="flex h-6 items-center gap-1.5 rounded-md px-2 text-accent hover:bg-hover"
        >
          <span aria-hidden="true">⤢</span>
          {t("expand")}
        </button>
        <button
          type="button"
          onClick={onClose}
          title={t("closePanel")}
          aria-label={t("closePanel")}
          aria-keyshortcuts="Escape"
          className="flex h-6 w-6 items-center justify-center rounded-md hover:bg-hover hover:text-ink"
        >
          <span aria-hidden="true">✕</span>
        </button>
      </div>
      <div className="scroller flex-1 pt-1.5 pb-7 [&>*:not(.task-words)]:px-5">{children}</div>
      {footer}
    </aside>
  );
}
