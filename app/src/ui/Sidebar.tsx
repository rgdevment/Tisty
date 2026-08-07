import { useState } from "react";
import type { List, Task } from "../core";
import { isOverdue, isToday } from "../format";
import { t } from "../locales";

interface Props {
  tasks: Task[];
  lists: List[];
}

export default function Sidebar({ tasks, lists }: Props) {
  const [openLists, setOpenLists] = useState(true);

  const counts = {
    inbox: tasks.filter((task) => !task.list).length,
    today: tasks.filter((task) => task.date && (isToday(task.date) || isOverdue(task.date))).length,
  };

  const active = (list: List) => tasks.filter((task) => task.list === list.id).length;
  const settled = lists.filter((list) => active(list) === 0);
  const working = lists.filter((list) => active(list) > 0);

  return (
    <aside className="flex flex-col overflow-hidden border-r border-hair bg-rail">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="px-2.5 pb-1.5">
        <nav className="flex flex-col gap-px">
          <Entry icon="⌕" label={t("search")} />
          <Entry icon="▤" label={t("inbox")} count={counts.inbox} />
          <Entry icon="☀" label={t("today")} count={counts.today} on />
          <Entry icon="▦" label={t("upcoming")} />
          <Entry icon="◈" label={t("tags")} />
          <Entry icon="▣" label={t("archive")} />
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
              <Entry key={list.id} label={list.name} count={active(list)} />
            ))}
            {settled.length > 0 && working.length > 0 && (
              <div className="mx-3 my-1.5 h-px bg-hair" />
            )}
            {settled.map((list) => (
              <Entry key={list.id} label={list.name} count="✓" muted />
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
}

function Entry({ icon, label, count, on, muted }: EntryProps) {
  return (
    <button
      className={`flex items-center gap-2.5 rounded-[7px] px-2.5 py-1.5 text-left text-[13.5px] hover:bg-hover ${
        on ? "bg-active font-semibold" : ""
      } ${muted ? "text-faint" : "text-ink"}`}
    >
      {icon && <span className={`w-4 shrink-0 text-center text-[13px] ${on ? "text-accent" : "text-soft"}`}>{icon}</span>}
      <span className="truncate">{label}</span>
      {count !== undefined && count !== 0 && (
        <span className="ml-auto text-xs text-faint tabular-nums">{count}</span>
      )}
    </button>
  );
}
