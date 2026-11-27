import { useLayoutEffect, useRef, useState } from "react";
import { newRuleForAll } from "../agentsSync";
import { useAttended } from "../attended";
import type { Filed, Folded, Papers, Wired } from "../core";
import { useGoing, useHiddenGoing } from "../dropLater";
import { capped, exactly } from "../format";
import type { KnownAgents } from "../knownAgents";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { landedOn } from "../settingsRows";
import { sifted } from "../sifting";
import { shown } from "../tongues";
import { usePinned } from "../usePinned";
import { HOOK_AGENTS } from "../userHooks";
import type { Chosen, Lean } from "../views";
import Glyph from "./Glyph";
import LibraryGlance from "./LibraryGlance";
import NewHookAsk from "./NewHookAsk";
import { useFades } from "./room";
import { Leaning } from "./SideTasks";
import Tree, { type useFolds } from "./Tree";

export type Revealed = { folder: string; at: number };

/// What the library does with the rows themselves: make, delete, rename and act on several at once.
export interface LibraryActs {
  newDoc: () => void;
  newTitle: string;
  dropDocs: (docs: Filed[]) => void;
  dropFolder: (folder: Folded) => void;
  rename: (folder: Folded) => void;
  moveDocs: (docs: Filed[], at: { x: number; y: number }) => void;
  awayDocs: (docs: Filed[]) => void;
}

interface Props {
  papers: Papers;
  folds?: ReturnType<typeof useFolds>;
  chosen: Chosen;
  leaning: Lean;
  onLean: (on: Lean) => void;
  revealing?: Revealed | null;
  here?: string | null;
  acting?: string | null;
  onChoose: (chosen: Chosen) => void;
  onHere: (folder?: string) => void;
  onMove: (folder: string, parent?: string, before?: string) => void;
  onFile: (doc: string, folder?: string, before?: string) => void;
  onPage?: (doc: string, pageOf: string) => void;
  onDiary?: () => void;
  onIdeas?: () => void;
  onDiaryMenu?: (at: { x: number; y: number }) => void;
  onIdeasMenu?: (at: { x: number; y: number }) => void;
  onAgentsMenu?: (at: { x: number; y: number }) => void;
  onFolderMenu: (folder: Folded, at: { x: number; y: number }) => void;
  onDocMenu: (doc: Filed, at: { x: number; y: number }) => void;
  onDocsMenu: (at: { x: number; y: number }) => void;
  onHereMenu: (at: { x: number; y: number }) => void;
  acts?: LibraryActs;
  /// The connected agents Settings already read, for the Hooks group of Agents › Global.
  agents?: readonly Wired[] | null;
  known?: KnownAgents;
  onTrouble?: (text: string) => void;
}

export default function SideLibrary(props: Props) {
  const { papers, chosen, leaning, onLean, onHere } = props;
  const many = shown(papers.docs).filter((one) => !one.pageOf && !one.away).length;
  const lit = chosen.named === "docs";
  const library = () => {
    onLean("docs");
    onHere();
  };

  const title = (
    <>
      <button
        type="button"
        onClick={library}
        aria-current={lit ? "page" : undefined}
        className={`flex items-center gap-2 text-[13px] font-semibold hover:text-hue-amber ${lit ? "text-hue-amber" : "text-ink"}`}
      >
        <Glyph name="library" className="h-4 w-4 text-hue-amber" />
        {t("library")}
      </button>
      <span title={exactly(many)} className="text-[11.5px] text-faint tabular-nums">
        {many ? capped(many) : ""}
      </span>
    </>
  );

  const shrunk = leaning === "tasks";
  return (
    <>
      {shrunk && (
        <LibraryGlance papers={papers} many={many} lit={lit} onLean={onLean} onLibrary={library} />
      )}
      <Wide {...props} title={title} hidden={shrunk} />
    </>
  );
}

