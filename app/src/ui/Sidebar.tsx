import { useRef, useState } from "react";
import { fold, railed, sideWidth, useBreadth, WIDE_WINDOW } from "../breadth";
import type { Filed, Folded, List, Papers } from "../core";
import { cameFrom } from "../guide/help";
import { useKnocking } from "../knocking";
import { t } from "../locales";
import { shown } from "../tongues";
import type { Chosen, Named } from "../views";
import Coming from "./Coming";
import Glyph from "./Glyph";
import Knocking, { KnockDot } from "./Knocking";
import Mode from "./Mode";
import MoveNotice from "./MoveNotice";
import SettingsNav from "./SettingsNav";
import SideEdge, { FoldButton, useFoldKey } from "./SideEdge";
import SyncFoot from "./SyncFoot";
import Tree from "./Tree";

interface Props {
  lists: List[];
  papers: Papers;
  counts: Record<string, number>;
  chosen: Chosen;
  waiting?: string;
  tried?: boolean;
  onChoose: (chosen: Chosen) => void;

  here?: string | null;
  acting?: string | null;
  onHere: (folder?: string) => void;
  onMove: (folder: string, parent?: string, before?: string) => void;
  onFile: (doc: string, folder?: string, before?: string) => void;
  onPage?: (doc: string, pageOf: string) => void;
  onFolderMenu: (folder: Folded, at: { x: number; y: number }) => void;
  onDocMenu: (doc: Filed, at: { x: number; y: number }) => void;
  onDocsMenu: (at: { x: number; y: number }) => void;
  onHereMenu: (at: { x: number; y: number }) => void;
  onSync?: () => Promise<unknown> | undefined;
  onTrouble?: (text: string) => void;
}

