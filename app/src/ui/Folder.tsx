import { useState } from "react";
import type { Filed, Folded } from "../core";
import { shortStamp } from "../format";
import { led } from "../leading";
import { fill, t, type Word } from "../locales";
import { weighed } from "../previews";
import Glyph from "./Glyph";
import { painted } from "./Hue";

const counted = (many: number, one: Word, more: Word): string =>
  many === 0 ? "" : many === 1 ? t(one) : fill(more, String(many));

const PEEK = 8;

const asksForMenu = (e: React.KeyboardEvent) =>
  e.key === "ContextMenu" || (e.shiftKey && e.key === "F10");

const menuAt = (e: React.KeyboardEvent) => {
  const box = (e.target as HTMLElement).getBoundingClientRect();
  return { x: box.left + 24, y: box.bottom };
};

const INDEX = "columns-1 gap-x-8 md:columns-2 xl:columns-3";
const ROW =
  "-ml-1.5 flex w-full cursor-pointer items-center gap-2 truncate rounded-[6px] px-1.5 py-[3px] text-left text-[12.5px] hover:bg-hover";

interface Props {
  folder: Folded | null;
  folders: Folded[];
  docs: Filed[];
  onOpen: (doc: Filed) => void;
  onHere: (folder?: string) => void;
  onMenu?: (folder: Folded, at: { x: number; y: number }) => void;
  onHereMenu?: (at: { x: number; y: number }) => void;
  onDocMenu?: (doc: Filed, at: { x: number; y: number }) => void;
}

