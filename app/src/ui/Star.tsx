import { openUrl } from "@tauri-apps/plugin-opener";
import { starDone } from "../core";
import { t } from "../locales";
import Corner from "./Corner";

const REPO = "https://github.com/rgdevment/Tisty";

interface Props {
  apart: string;
  onSettled: () => void;
  onError: (problem: unknown) => void;
}

export default function Star({ apart, onSettled, onError }: Props) {
  const settle = (open: boolean) => {
    onSettled();
    starDone().catch(onError);
    if (open) openUrl(REPO).catch(onError);
  };

  return (
    <Corner
      apart={apart}
      label={t("supportTitle")}
      mark={<StarMark className="mt-px size-[15px] shrink-0 text-hue-amber" />}
      title={t("starThanks")}
      why={t("starWhy")}
      go={{ label: t("starGo"), onClick: () => settle(true) }}
      no={{ label: t("starNo"), onClick: () => settle(false) }}
      later={{ label: t("starLater"), onClick: onSettled }}
    />
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
