import { useCallback, useRef, useState } from "react";
import { fold, railed, sideWidth, useBreadth, useRoomFor, WIDE_WINDOW } from "../breadth";
import type { Counted, Filed, Folded, List, Papers } from "../core";
import { cameFrom } from "../guide/help";
import { useKnocking } from "../knocking";
import { t } from "../locales";
import type { Chosen, Lean } from "../views";
import Coming from "./Coming";
import { focusFinder, useFindKey } from "./finding";
import Knocking, { KnockDot } from "./Knocking";
import Mode from "./Mode";
import MoveNotice from "./MoveNotice";
import RailNotices from "./RailNotices";
import SettingsNav from "./SettingsNav";
import SideEdge, { FoldButton, useFoldKey } from "./SideEdge";
import SideLibrary, { type Revealed } from "./SideLibrary";
import SideRail from "./SideRail";
import SideTasks from "./SideTasks";
import SyncFoot from "./SyncFoot";

interface Props {
  lists: List[];
  tags?: Counted[];
  papers: Papers;
  counts: Record<string, number>;
  chosen: Chosen;
  waiting?: string;
  tried?: boolean;
  onChoose: (chosen: Chosen) => void;
  query?: string;
  onSearch?: (query: string) => void;

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
  onListMenu?: (list: List, at: { x: number; y: number }) => void;
  room?: number;
  onSync?: () => Promise<unknown> | undefined;
  onTrouble?: (text: string) => void;
}

export default function Sidebar({
  lists,
  tags = [],
  papers,
  counts,
  chosen,
  waiting,
  tried,
  onChoose,
  query = "",
  onSearch,
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
  onListMenu,
  room = 0,
  onSync,
  onTrouble,
}: Props) {
  useRoomFor(room);
  const knocked = useKnocking().waiting.length > 0;
  const told = knocked ? t("knockDot") : tried ? t("doorKnockedUnnamed") : undefined;
  const keeping = chosen.named === "keeping";
  const away = useRef<Chosen>({ named: "tasks" });
  if (!keeping) away.current = chosen;
  const breadth = railed(useBreadth(), keeping);
  const folded = breadth.folded;
  const [leaning, setLeaning] = useState<Lean>("docs");
  const [revealing, setRevealing] = useState<Revealed | null>(null);
  const reveal = (folder?: string) => {
    setLeaning("docs");
    onHere(folder);
    if (folder) setRevealing({ folder, at: Date.now() });
  };
  useFoldKey(!keeping);
  const find = useCallback(() => {
    if (folded) fold(false);
    requestAnimationFrame(focusFinder);
  }, [folded]);
  useFindKey(!keeping, find);

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
      ) : folded ? (
        <SideRail
          lists={lists}
          papers={papers}
          counts={counts}
          chosen={chosen}
          here={here}
          onChoose={onChoose}
          onReveal={reveal}
        />
      ) : (
        <>
          <SideTasks
            counts={counts}
            lists={lists}
            tags={tags}
            chosen={chosen}
            leaning={leaning}
            onLean={setLeaning}
            onChoose={onChoose}
            acting={acting}
            onListMenu={onListMenu}
            query={query}
            onSearch={onSearch ?? (() => onChoose({ named: "search" }))}
          />
          <SideLibrary
            papers={papers}
            chosen={chosen}
            leaning={leaning}
            onLean={setLeaning}
            onReveal={reveal}
            revealing={revealing}
            here={here}
            acting={acting}
            onChoose={onChoose}
            onHere={onHere}
            onMove={onMove}
            onFile={onFile}
            onPage={onPage}
            onFolderMenu={onFolderMenu}
            onDocMenu={onDocMenu}
            onDocsMenu={onDocsMenu}
            onHereMenu={onHereMenu}
          />
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
        {folded && <RailNotices />}
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
      {!keeping && (
        <SideEdge width={sideWidth(breadth, window.innerWidth >= WIDE_WINDOW)} folded={folded} />
      )}
    </aside>
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
