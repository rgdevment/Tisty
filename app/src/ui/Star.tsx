import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect } from "react";
import { starDone } from "../core";
import { t } from "../locales";

const REPO = "https://github.com/rgdevment/Tisty";

interface Props {
  apart: string;
  onSettled: () => void;
  onError: (problem: unknown) => void;
}

export default function Star({ apart, onSettled, onError }: Props) {
  useEffect(() => {
    const away = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.stopPropagation();
      onSettled();
    };
    window.addEventListener("keydown", away, true);
    return () => window.removeEventListener("keydown", away, true);
  }, [onSettled]);

  const settle = (open: boolean) => {
    onSettled();
    starDone().catch(onError);
    if (open) openUrl(REPO).catch(onError);
  };

  return (
    <section
      role="status"
      aria-label={t("supportTitle")}
      className={`arrive absolute bottom-3 z-30 w-[280px] rounded-[10px] border border-line bg-bg px-[13px] py-3 shadow-lift ${apart}`}
    >
      <div className="flex items-start gap-2.5">
        <StarMark className="mt-px size-[15px] shrink-0 text-hue-amber" />
        <span className="min-w-0 flex-1">
          <b className="block text-[13px] leading-snug font-semibold">{t("starThanks")}</b>
          <span className="mt-1 block text-[11.5px] leading-relaxed text-faint">
            {t("starWhy")}
          </span>
        </span>
        <button
          type="button"
          aria-label={t("starLater")}
          title={t("starLater")}
          onClick={onSettled}
          className="-mr-1 shrink-0 cursor-pointer rounded-md px-1 text-[11.5px] text-faint outline-none hover:text-ink focus-visible:ring-2 focus-visible:ring-accent"
        >
          ✕
        </button>
      </div>
      <div className="mt-3 flex flex-wrap items-center gap-2">
        <button
          type="button"
          onClick={() => settle(true)}
          className="flex cursor-pointer items-center gap-1.5 rounded-[10px] bg-accent px-3 py-1.5 text-[12.5px] font-medium text-bg outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-bg"
        >
          <StarMark className="size-[13px]" />
          {t("starGo")}
        </button>
        <button
          type="button"
          onClick={() => settle(false)}
          className="ml-auto cursor-pointer rounded-[10px] border border-line px-2.5 py-1 text-[12.5px] text-soft outline-none hover:bg-hover focus-visible:ring-2 focus-visible:ring-accent"
        >
          {t("starNo")}
        </button>
      </div>
    </section>
  );
}

function StarMark({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 16 16" aria-hidden="true" className={className}>
      <path
        fill="currentColor"
        d="M8 1.2l2.1 4.3 4.7.7-3.4 3.3.8 4.7L8 12l-4.2 2.2.8-4.7L1.2 6.2l4.7-.7L8 1.2z"
      />
    </svg>
  );
}
