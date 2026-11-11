import { useMemo, useRef } from "react";
import type { Filed, Papers } from "../core";
import { brief, capped, exactly } from "../format";
import { lately } from "../lately";
import { fill, t } from "../locales";
import type { Lean } from "../views";
import { Tiles } from "./DayCounts";
import Glyph from "./Glyph";
import { painted } from "./Hue";
import { asksForMenu, menuAt } from "./menuKeys";
import { fitting, useRoom } from "./room";
import { Leaning } from "./SideTasks";

const SQUARE = 36;

interface Props {
  papers: Papers;
  many: number;
  lit: boolean;
  here?: string | null;
  acting?: string | null;
  onLean: (on: Lean) => void;
  onLibrary: () => void;
  onOpen: (doc: Filed) => void;
  onReveal: (folder?: string) => void;
  onDocMenu: (doc: Filed, at: { x: number; y: number }) => void;
}

export default function LibraryGlance({
  papers,
  many,
  lit,
  here,
  acting,
  onLean,
  onLibrary,
  onOpen,
  onReveal,
  onDocMenu,
}: Props) {
  const day = new Date().toDateString();
  const seen = useMemo(() => lately(papers, new Date(day)), [papers, day]);
  const tiles = [
    { key: "today", said: t("glanceToday"), many: seen.today, tone: "text-hue-amber" },
    { key: "week", said: t("glanceWeek"), many: seen.week },
    { key: "folders", said: t("glanceFolders"), many: seen.folders },
    {
      key: "marked",
      said: t("glanceMarked"),
      many: seen.marked,
      tone: seen.marked ? "text-urgent" : "",
    },
  ];

  return (
    <section aria-label={t("library")} className="shrink-0 px-2.5 pb-2">
      <span aria-hidden="true" className="mx-2.5 mt-2 mb-3 block h-px bg-hair" />
      <div>
        <div className="flex items-center gap-2 pl-2.5">
          <button
            type="button"
            onClick={onLibrary}
            aria-current={lit ? "page" : undefined}
            className={`text-[12.5px] font-semibold hover:text-hue-amber ${
              lit ? "text-hue-amber" : "text-faint"
            }`}
          >
            {t("library")}
          </button>
          {many ? (
            <span
              title={exactly(many)}
              className="ml-auto shrink-0 text-[11.5px] text-faint tabular-nums"
            >
              {capped(many)}
            </span>
          ) : null}
          <span className={many ? "" : "ml-auto"}>
            <Leaning toward="docs" label={t("growLibrary")} onLean={onLean} />
          </span>
        </div>
        <div className="mt-1.5">
          <Tiles tiles={tiles} />
        </div>
        {seen.recent.length > 0 && (
          <ul aria-label={t("glanceRecent")} className="mt-1.5">
            {seen.recent.map((doc) => (
              <li key={doc.id}>
                <button
                  type="button"
                  onClick={() => onOpen(doc)}
                  onContextMenu={(e) => {
                    e.preventDefault();
                    onDocMenu(doc, { x: e.clientX, y: e.clientY });
                  }}
                  onKeyDown={(e) => {
                    if (!asksForMenu(e)) return;
                    e.preventDefault();
                    onDocMenu(doc, menuAt(e));
                  }}
                  className={`flex w-full items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left text-[13px] text-soft hover:bg-hover hover:text-ink ${
                    acting === doc.id ? "bg-active" : ""
                  }`}
                >
                  <Glyph name="page" className="h-[13px] w-[13px] text-faint" />
                  <span className="truncate">{doc.title || t("untitledDoc")}</span>
                  <span className="ml-auto shrink-0 text-[11.5px] text-faint tabular-nums">
                    {brief(doc.wrote ?? "")}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        )}
        <div className="mt-1.5 px-1">
          <Shelf papers={papers} here={here} onPick={onReveal} />
        </div>
      </div>
    </section>
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
    <div ref={strip} className="flex gap-1 overflow-hidden">
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
