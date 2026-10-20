import { useEffect, useRef, useState } from "react";
import { agentTurn, type DoorDue, doorDone, doorHeard } from "../core";
import { fill, t } from "../locales";
import { clientNamed } from "../who";
import Corner from "./Corner";

const WELCOMED_FOR_MS = 2600;

interface Props {
  apart: string;
  due: DoorDue;
  onSettled: () => void;
  onOpen: () => void;
  onError: (problem: unknown) => void;
}

export default function Door({ apart, due, onSettled, onOpen, onError }: Props) {
  const [welcomed, setWelcomed] = useState(false);
  if (welcomed) {
    const name = clientNamed(due.by);
    return (
      <Welcomed
        apart={apart}
        title={name ? fill("doorWelcomed", name) : t("doorWelcomedUnnamed")}
        onSettled={onSettled}
      />
    );
  }
  const letIn = (heard: () => Promise<void>) => {
    agentTurn(true)
      .then(() => {
        setWelcomed(true);
        return heard();
      })
      .catch(onError);
  };
  if (due.why === "knocked") {
    return (
      <Knocked {...{ apart, due, onSettled, onOpen, onError }} letIn={() => letIn(doorHeard)} />
    );
  }
  const settle = (open: boolean) => {
    onSettled();
    doorDone().catch(onError);
    if (open) onOpen();
  };

  if (due.why === "mute") {
    return (
      <Corner
        apart={apart}
        label={t("doorTitle")}
        mark={<DoorMark className="mt-px size-[15px] shrink-0 text-hue-blue" />}
        title={t("doorMute")}
        why={t("doorMuteWhy")}
        go={{ label: t("doorLetIn"), onClick: () => letIn(doorDone) }}
        aside={{ label: t("doorSeeSettings"), onClick: () => settle(true) }}
        no={{ label: t("doorNo"), onClick: () => settle(false) }}
        later={{ label: t("doorLater"), onClick: onSettled }}
      />
    );
  }

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

function Knocked({ apart, due, onSettled, onOpen, onError, letIn }: Props & { letIn: () => void }) {
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
      go={{ label: t("doorLetIn"), onClick: letIn }}
      aside={{ label: t("doorSeeSettings"), onClick: () => heard(true) }}
      later={{ label: t("doorLater"), onClick: () => heard(false) }}
    />
  );
}

function Welcomed({
  apart,
  title,
  onSettled,
}: {
  apart: string;
  title: string;
  onSettled: () => void;
}) {
  const settled = useRef(onSettled);
  settled.current = onSettled;
  useEffect(() => {
    const away = () => settled.current();
    const timer = window.setTimeout(away, WELCOMED_FOR_MS);
    window.addEventListener("click", away, true);
    window.addEventListener("keydown", away, true);
    return () => {
      window.clearTimeout(timer);
      window.removeEventListener("click", away, true);
      window.removeEventListener("keydown", away, true);
    };
  }, []);

  return (
    <section
      role="status"
      aria-label={t("doorTitle")}
      className={`arrive absolute bottom-3 z-30 w-[280px] rounded-[10px] border border-line bg-bg px-[13px] py-3 shadow-lift ${apart}`}
    >
      <div className="flex items-start gap-2.5">
        <DoorMark className="mt-px size-[15px] shrink-0 text-hue-blue" />
        <b className="block min-w-0 flex-1 text-[13px] leading-snug font-semibold">{title}</b>
      </div>
    </section>
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
