import { useState } from "react";
import type { Folded, List, Papers, Filed } from "../core";
import Tree from "./Tree";
import { t } from "../locales";
import type { Chosen, Named } from "../views";
import mark from "../assets/tisty.png";

interface Props {
  lists: List[];
  papers: Papers;
  counts: Record<string, number>;
  chosen: Chosen;
  ready: boolean;
  onChoose: (chosen: Chosen) => void;

  here?: string | null;
  onHere: (folder?: string) => void;
  onMove: (folder: string, parent?: string) => void;
  onFile: (doc: string, folder?: string) => void;
  onFolderMenu: (folder: Folded, at: { x: number; y: number }) => void;
  onDocMenu: (doc: Filed, at: { x: number; y: number }) => void;
  onDocsMenu: (at: { x: number; y: number }) => void;
  onHereMenu: (at: { x: number; y: number }) => void;
}

const NAMED: { key: Named; icon: string }[] = [
  { key: "search", icon: "⌕" },
  { key: "tasks", icon: "☀" },
  { key: "tags", icon: "◈" },
  { key: "lists", icon: "▤" },
  { key: "archive", icon: "▣" },
];

export default function Sidebar({
  lists,
  papers,
  counts,
  chosen,
  ready,
  onChoose,
  here,
  onHere,
  onMove,
  onFile,
  onFolderMenu,
  onDocMenu,
  onDocsMenu,
  onHereMenu,
}: Props) {
  const [openDocs, setOpenDocs] = useState(true);

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
              count={key === "lists" ? lists.length || undefined : counts[key]}
              on={!chosen.list && !chosen.tags?.length && chosen.named === key}
              onClick={() => onChoose({ named: key })}
            />
          ))}
        </nav>
      </div>

      <div className="scroller flex-1 px-2.5 pb-4">
        <div className="mx-1 mt-3 mb-1 h-px bg-hair" />
        <div className="flex items-center pt-1 pb-1.5">
          <button
            type="button"
            onClick={() => setOpenDocs((open) => !open)}
            aria-expanded={openDocs}
            aria-label={t("docs")}
            className="flex flex-1 items-center gap-1.5 px-2.5 text-[11px] font-semibold tracking-[0.06em] text-faint uppercase"
          >
            <span
              aria-hidden
              className={`text-[9px] transition-transform ${openDocs ? "" : "-rotate-90"}`}
            >
              ▼
            </span>
            {t("docs")}
            <span className="ml-auto text-[11px] font-normal">{papers.docs.length || ""}</span>
          </button>
          <button
            type="button"
            onClick={(e) => {
              const box = e.currentTarget.getBoundingClientRect();
              onDocsMenu({ x: box.right - 8, y: box.bottom + 4 });
            }}
            aria-label={t("docsActions")}
            aria-haspopup="menu"
            className="mr-1 grid h-5 w-5 place-items-center rounded text-[14px] text-faint hover:bg-hover hover:text-ink"
          >
            +
          </button>
        </div>

        {openDocs && (
          <Tree
            papers={papers}
            open={chosen.doc}
            here={here}
            onHere={onHere}
            onMove={onMove}
            onOpen={(doc) => onChoose({ named: "docs", doc: doc.file })}
            onFile={onFile}
            onFolderMenu={onFolderMenu}
            onHereMenu={onHereMenu}
            onDocMenu={onDocMenu}
          />
        )}
      </div>

      <div className="flex shrink-0 items-center border-t border-hair px-2.5 py-2">
        <button
          type="button"
          aria-label={t("keeping")}
          title={t("keeping")}
          onClick={() => onChoose({ named: "keeping" })}
          className={`grid h-7 w-7 place-items-center rounded-[7px] text-[13px] hover:bg-hover ${
            chosen.named === "keeping" ? "bg-active text-accent" : "text-soft"
          }`}
        >
          <span aria-hidden="true">⚙</span>
        </button>
        <button
          type="button"
          aria-label={ready ? `${t("aboutScreen")} · ${t("updateWaiting")}` : t("aboutScreen")}
          title={t("aboutScreen")}
          onClick={() => onChoose({ named: "aboutScreen" })}
          className={`relative ml-auto grid h-7 w-7 place-items-center rounded-[7px] hover:bg-hover ${
            chosen.named === "aboutScreen" ? "bg-active" : ""
          }`}
        >
          {ready && (
            <span
              aria-hidden
              className="absolute top-0.5 right-0.5 h-1.5 w-1.5 rounded-full bg-accent"
            />
          )}
          <img
            src={mark}
            alt=""
            className={`h-[18px] w-[18px] rounded-[4px] ${
              chosen.named === "aboutScreen" ? "" : "opacity-70"
            }`}
          />
        </button>
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
