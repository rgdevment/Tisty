import { updateAutomatic } from "../core";
import { t } from "../locales";
import Corner from "./Corner";

interface Props {
  apart: string;
  onSettled: () => void;
  onError: (problem: unknown) => void;
}

export default function Alone({ apart, onSettled, onError }: Props) {
  const settle = (wanted: boolean) => {
    onSettled();
    updateAutomatic(wanted).catch(onError);
  };

  return (
    <Corner
      apart={apart}
      label={t("aloneLabel")}
      mark={
        <span
          aria-hidden="true"
          className="mt-px size-[13px] shrink-0 rounded-full bg-accent-soft"
        />
      }
      title={t("aloneTitle")}
      why={t("aloneWhy")}
      go={{ label: t("aloneGo"), onClick: () => settle(true) }}
      no={{ label: t("aloneNo"), onClick: () => settle(false) }}
      later={{ label: t("aloneLater"), onClick: onSettled }}
    />
  );
}
