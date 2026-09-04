import { useState } from "react";
import type { Filed, Folded, List, Papers } from "../core";
import { t } from "../locales";
import type { Chosen, Named } from "../views";
import Glyph from "./Glyph";
import Tree from "./Tree";

interface Props {
  lists: List[];
  papers: Papers;
  counts: Record<string, number>;
  chosen: Chosen;
  waiting?: string;
  onChoose: (chosen: Chosen) => void;

  here?: string | null;
  onHere: (folder?: string) => void;
  onMove: (folder: string, parent?: string, before?: string) => void;
  onFile: (doc: string, folder?: string, before?: string) => void;
  onPage?: (doc: string, pageOf: string) => void;
  onFolderMenu: (folder: Folded, at: { x: number; y: number }) => void;
  onDocMenu: (doc: Filed, at: { x: number; y: number }) => void;
  onDocsMenu: (at: { x: number; y: number }) => void;
  onHereMenu: (at: { x: number; y: number }) => void;
}

const NAMED: { key: Named; icon: string }[] = [
  { key: "search", icon: "search" },
  { key: "tasks", icon: "sun" },
  { key: "quadrants", icon: "grid" },
  { key: "lists", icon: "rows" },
  { key: "tags", icon: "tag" },
  { key: "archive", icon: "archive" },
];

export default function Sidebar({
  lists,
  papers,
  counts,
  chosen,
  waiting,
  onChoose,
  here,
  onHere,
  onMove,
  onFile,
  onPage,
  onFolderMenu,
  onDocMenu,
  onDocsMenu,
  onHereMenu,
}: Props) {
  const [openDocs, setOpenDocs] = useState(true);

  return (
    <aside className="flex flex-col overflow-hidden">
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
            <span className="ml-auto text-[11px] font-normal">
              {papers.docs.filter((one) => !one.pageOf).length || ""}
            </span>
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
            onPage={onPage}
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
          className={`grid size-7 place-items-center rounded-[7px] hover:bg-hover ${
            chosen.named === "keeping" ? "bg-active text-accent" : "text-soft"
          }`}
        >
          <Drawn>
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9c.2.62.77 1.05 1.42 1.06H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z" />
          </Drawn>
        </button>
        {waiting ? (
          <button
            type="button"
            aria-label={`${t("aboutScreen")} · ${t("updateWaiting")}`}
            onClick={() => onChoose({ named: "aboutScreen" })}
            className="ml-auto flex items-center gap-1.5 rounded-[7px] bg-accent-soft px-2 py-1 text-[11.5px] font-medium text-accent hover:bg-hover"
          >
            <span aria-hidden="true" className="size-1.5 rounded-full bg-current" />
            Tisty {waiting}
          </button>
        ) : (
          <button
            type="button"
            aria-label={t("aboutScreen")}
            title={t("aboutScreen")}
            onClick={() => onChoose({ named: "aboutScreen" })}
            className={`ml-auto grid size-7 place-items-center rounded-[7px] hover:bg-hover ${
              chosen.named === "aboutScreen" ? "bg-active text-accent" : "text-soft"
            }`}
          >
            <Drawn>
              <circle cx="12" cy="12" r="9" />
              <path d="M12 16v-4M12 8h.01" />
            </Drawn>
          </button>
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
      type="button"
      onClick={onClick}
      className={`flex items-center gap-2.5 rounded-[7px] px-2.5 py-1.5 text-left text-[13.5px] hover:bg-hover ${
        on ? "bg-active font-semibold" : ""
      } ${muted ? "text-faint" : "text-ink"}`}
    >
      {icon && (
        <span
          className={`flex w-4 shrink-0 justify-center text-[13px] ${on ? "text-accent" : "text-soft"}`}
        >
          <Glyph name={icon} />
        </span>
      )}
      <span className="truncate">{label}</span>
      {count !== undefined && count !== 0 && (
        <span className="ml-auto text-xs text-faint tabular-nums">{count}</span>
      )}
    </button>
  );
}

function Drawn({ children }: { children: React.ReactNode }) {
  return (
    <svg
      viewBox="0 0 24 24"
      aria-hidden="true"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.8}
      strokeLinecap="round"
      strokeLinejoin="round"
      className="size-[17px]"
    >
      {children}
    </svg>
  );
}
