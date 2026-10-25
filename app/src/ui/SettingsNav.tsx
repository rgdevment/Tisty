import { useEffect, useRef } from "react";
import { t } from "../locales";
import type { Tab } from "../views";
import Glyph from "./Glyph";

export const SECTIONS: { key: Tab; label: Parameters<typeof t>[0]; icon: string }[] = [
  { key: "general", label: "tabGeneral", icon: "settings" },
  { key: "sync", label: "tabSync", icon: "cloud" },
  { key: "security", label: "tabSecurity", icon: "shield" },
  { key: "data", label: "tabData", icon: "database" },
  { key: "agents", label: "tabAgents", icon: "wand" },
  { key: "upkeep", label: "tabUpkeep", icon: "tools" },
];

interface Props {
  tab: Tab;
  waiting?: string;
  onTab: (tab: Tab) => void;
  onBack: () => void;
}

const typing = (at: EventTarget | null) =>
  at instanceof HTMLElement &&
  (at.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(at.tagName));

export default function SettingsNav({ tab, waiting, onTab, onBack }: Props) {
  const back = useRef(onBack);
  back.current = onBack;
  useEffect(() => {
    const leave = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || event.defaultPrevented || typing(event.target)) return;
      if (document.querySelector("[role=dialog]")) return;
      back.current();
    };
    window.addEventListener("keydown", leave);
    return () => window.removeEventListener("keydown", leave);
  }, []);

  return (
    <>
      <button
        type="button"
        onClick={onBack}
        className="mb-2 flex items-center gap-1.5 rounded-[10px] px-2.5 py-1.5 text-[13px] text-soft hover:bg-hover"
      >
        <Glyph name="chevron" className="size-3.5 rotate-90" />
        {t("settingsBack")}
      </button>
      <p className="px-2.5 pb-1 text-[11.5px] font-semibold tracking-[0.06em] text-faint uppercase">
        {t("keeping")}
      </p>
      <div role="tablist" aria-orientation="vertical" className="flex flex-col gap-px">
        {SECTIONS.map((one) => (
          <Section
            key={one.key}
            icon={one.icon}
            label={t(one.label)}
            on={tab === one.key}
            onClick={() => onTab(one.key)}
          />
        ))}
        <div className="mx-1 my-1.5 h-px bg-hair" />
        <Section
          icon="info"
          label={t("aboutScreen")}
          on={tab === "about"}
          dot={waiting ? t("updateWaiting") : undefined}
          onClick={() => onTab("about")}
        />
      </div>
    </>
  );
}

function Section({
  icon,
  label,
  on,
  dot,
  onClick,
}: {
  icon: string;
  label: string;
  on: boolean;
  dot?: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      role="tab"
      aria-selected={on}
      aria-label={dot ? `${label} · ${dot}` : undefined}
      onClick={onClick}
      className={`flex items-center gap-2.5 rounded-[10px] px-2.5 py-1.5 text-left text-[13px] text-ink hover:bg-hover ${
        on ? "bg-active font-semibold" : ""
      }`}
    >
      <span className={`flex w-4 shrink-0 justify-center ${on ? "text-accent" : "text-soft"}`}>
        <Glyph name={icon} />
      </span>
      <span className="truncate">{label}</span>
      {dot && <span aria-hidden="true" className="ml-auto size-1.5 rounded-full bg-accent" />}
    </button>
  );
}
