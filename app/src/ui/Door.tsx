import { doorDone } from "../core";
import { t } from "../locales";
import Corner from "./Corner";

interface Props {
  apart: string;
  onSettled: () => void;
  onOpen: () => void;
  onError: (problem: unknown) => void;
}

export default function Door({ apart, onSettled, onOpen, onError }: Props) {
  const settle = (open: boolean) => {
    onSettled();
    doorDone().catch(onError);
    if (open) onOpen();
  };

  return (
    <Corner
      apart={apart}
      label={t("doorTitle")}
      mark={<DoorMark className="mt-px size-[15px] shrink-0 text-hue-blue" />}
      title={t("doorThanks")}
      why={t("doorWhy")}
      go={{ label: t("doorGo"), onClick: () => settle(true) }}
      no={{ label: t("doorNo"), onClick: () => settle(false) }}
      later={{ label: t("doorLater"), onClick: onSettled }}
    />
  );
}

function DoorMark({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 16 16" aria-hidden="true" className={className}>
      <path
        fill="currentColor"
        d="M4.4 1.6h7.2c.7 0 1.2.5 1.2 1.2v10.4c0 .7-.5 1.2-1.2 1.2H4.4c-.7 0-1.2-.5-1.2-1.2V2.8c0-.7.5-1.2 1.2-1.2zm5.8 6.9a.95.95 0 100-1.9.95.95 0 000 1.9z"
      />
    </svg>
  );
}