function Wide({
  papers,
  chosen,
  onLean,
  here,
  acting,
  onChoose,
  onHere,
  onMove,
  onFile,
  onPage,
  onDiary,
  onIdeas,
  onDiaryMenu,
  onIdeasMenu,
  onAgentsMenu,
  onFolderMenu,
  onDocMenu,
  onDocsMenu,
  onHereMenu,
  acts,
  revealing,
  folds,
  title,
  hidden,
  agents,
  known,
  onTrouble,
}: Props & { title: React.ReactNode; hidden: boolean }) {
  const [hookAsked, setHookAsked] = useState(false);
  const newRule = (level: string) =>
    newRuleForAll(level)
      .then((made) => onChoose({ named: "docs", doc: made.id }))
      .catch((problem) => onTrouble?.(saidPlainly(problem)));
  const [sifting, setSifting] = useState(false);
  const [picked, setPicked] = useState<ReadonlySet<string>>(new Set());
  const going = useHiddenGoing();
  const handed = useGoing().flatMap((one) => (one.moves ? [one.moves] : []));
  const [query, setQuery] = useState("");
  const pinned = usePinned(papers);
  const pane = useFades<HTMLDivElement>();
  const scrolled = useRef(0);
  useLayoutEffect(() => {
    if (!hidden && pane.current) pane.current.scrollTop = scrolled.current;
  }, [hidden, pane]);
  const asked = sifting ? query.trim() : "";
  const landing = (at: string | null): string | null => {
    const moved = handed.find((one) => one.from === at);
    return moved ? landing(moved.to) : at;
  };
  const seen = {
    folders: papers.folders
      .filter((one) => !going.has(one.id))
      .map((one) => (handed.length ? { ...one, parent: landing(one.parent) } : one)),
    docs: shown(papers.docs, chosen.doc)
      .filter((one) => !going.has(one.id))
      .map((one) => (handed.length ? { ...one, folder: landing(one.folder ?? null) } : one)),
  };
  const pickedDocs = seen.docs.filter((one) => picked.has(one.id));
  const pick = (ids: string[]) => setPicked(new Set(ids));
  const dropped = (row: { id: string; kind: "doc" | "folder" }) => {
    if (!acts) return;
    if (row.kind === "folder") {
      const folder = seen.folders.find((one) => one.id === row.id);
      return folder && acts.dropFolder(folder);
    }
    const docs = picked.has(row.id) ? pickedDocs : seen.docs.filter((one) => one.id === row.id);
    pick([]);
    if (docs.length) acts.dropDocs(docs);
  };
  const left = sifted(seen, asked);
  const nothing = Boolean(asked) && !left.docs.length && !left.folders.length;
  const close = () => {
    setSifting(false);
    setQuery("");
  };

  return (
    <div hidden={hidden} className={`min-h-0 flex-1 flex-col pb-2 ${hidden ? "hidden" : "flex"}`}>
      <span aria-hidden="true" className="mx-5 mt-2 mb-3 h-px shrink-0 bg-hair" />
      <div className="flex min-h-0 flex-1 flex-col">
        <div className="flex items-center gap-2 pr-4 pb-1 pl-5">
          {title}
          <span className="ml-auto -mr-1.5 flex items-center gap-0.5">
            <button
              type="button"
              onClick={() => (sifting ? close() : setSifting(true))}
              aria-pressed={sifting}
              aria-label={t("siftDocs")}
              title={t("siftDocs")}
              className={`grid size-6 place-items-center rounded-md hover:bg-hover hover:text-ink ${
                sifting ? "bg-active text-ink" : "text-faint"
              }`}
            >
              <Glyph name="filter" className="h-[13px] w-[13px]" />
            </button>
            {acts && (
              <button
                type="button"
                onClick={acts.newDoc}
                title={acts.newTitle}
                className="flex h-6 items-center gap-1 rounded-l-md bg-accent-soft pr-1.5 pl-1.5 text-[11.5px] font-semibold text-accent hover:bg-active"
              >
                <Glyph name="plus" className="h-[11px] w-[11px]" />
                {t("newShort")}
              </button>
            )}
            <button
              type="button"
              onClick={(e) => {
                const box = e.currentTarget.getBoundingClientRect();
                onDocsMenu({ x: box.right - 8, y: box.bottom + 4 });
              }}
              aria-label={t("docsActions")}
              title={t("docsActions")}
              aria-haspopup="menu"
              className={`grid h-6 place-items-center text-faint hover:bg-hover hover:text-ink ${
                acts ? "w-5 rounded-r-md bg-accent-soft text-accent" : "w-6 rounded-md"
              }`}
            >
              <Glyph name={acts ? "chevron" : "plus"} className="h-[11px] w-[11px]" />
            </button>
            <Leaning toward="tasks" label={t("shrinkLibrary")} onLean={onLean} />
          </span>
        </div>
        {sifting && <Sift query={query} onQuery={setQuery} onClose={close} />}
        <div
          ref={pane}
          onScroll={(e) => {
            if (!hidden) scrolled.current = e.currentTarget.scrollTop;
          }}
          className="scroller fading min-h-0 flex-1 pt-1 pr-2.5 pb-2 pl-4"
        >
          {nothing && (
            <p className="px-2.5 py-2 text-[12.5px] text-faint">{fill("siftNothing", asked)}</p>
          )}
          <div className={nothing ? "hidden" : undefined}>
            <Tree
              papers={left}
              folds={folds}
              pinned={pinned?.file ?? null}
              ideas={pinned?.ideas}
              ideasMarked={pinned?.marked}
              reveal={revealing}
              unfolded={Boolean(asked)}
              open={chosen.doc}
              here={here}
              acting={acting}
              onHere={onHere}
              onMove={onMove}
              onOpen={(doc) => onChoose({ named: "docs", doc: doc.file })}
              onFile={onFile}
              onPage={onPage}
              onDiary={onDiary}
              onIdeas={onIdeas}
              onDiaryMenu={onDiaryMenu}
              onIdeasMenu={onIdeasMenu}
              onAgentsMenu={onAgentsMenu}
              onFolderMenu={onFolderMenu}
              onHereMenu={onHereMenu}
              onDocMenu={onDocMenu}
              picked={picked}
              onPicked={acts ? pick : undefined}
              onDelete={acts ? dropped : undefined}
              onRename={acts?.rename}
              agents={agents}
              known={known}
              onHooks={() => onChoose(landedOn({ page: "agents", shelf: "wired" }))}
              onNewRule={newRule}
              onNewHook={() => setHookAsked(true)}
            />
          </div>
        </div>
        {hookAsked && (
          <NewHookAsk
            connected={(agents ?? [])
              .filter((one) => one.wired && !!one.hooks)
              .map((one) => one.id)
              .filter((id) => HOOK_AGENTS.some(([known]) => known === id))}
            onMade={(made) => {
              setHookAsked(false);
              onChoose({ named: "docs", doc: made.id });
            }}
            onClose={() => setHookAsked(false)}
          />
        )}
        {acts && pickedDocs.length > 1 && (
          <PickedBar
            many={pickedDocs.length}
            onMove={(at) => acts.moveDocs(pickedDocs, at)}
            onAway={() => {
              acts.awayDocs(pickedDocs);
              pick([]);
            }}
            onDrop={() => {
              acts.dropDocs(pickedDocs);
              pick([]);
            }}
            onLetGo={() => pick([])}
          />
        )}
      </div>
    </div>
  );
}