const NAMED: { key: Named; icon: string }[] = [
  { key: "search", icon: "search" },
  { key: "tasks", icon: "sun" },
  { key: "quadrants", icon: "grid" },
  { key: "spread", icon: "calendar" },
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
  tried,
  onChoose,
  here,
  acting,
  onHere,
  onMove,
  onFile,
  onPage,
  onFolderMenu,
  onDocMenu,
  onDocsMenu,
  onHereMenu,
  onSync,
  onTrouble,
}: Props) {
  const [openDocs, setOpenDocs] = useState(true);
  const knocked = useKnocking().waiting.length > 0;
  const told = knocked ? t("knockDot") : tried ? t("doorKnockedUnnamed") : undefined;
  const keeping = chosen.named === "keeping";
  const away = useRef<Chosen>({ named: "tasks" });
  if (!keeping) away.current = chosen;
  const breadth = railed(useBreadth(), keeping);
  const folded = breadth.folded;
  useFoldKey();

  return (
    <aside className="relative flex flex-col overflow-hidden">
      <div
        data-tauri-drag-region
        className={`flex h-9 shrink-0 items-center ${folded ? "" : "justify-end px-2.5"}`}
      >
        {!folded && !keeping && <FoldButton folded={false} />}
      </div>
      {keeping ? (
        <div className="scroller flex-1 px-2.5 pb-4">
          <SettingsNav
            tab={chosen.tab ?? "general"}
            waiting={waiting}
            onTab={(tab) => onChoose({ named: "keeping", tab })}
            onBack={() => onChoose(away.current)}
          />
        </div>
      ) : (
        <>
          <div className={folded ? "pb-1.5" : "px-2.5 pb-1.5"}>
            <nav className={`flex flex-col ${folded ? "items-center gap-1" : "gap-px"}`}>
              {folded && <FoldButton folded />}
              {NAMED.map(({ key, icon }) => (
                <Entry
                  key={key}
                  folded={folded}
                  icon={icon}
                  label={t(key)}
                  count={key === "lists" ? lists.length || undefined : counts[key]}
                  on={!chosen.list && !chosen.tags?.length && chosen.named === key}
                  onClick={() => onChoose({ named: key })}
                />
              ))}
            </nav>
          </div>

          {folded ? (
            <div className="flex flex-1 flex-col items-center gap-1 pt-1">
              <span aria-hidden="true" className="mb-1.5 h-px w-6 bg-hair" />
              <Entry
                folded
                icon="folder"
                label={t("docs")}
                count={shown(papers.docs).filter((one) => !one.pageOf).length || undefined}
                on={chosen.named === "docs"}
                onClick={() => fold(false)}
              />
            </div>
          ) : (
            <div className="scroller flex-1 px-2.5 pb-4">
              <div className="mx-1 mt-3 mb-1 h-px bg-hair" />
              <div className="pinned-head sticky top-0 z-20 flex h-[var(--tisty-docs-head)] items-center bg-rail shadow-[0_1px_0_var(--tisty-hair)]">
                <button
                  type="button"
                  onClick={() => setOpenDocs((open) => !open)}
                  aria-expanded={openDocs}
                  aria-label={t("docs")}
                  className="flex flex-1 items-center gap-1.5 px-2.5 text-[11.5px] font-semibold tracking-[0.06em] text-faint uppercase"
                >
                  {!openDocs && <Glyph name="chevron" className="h-3 w-3 shrink-0 -rotate-90" />}
                  {t("docs")}
                  <span className="ml-auto text-[11.5px] font-normal">
                    {shown(papers.docs).filter((one) => !one.pageOf).length || ""}
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
                  className="mr-1 grid h-5 w-5 place-items-center rounded-md text-[13px] text-faint hover:bg-hover hover:text-ink"
                >
                  +
                </button>
              </div>

              {openDocs && (
                <Tree
                  papers={{ ...papers, docs: shown(papers.docs, chosen.doc) }}
                  open={chosen.doc}
                  here={here}
                  acting={acting}
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
          )}
        </>
      )}

      {!folded && (
        <>
          <Knocking />
          <MoveNotice
            onOpen={() =>
              onChoose({ named: "keeping", tab: "sync", move: "pick", movedAt: Date.now() })
            }
          />
          <Coming
            onOpen={(move) =>
              onChoose({ named: "keeping", tab: "sync", move, movedAt: Date.now() })
            }
          />
        </>
      )}
      <div
        className={`flex shrink-0 border-t border-hair py-2 ${
          folded ? "flex-col items-center gap-1" : "items-center px-2.5"
        }`}
      >
        <button
          type="button"
          aria-label={told ? `${t("keeping")} · ${told}` : t("keeping")}
          title={t("keeping")}
          onClick={() => onChoose(keeping ? chosen : { named: "keeping" })}
          className={`relative grid size-7 place-items-center rounded-[10px] hover:bg-hover ${
            chosen.named === "keeping" ? "bg-active text-accent" : "text-soft"
          }`}
        >
          <Drawn>
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9c.2.62.77 1.05 1.42 1.06H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z" />
          </Drawn>
          <KnockDot />
          {tried && !knocked && (
            <span
              role="img"
              aria-label={t("doorKnockedUnnamed")}
              className="absolute top-1 right-1 h-[7px] w-[7px] rounded-full bg-hue-blue"
            />
          )}
        </button>
        {onSync ? (
          <SyncFoot
            narrow={folded}
            onSync={onSync}
            onTrouble={onTrouble}
            onOpen={() => onChoose({ named: "keeping", tab: "sync", movedAt: Date.now() })}
            fallback={
              folded ? null : (
                <Mode
                  onOpen={() => onChoose({ named: "keeping", tab: "sync", movedAt: Date.now() })}
                />
              )
            }
          />
        ) : (
          !folded && (
            <Mode onOpen={() => onChoose({ named: "keeping", tab: "sync", movedAt: Date.now() })} />
          )
        )}
        <button
          type="button"
          aria-label={t("help")}
          title={`${t("help")} · F1`}
          onClick={() => onChoose({ named: "help", from: cameFrom(chosen) })}
          className={`grid size-7 place-items-center rounded-[10px] hover:bg-hover ${folded ? "" : "ml-auto"} ${
            chosen.named === "help" ? "bg-active text-accent" : "text-soft"
          }`}
        >
          <Drawn>
            <circle cx="12" cy="12" r="9" />
            <path d="M9.1 9a3 3 0 0 1 5.8 1c0 2-3 3-3 3M12 17h.01" />
          </Drawn>
        </button>
        {waiting && (
          <button
            type="button"
            aria-label={`${t("aboutScreen")} · ${t("updateWaiting")}`}
            title={folded ? `Tisty ${waiting}` : undefined}
            onClick={() => onChoose({ named: "keeping", tab: "about" })}
            className={`flex items-center gap-1.5 rounded-[10px] bg-accent-soft font-medium text-accent hover:bg-hover ${
              folded ? "size-7 justify-center" : "px-2 py-1 text-[11.5px]"
            }`}
          >
            <span aria-hidden="true" className="size-1.5 rounded-full bg-current" />
            {!folded && `Tisty ${waiting}`}
          </button>
        )}
      </div>
      {!keeping && <SideEdge width={sideWidth(breadth, window.innerWidth >= WIDE_WINDOW)} />}
    </aside>
  );
}

interface EntryProps {
  folded?: boolean;
  icon?: string;
  label: string;
  count?: number | string;
  on?: boolean;
  muted?: boolean;
  onClick?: () => void;
}

function Entry({ folded, icon, label, count, on, muted, onClick }: EntryProps) {
  if (folded) {
    const said = count ? `${label} · ${count}` : label;
    return (
      <button
        type="button"
        onClick={onClick}
        aria-label={said}
        aria-current={on ? "page" : undefined}
        title={said}
        className={`grid size-9 place-items-center rounded-[10px] text-[13px] transition-colors duration-150 hover:bg-hover ${
          on ? "bg-active text-accent" : muted ? "text-faint" : "text-soft hover:text-ink"
        }`}
      >
        {icon && <Glyph name={icon} />}
      </button>
    );
  }
  return (
    <button
      type="button"
      onClick={onClick}
      className={`flex items-center gap-2.5 rounded-[10px] px-2.5 py-1.5 text-left text-[13px] hover:bg-hover ${
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
        <span className="ml-auto text-[11.5px] text-faint tabular-nums">{count}</span>
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
