import { useState } from "react";
import type { Doc, List } from "../core";
import { t } from "../locales";
import type { Chosen, Named } from "../views";
import mark from "../assets/tisty.png";

interface Props {
  lists: List[];
  docs: Doc[];
  counts: Record<string, number>;
  chosen: Chosen;
  /// A newer Tisty exists. A dot and nothing else: it is worth noticing once,
  /// never worth interrupting for.
  ready: boolean;
  onChoose: (chosen: Chosen) => void;
  onNewDoc: () => void;
}

const NAMED: { key: Named; icon: string }[] = [
  { key: "search", icon: "⌕" },
  { key: "tasks", icon: "☀" },
  { key: "tags", icon: "◈" },
  { key: "lists", icon: "▤" },
  { key: "archive", icon: "▣" },
];

export default function Sidebar({ lists, docs, counts, chosen, ready, onChoose, onNewDoc }: Props) {
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
            onClick={() => setOpenDocs((open) => !open)}
            className="flex flex-1 items-center gap-1.5 px-2.5 text-[11px] font-semibold tracking-[0.06em] text-faint uppercase"
          >
            <span className={`text-[9px] transition-transform ${openDocs ? "" : "-rotate-90"}`}>
              ▼
            </span>
            {t("docs")}
            <span className="ml-auto text-[11px] font-normal">{docs.length || ""}</span>
          </button>
          <button
            type="button"
            onClick={onNewDoc}
            aria-label={t("newDoc")}
            title={t("newDoc")}
            className="mr-1 grid h-5 w-5 place-items-center rounded text-[13px] text-faint hover:bg-hover hover:text-ink"
          >
            +
          </button>
        </div>

        {openDocs && (
          <nav className="flex flex-col gap-px">
            {docs.map((doc) => (
              <Entry
                key={doc.id}
                icon="▸"
                label={doc.title || t("untitledDoc")}
                on={chosen.doc === doc.id}
                onClick={() => onChoose({ named: "docs", doc: doc.id })}
              />
            ))}
          </nav>
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
          {/* The mark itself: what is behind it is what Tisty is, its version
              and its licence. A glyph would have to stand for that; this is it. */}
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
