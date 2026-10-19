import { type DoorDue, doorDone, doorHeard } from "../core";
import { fill, t } from "../locales";
import { clientNamed } from "../who";
import Corner from "./Corner";

interface Props {
  apart: string;
  due: DoorDue;
  onSettled: () => void;
  onOpen: () => void;
  onError: (problem: unknown) => void;
}

export default function Door({ apart, due, onSettled, onOpen, onError }: Props) {
  if (due.why === "knocked") return <Knocked {...{ apart, due, onSettled, onOpen, onError }} />;
  const settle = (open: boolean) => {
    onSettled();
    doorDone().catch(onError);
    if (open) onOpen();
  };
  const mute = due.why === "mute";

  return (
    <Corner
      apart={apart}
      label={t("doorTitle")}
      mark={<DoorMark className="mt-px size-[15px] shrink-0 text-hue-blue" />}
      title={t(mute ? "doorMute" : "doorThanks")}
      why={t(mute ? "doorMuteWhy" : "doorWhy")}
      go={{ label: t(mute ? "doorLetIn" : "doorGo"), onClick: () => settle(true) }}
      no={{ label: t("doorNo"), onClick: () => settle(false) }}
      later={{ label: t("doorLater"), onClick: onSettled }}
    />
  );
}

function Knocked({ apart, due, onSettled, onOpen, onError }: Props) {
  const name = clientNamed(due.by);
  const heard = (open: boolean) => {
    onSettled();
    doorHeard().catch(onError);
    if (open) onOpen();
  };

  return (
    <Corner
      apart={apart}
      label={t("doorTitle")}
      mark={<DoorMark className="mt-px size-[15px] shrink-0 text-hue-blue" />}
      title={name ? fill("doorKnocked", name) : t("doorKnockedUnnamed")}
      why={t("doorKnockedWhy")}
      go={{ label: t("doorLetIn"), onClick: () => heard(true) }}
      later={{ label: t("doorLater"), onClick: () => heard(false) }}
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