export default function Folder({
  folder,
  folders,
  docs,
  onOpen,
  onHere,
  onMenu,
  onHereMenu,
  onDocMenu,
}: Props) {
  const [shut, setShut] = useState<Set<string>>(new Set());
  const fold = (id: string) =>
    setShut((was) => {
      const now = new Set(was);
      if (!now.delete(id)) now.add(id);
      return now;
    });

  const under = folder ? folders.filter((one) => one.parent === folder.id) : [];
  const inside = docs.filter(
    (one) =>
      !one.archived &&
      !one.pageOf &&
      (folder
        ? one.folder === folder.id
        : one.folder === null || !folders.some((at) => at.id === one.folder)),
  );
  const trail: Folded[] = [];
  for (let at = folder?.parent; at; ) {
    const up = folders.find((one) => one.id === at);
    if (!up || trail.some((one) => one.id === up.id)) break;
    trail.unshift(up);
    at = up.parent;
  }
  const ownMenu = folder
    ? onMenu && ((at: { x: number; y: number }) => onMenu(folder, at))
    : onHereMenu;

  const below = (id: string) => {
    const seen = new Set<string>();
    const left = [id];
    let many = 0;
    for (let at = left.pop(); at !== undefined; at = left.pop()) {
      if (seen.has(at)) continue;
      seen.add(at);
      many += docs.filter((one) => !one.archived && !one.pageOf && one.folder === at).length;
      for (const one of folders) if (one.parent === at) left.push(one.id);
    }
    return many;
  };

  const deeper = folder ? below(folder.id) - inside.length : 0;
  const missing = inside.filter((one) => one.gone).length;
  const said = [
    counted(under.length, "folderIsOne", "foldersAre"),
    deeper > 0
      ? counted(inside.length + deeper, "paperInAllIsOne", "papersInAll")
      : counted(inside.length, "paperIsOne", "papersAre"),
    deeper > 0 ? counted(inside.length, "paperRightHereIsOne", "papersRightHere") : "",
    counted(missing, "paperMissingIsOne", "papersMissing"),
  ]
    .filter(Boolean)
    .join(" · ");

  const pages = (id: string) => docs.filter((one) => one.pageOf === id).length;

  const paper = (one: Filed) => {
    const worn = led(one.title || t("untitledDoc"));
    return (
      <li key={one.id} className="break-inside-avoid">
        <button
          type="button"
          onClick={() => onOpen(one)}
          aria-haspopup={onDocMenu ? "menu" : undefined}
          aria-keyshortcuts={onDocMenu ? "Shift+F10" : undefined}
          onContextMenu={(e) => {
            if (!onDocMenu) return;
            e.preventDefault();
            onDocMenu(one, { x: e.clientX, y: e.clientY });
          }}
          onKeyDown={(e) => {
            if (!onDocMenu || !asksForMenu(e)) return;
            e.preventDefault();
            onDocMenu(one, menuAt(e));
          }}
          className={ROW}
        >
          <span className="shrink-0 text-faint">
            {worn.mark ? (
              <span className="text-[13px] leading-none">{worn.mark}</span>
            ) : (
              <Glyph name="page" className="h-3.5 w-3.5" />
            )}
          </span>
          <span className="truncate text-ink">{worn.rest}</span>
          {pages(one.id) > 0 && (
            <span className="shrink-0 text-[11.5px] text-faint">
              {pages(one.id) === 1 ? t("pageHeld") : fill("pagesHeld", String(pages(one.id)))}
            </span>
          )}
          {one.gone && <span className="shrink-0 text-[11.5px] text-urgent">{t("goneDoc")}</span>}
          {!one.gone && (one.wrote || one.bytes) && (
            <span className="ml-auto shrink-0 text-[11px] text-faint tabular-nums">
              {[one.wrote ? shortStamp(one.wrote) : "", one.bytes ? weighed(one.bytes) : ""]
                .filter(Boolean)
                .join(" · ")}
            </span>
          )}
        </button>
      </li>
    );
  };

  const named = (one: Folded) => (
    <button
      type="button"
      onClick={() => onHere(one.id)}
      aria-haspopup={onMenu ? "menu" : undefined}
      aria-keyshortcuts={onMenu ? "Shift+F10" : undefined}
      onContextMenu={(e) => {
        if (!onMenu) return;
        e.preventDefault();
        onMenu(one, { x: e.clientX, y: e.clientY });
      }}
      onKeyDown={(e) => {
        if (!onMenu || !asksForMenu(e)) return;
        e.preventDefault();
        onMenu(one, menuAt(e));
      }}
      className={ROW}
    >
      <span className={`shrink-0 ${painted(one.color)}`}>
        <Glyph name={one.icon ?? "folder"} className="h-3.5 w-3.5" />
      </span>
      <span className="truncate font-medium text-ink">{one.name}</span>
      <span className="shrink-0 text-[11.5px] text-faint">
        {one.holds ? counted(one.holds, "paperIsOne", "papersAre") : t("folderEmpty")}
      </span>
    </button>
  );

  return (
    <main className="scroller min-w-0 flex-1 px-7 pt-5 pb-8">
      {trail.length > 0 && (
        <nav aria-label={t("whereYouAre")} className="mb-2 flex flex-wrap items-center gap-1.5">
          {trail.map((up, at) => (
            <span key={up.id} className="flex items-center gap-1.5">
              {at > 0 && (
                <span aria-hidden="true" className="text-[10px] text-faint">
                  ›
                </span>
              )}
              <button
                type="button"
                onClick={() => onHere(up.id)}
                className="cursor-pointer text-[11.5px] text-faint hover:text-soft"
              >
                {up.name}
              </button>
            </span>
          ))}
        </nav>
      )}

      <h1 className="text-lg font-semibold">
        <button
          type="button"
          disabled={!ownMenu}
          aria-haspopup={ownMenu ? "menu" : undefined}
          aria-keyshortcuts={ownMenu ? "Shift+F10" : undefined}
          onContextMenu={(e) => {
            if (!ownMenu) return;
            e.preventDefault();
            ownMenu({ x: e.clientX, y: e.clientY });
          }}
          onKeyDown={(e) => {
            if (!ownMenu || !asksForMenu(e)) return;
            e.preventDefault();
            ownMenu(menuAt(e));
          }}
          className="flex items-center gap-2.5 text-left disabled:cursor-default"
        >
          <span className={folder ? painted(folder.color) : "text-faint"}>
            <Glyph
              name={folder ? (folder.icon ?? "folder") : "inbox"}
              className="h-[19px] w-[19px]"
            />
          </span>
          {folder ? folder.name : t("unfiled")}
        </button>
      </h1>
      <p className="mt-1 text-[12.5px] text-faint">
        {under.length + inside.length === 0 ? t("folderHoldsNothing") : said}
      </p>

      {under.map((one) => {
        const closed = shut.has(one.id);
        const deeper = folders.filter((at) => at.parent === one.id);
        const held = docs.filter((at) => !at.archived && !at.pageOf && at.folder === one.id);
        const shown = held.slice(0, Math.max(PEEK - deeper.length, 2));
        const left = held.length - shown.length;
        return (
          <section key={one.id} className="mt-4">
            <div className="flex items-center gap-1">
              <button
                type="button"
                onClick={() => fold(one.id)}
                aria-label={fill(closed ? "openFolder" : "closeFolder", one.name)}
                aria-expanded={!closed}
                aria-controls={`holds-${one.id}`}
                className="grid h-5 w-3 shrink-0 cursor-pointer place-items-center rounded text-[9px] text-faint hover:text-ink"
              >
                <span className={`transition-transform ${closed ? "-rotate-90" : ""}`}>▼</span>
              </button>
              {named(one)}
              <span className="h-px min-w-6 flex-1 bg-hair" />
            </div>
            {!closed && (deeper.length > 0 || held.length > 0) && (
              <ul id={`holds-${one.id}`} className={`mt-1 pl-5 ${INDEX}`}>
                {deeper.map((at) => (
                  <li key={at.id} className="break-inside-avoid">
                    {named(at)}
                  </li>
                ))}
                {shown.map(paper)}
                {left > 0 && (
                  <li className="break-inside-avoid">
                    <button
                      type="button"
                      onClick={() => onHere(one.id)}
                      className={`${ROW} text-faint hover:text-soft`}
                    >
                      {fill("moreHere", String(left))}
                    </button>
                  </li>
                )}
              </ul>
            )}
          </section>
        );
      })}

      {inside.length > 0 && (
        <>
          {under.length > 0 && (
            <h2 className="mt-5 mb-1 text-[10.5px] font-semibold tracking-[0.05em] text-faint uppercase">
              {t("papersHere")}
            </h2>
          )}
          <ul className={`mt-3 ${INDEX}`}>{inside.map(paper)}</ul>
        </>
      )}
    </main>
  );
}
