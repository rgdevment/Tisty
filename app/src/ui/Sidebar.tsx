import { useState } from "react";
import type { List } from "../core";
import { t } from "../locales";
import type { Chosen, Named } from "../views";

interface Props {
  lists: List[];
  counts: Record<string, number>;
  chosen: Chosen;
  onChoose: (chosen: Chosen) => void;
}

const NAMED: { key: Named; icon: string }[] = [
  { key: "search", icon: "⌕" },
  { key: "inbox", icon: "▤" },
  { key: "today", icon: "☀" },
  { key: "upcoming", icon: "▦" },
  { key: "tags", icon: "◈" },
  { key: "archive", icon: "▣" },
];

export default function Sidebar({ lists, counts, chosen, onChoose }: Props) {
  const [openLists, setOpenLists] = useState(true);

  const settled = lists.filter((list) => !counts[list.id]);
  const working = lists.filter((list) => counts[list.id]);

  return (
    <aside className="flex flex-col overflow-hidden border-r border-hair bg-rail">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="px-2.5 pb-1.5">
        <nav className="flex flex-col gap-px">
          {NAMED.map(({ key, icon }) => (
            <Entry
              key={key}
              icon={icon}
              label={t(key)}
              count={counts[key]}
              on={!chosen.list && !chosen.tags?.length && chosen.named === key}
              onClick={() => onChoose({ named: key })}
            />
          ))}
        </nav>
      </div>

      <div className="scroller flex-1 px-2.5 pb-4">
        <button
          onClick={() => setOpenLists((open) => !open)}
          className="flex w-full items-center gap-1.5 px-2.5 pt-4 pb-1.5 text-[11px] font-semibold tracking-[0.06em] text-faint uppercase"
        >
          <span className={`text-[9px] transition-transform ${openLists ? "" : "-rotate-90"}`}>▼</span>
          {t("lists")}
          <span className="ml-auto text-[11px] font-normal">{lists.length || ""}</span>
        </button>

        {openLists && (
          <nav className="flex flex-col gap-px">
            {working.map((list) => (
              <Entry
                key={list.id}
                label={list.name}
                count={counts[list.id]}
                on={chosen.list === list.id}
                onClick={() => onChoose({ list: list.id })}
              />
            ))}
            {settled.length > 0 && working.length > 0 && (
              <div className="mx-3 my-1.5 h-px bg-hair" />
            )}
            {settled.map((list) => (
              <Entry
                key={list.id}
                label={list.name}
                count="✓"
                muted
                on={chosen.list === list.id}
                onClick={() => onChoose({ list: list.id })}
              />
            ))}
          </nav>
        )}
      </div>
    </aside>
  );
}

interface EntryProps {
  icon?: string;
  label: string;
  count?: number | string;
  on?: boolean;
  muted?: boolean;
  onClick?: () => void;
}

function Entry({ icon, label, count, on, muted, onClick }: EntryProps) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-2.5 rounded-[7px] px-2.5 py-1.5 text-left text-[13.5px] hover:bg-hover ${
        on ? "bg-active font-semibold" : ""
      } ${muted ? "text-faint" : "text-ink"}`}
    >
      {icon && (
        <span className={`w-4 shrink-0 text-center text-[13px] ${on ? "text-accent" : "text-soft"}`}>
          {icon}
        </span>
      )}
      <span className="truncate">{label}</span>
      {count !== undefined && count !== 0 && (
        <span className="ml-auto text-xs text-faint tabular-nums">{count}</span>
      )}
    </button>
  );
}
