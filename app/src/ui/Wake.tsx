import { type Waking, wakeFor } from "../core";
import { t } from "../locales";
import { wakeAsked } from "../reminding";
import Corner from "./Corner";

interface Props {
  apart: string;
  due: Waking;
  onSettled: () => void;
  onError: (problem: unknown) => void;
}

export default function Wake({ apart, due, onSettled, onError }: Props) {
  const settle = (wanted: boolean) => {
    onSettled();
    (wanted ? wakeFor(true).then(wakeAsked) : wakeAsked()).catch(onError);
  };

  return (
    <Corner
      apart={apart}
      label={t("wakeOfferLabel")}
      mark={
        <span aria-hidden="true" className="mt-px shrink-0 text-[13px] leading-none">
          ⏰
        </span>
      }
      title={t("wakeOfferTitle")}
      why={t(due.theirs ? "wakeTheirs" : "wakeOfferWhy")}
      go={
        due.theirs
          ? { label: t("wakeOfferSeen"), onClick: () => settle(false) }
          : { label: t("wakeOfferGo"), onClick: () => settle(true) }
      }
      no={due.theirs ? undefined : { label: t("wakeOfferNo"), onClick: () => settle(false) }}
      later={{ label: t("wakeOfferLater"), onClick: onSettled }}
    />
  );
}