function Sift({
  query,
  onQuery,
  onClose,
}: {
  query: string;
  onQuery: (query: string) => void;
  onClose: () => void;
}) {
  const field = useAttended<HTMLInputElement>();
  return (
    <form
      onSubmit={(e) => e.preventDefault()}
      className="mx-2.5 mt-1 mb-1 flex h-[30px] shrink-0 items-center gap-2 rounded-md bg-hover px-2.5 text-faint focus-within:bg-active"
    >
      <Glyph name="filter" className="h-[13px] w-[13px]" />
      <input
        ref={field}
        value={query}
        placeholder={t("siftDocs")}
        aria-label={t("siftDocs")}
        onChange={(e) => onQuery(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Escape") onClose();
        }}
        className="min-w-0 flex-1 bg-transparent text-[12.5px] text-ink outline-none placeholder:text-faint"
      />
      <button
        type="button"
        onClick={onClose}
        aria-label={t("siftClose")}
        className="grid size-5 place-items-center rounded-md hover:bg-active hover:text-ink"
      >
        <Glyph name="close" className="h-3 w-3" />
      </button>
    </form>
  );
}

function PickedBar({
  many,
  onMove,
  onAway,
  onDrop,
  onLetGo,
}: {
  many: number;
  onMove: (at: { x: number; y: number }) => void;
  onAway: () => void;
  onDrop: () => void;
  onLetGo: () => void;
}) {
  const act = "rounded-md bg-hover px-2 py-1 hover:bg-active";
  return (
    <div
      role="toolbar"
      aria-label={fill("pickedMany", String(many))}
      className="mx-4 mt-1 flex items-center gap-1.5 rounded-[10px] border border-hair bg-bg p-1.5 text-[11.5px] text-ink shadow-lift"
    >
      <span className="px-1 text-faint">{fill("pickedMany", String(many))}</span>
      <button
        type="button"
        className={act}
        onClick={(e) => {
          const box = e.currentTarget.getBoundingClientRect();
          onMove({ x: box.left, y: box.top - 4 });
        }}
      >
        {t("moveTo")}
      </button>
      <button type="button" className={act} onClick={onAway}>
        {t("putAway")}
      </button>
      <button type="button" className={`${act} text-urgent`} onClick={onDrop}>
        {t("deleteIt")}
      </button>
      <button
        type="button"
        onClick={onLetGo}
        aria-label={t("pickedLetGo")}
        title={t("pickedLetGo")}
        className="ml-auto grid size-5 place-items-center rounded-md text-faint hover:bg-hover hover:text-ink"
      >
        <Glyph name="close" className="h-3 w-3" />
      </button>
    </div>
  );
}
