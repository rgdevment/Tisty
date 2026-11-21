import { useEffect, useRef } from "react";
import { counted, type Item } from "../attention";
import { t } from "../locales";
import { GROUPS, pageOf } from "../settingsPages";
import type { Tab } from "../views";
import Glyph from "./Glyph";
import { PANEL } from "./settingsPanel";
import { tabKeys } from "./tabbing";
import { typing } from "./typing";

const ALL_TABS: Tab[] = ["summary", ...GROUPS.flatMap((group) => group.pages)];
const tabId = (one: Tab) => `settings-tab-${one}`;

export interface Finding {
  attention: Item[] | null;
  finding: string;
  setFinding: (query: string) => void;
}

interface Props {
  tab: Tab;
  waiting?: string;
  onTab: (tab: Tab) => void;
  onBack: () => void;
  keeping?: Finding;
}

export default function SettingsNav({ tab, waiting, onTab, onBack, keeping }: Props) {
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
  const many = keeping?.attention ? counted(keeping.attention) : null;
  const finding = keeping?.finding ?? "";
  const go = (one: Tab) => {
    keeping?.setFinding("");
    onTab(one);
  };
  const shown = finding ? undefined : tab;
  const moved = tabKeys(ALL_TABS, true, tabId);

  return (
    <>
      <button
        type="button"
        onClick={onBack}
        aria-label={t("settingsBackTo")}
        title={t("settingsBackTo")}
        className="mb-2.5 flex w-full items-center gap-1.5 rounded-[10px] border border-line px-2.5 py-1.5 text-[13px] text-ink hover:bg-hover"
      >
        <Glyph name="chevron" className="size-3.5 rotate-90" />
        <span className="flex-1 text-left">{t("settingsBack")}</span>
        <kbd className="rounded-md border border-line px-1.5 font-sans text-[10.5px] text-faint">
          Esc
        </kbd>
      </button>
      {keeping && <Find asked={finding} onAsk={keeping.setFinding} onLeave={onBack} />}
      <div role="tablist" aria-orientation="vertical" className="flex flex-col gap-px">
        <Entry tab={shown} one="summary" onTab={go} onKey={moved} />
        {GROUPS.map((group) => (
          <div key={group.pages[0]} role="presentation" className="flex flex-col gap-px">
            <p className="px-2.5 pt-3 pb-1 text-[11.5px] font-semibold tracking-[0.06em] text-faint uppercase">
              {group.brand ?? (group.label && t(group.label))}
            </p>
            {group.pages.map((one) => (
              <Entry
                key={one}
                tab={shown}
                one={one}
                onKey={moved}
                many={many?.[one]}
                dot={!many && one === "about" && waiting ? t("updateWaiting") : undefined}
                onTab={go}
              />
            ))}
          </div>
        ))}
      </div>
    </>
  );
}

function Find({
  asked,
  onAsk,
  onLeave,
}: {
  asked: string;
  onAsk: (query: string) => void;
  onLeave: () => void;
}) {
  return (
    <label className="mb-3 flex items-center gap-1.5 rounded-[10px] border border-hair bg-bg px-2.5 py-[5px] text-faint focus-within:border-line">
      <Glyph name="search" className="h-[13px] w-[13px] shrink-0" />
      <input
        type="search"
        value={asked}
        placeholder={t("findSetting")}
        aria-label={t("findSetting")}
        autoComplete="off"
        spellCheck={false}
        onChange={(e) => onAsk(e.target.value)}
        onKeyDown={(e) => {
          if (e.key !== "Escape") return;
          e.preventDefault();
          if (asked) onAsk("");
          else onLeave();
        }}
        className="min-w-0 flex-1 bg-transparent text-[12.5px] text-ink outline-none placeholder:text-faint [&::-webkit-search-cancel-button]:hidden"
      />
    </label>
  );
}

function Entry({
  tab,
  one,
  many,
  dot,
  onTab,
  onKey,
}: {
  tab?: Tab;
  one: Tab;
  many?: number;
  dot?: string;
  onTab: (tab: Tab) => void;
  onKey: (event: React.KeyboardEvent<HTMLElement>, at: Tab) => void;
}) {
  const page = pageOf(one);
  return (
    <Section
      id={tabId(one)}
      icon={page.icon}
      label={t(page.label)}
      on={tab === one}
      reachable={(tab ?? "summary") === one}
      many={many}
      dot={dot}
      onClick={() => onTab(one)}
      onKeyDown={(event) => onKey(event, one)}
    />
  );
}

function Section({
  id,
  icon,
  label,
  on,
  reachable,
  many,
  dot,
  onClick,
  onKeyDown,
}: {
  id: string;
  icon: string;
  label: string;
  on: boolean;
  reachable: boolean;
  many?: number;
  dot?: string;
  onClick: () => void;
  onKeyDown: (event: React.KeyboardEvent<HTMLElement>) => void;
}) {
  const told = many ? `${label} · ${many}` : dot ? `${label} · ${dot}` : undefined;
  return (
    <button
      id={id}
      type="button"
      role="tab"
      aria-selected={on}
      aria-controls={PANEL}
      aria-label={told}
      tabIndex={reachable ? 0 : -1}
      onClick={onClick}
      onKeyDown={onKeyDown}
      className={`flex items-center gap-2.5 rounded-[10px] px-2.5 py-1.5 text-left text-[13px] text-ink hover:bg-hover ${
        on ? "bg-active font-semibold" : ""
      }`}
    >
      <span className={`flex w-4 shrink-0 justify-center ${on ? "text-accent" : "text-soft"}`}>
        <Glyph name={icon} />
      </span>
      <span className="truncate">{label}</span>
      {many ? (
        <span
          aria-hidden="true"
          className="ml-auto min-w-[17px] rounded-full bg-hue-amber/10 px-1.5 text-center text-[10.5px] font-semibold text-hue-amber"
        >
          {many}
        </span>
      ) : (
        dot && <span aria-hidden="true" className="ml-auto size-1.5 rounded-full bg-accent" />
      )}
    </button>
  );
}
