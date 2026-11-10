import { useLayoutEffect, useRef, useState } from "react";
import { useAttended } from "../attended";
import type { Filed, Folded, Papers } from "../core";
import { fill, t } from "../locales";
import { sifted } from "../sifting";
import { shown } from "../tongues";
import type { Chosen, Lean } from "../views";
import Glyph from "./Glyph";
import { painted } from "./Hue";
import { fitting, useFades, useRoom } from "./room";
import { Leaning } from "./SideTasks";
import Tree from "./Tree";

const SQUARE = 36;

export type Revealed = { folder: string; at: number };

interface Props {
  papers: Papers;
  chosen: Chosen;
  leaning: Lean;
  onLean: (on: Lean) => void;
  onReveal: (folder?: string) => void;
  revealing?: Revealed | null;
  here?: string | null;
  acting?: string | null;
  onChoose: (chosen: Chosen) => void;
  onHere: (folder?: string) => void;
  onMove: (folder: string, parent?: string, before?: string) => void;
  onFile: (doc: string, folder?: string, before?: string) => void;
  onPage?: (doc: string, pageOf: string) => void;
  onFolderMenu: (folder: Folded, at: { x: number; y: number }) => void;
  onDocMenu: (doc: Filed, at: { x: number; y: number }) => void;
  onDocsMenu: (at: { x: number; y: number }) => void;
  onHereMenu: (at: { x: number; y: number }) => void;
}

export default function SideLibrary(props: Props) {
  const { papers, chosen, leaning, onLean, onReveal, onHere } = props;
  const many = shown(papers.docs).filter((one) => !one.pageOf).length;
  const lit = chosen.named === "docs";

  const title = (
    <>
      <button
        type="button"
        onClick={() => {
          onLean("docs");
          onHere();
        }}
        aria-current={lit ? "page" : undefined}
        className={`text-[13px] font-semibold hover:text-hue-amber ${lit ? "text-hue-amber" : "text-ink"}`}
      >
        {t("library")}
      </button>
      <span className="text-[11.5px] text-faint tabular-nums">{many || ""}</span>
    </>
  );

  const shrunk = leaning === "tasks";
  return (
    <>
      {shrunk && (
        <div className="relative shrink-0 pb-2">
          <span aria-hidden="true" className="absolute inset-x-5 top-0 h-px bg-hair" />
          <div className="flex items-center gap-2 px-5 pt-3 pb-1.5">
            {title}
            <span className="ml-auto -mr-1.5">
              <Leaning toward="docs" label={t("growLibrary")} onLean={onLean} />
            </span>
          </div>
          <Shelf papers={papers} here={props.here} onPick={onReveal} />
        </div>
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
  onFolderMenu,
  onDocMenu,
  onDocsMenu,
  onHereMenu,
  revealing,
  title,
  hidden,
}: Props & { title: React.ReactNode; hidden: boolean }) {
  const [sifting, setSifting] = useState(false);
  const [query, setQuery] = useState("");
  const pane = useFades<HTMLDivElement>();
  const scrolled = useRef(0);
  useLayoutEffect(() => {
    if (!hidden && pane.current) pane.current.scrollTop = scrolled.current;
  }, [hidden, pane]);
  const asked = sifting ? query.trim() : "";
  const seen = { ...papers, docs: shown(papers.docs, chosen.doc) };
  const left = sifted(seen, asked);
  const nothing = Boolean(asked) && !left.docs.length && !left.folders.length;
  const close = () => {
    setSifting(false);
    setQuery("");
  };

  return (
    <div
      hidden={hidden}
      className={`relative min-h-0 flex-1 flex-col ${hidden ? "hidden" : "flex"}`}
    >
      <span aria-hidden="true" className="absolute inset-x-5 top-0 h-px bg-hair" />
      <div className="flex items-center gap-2 px-5 pt-3 pb-1.5">
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
          <button
            type="button"
            onClick={(e) => {
              const box = e.currentTarget.getBoundingClientRect();
              onDocsMenu({ x: box.right - 8, y: box.bottom + 4 });
            }}
            aria-label={t("docsActions")}
            aria-haspopup="menu"
            className="grid size-6 place-items-center rounded-md text-faint hover:bg-hover hover:text-ink"
          >
            <Glyph name="plus" className="h-[13px] w-[13px]" />
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
        className="scroller fading min-h-0 flex-1 px-2.5 pb-4"
      >
        {nothing && (
          <p className="px-2.5 py-2 text-[12.5px] text-faint">{fill("siftNothing", asked)}</p>
        )}
        <div className={nothing ? "hidden" : undefined}>
          <Tree
            papers={left}
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
            onFolderMenu={onFolderMenu}
            onHereMenu={onHereMenu}
            onDocMenu={onDocMenu}
          />
        </div>
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
      className="mx-2.5 mb-1.5 flex h-[30px] shrink-0 items-center gap-2 rounded-md bg-hover px-2.5 text-faint focus-within:bg-active"
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

function Shelf({
  papers,
  here,
  onPick,
}: {
  papers: Papers;
  here?: string | null;
  onPick: (folder?: string) => void;
}) {
  const strip = useRef<HTMLDivElement>(null);
  const room = useRoom(strip, "width");
  const top = papers.folders.filter((one) => one.parent === null && !one.away);
  const fits = fitting(top.length, room, SQUARE);
  const hidden = top.length - fits;

  return (
    <div ref={strip} className="flex gap-1 overflow-hidden px-4">
      {top.slice(0, fits).map((folder) => (
        <button
          key={folder.id}
          type="button"
          onClick={() => onPick(folder.id)}
          aria-label={folder.name}
          aria-current={here === folder.id ? "true" : undefined}
          title={folder.name}
          className={`grid size-8 shrink-0 place-items-center rounded-md hover:bg-hover ${
            here === folder.id ? "bg-active" : ""
          }`}
        >
          <span
            className={`grid size-6 place-items-center rounded-md bg-current/10 ${painted(folder.color)}`}
          >
            <Glyph name={folder.icon ?? "folder"} className="h-[13px] w-[13px]" />
          </span>
        </button>
      ))}
      {hidden > 0 && (
        <button
          type="button"
          onClick={() => onPick()}
          aria-label={fill("foldersMore", String(hidden))}
          title={fill("foldersMore", String(hidden))}
          className="h-8 shrink-0 rounded-md px-2 text-[11.5px] font-semibold text-faint hover:bg-hover hover:text-ink"
        >
          +{hidden}
        </button>
      )}
      {!top.length && <p className="px-1 py-1.5 text-[11.5px] text-faint">{t("noDocsYet")}</p>}
    </div>
  );
}
