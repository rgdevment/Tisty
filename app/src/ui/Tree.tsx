import { useEffect, useRef, useState } from "react";
import type { Filed, Folded, Papers } from "../core";
import { led } from "../leading";
import { fill, t } from "../locales";
import {
  type Carried,
  DEEPEST,
  fits,
  type Kind,
  LOOSE,
  marked,
  type Spot,
  settled,
  speedAt,
  stepOf,
  type Where,
  zoneIn,
} from "./dragging";
import Glyph from "./Glyph";
import { painted } from "./Hue";
import { onMac } from "./WindowChrome";

const STIRS = 4;

interface Props {
  papers: Papers;
  open?: string;
  here?: string | null;
  onOpen: (doc: Filed) => void;
  onFile: (doc: string, folder?: string, before?: string) => void;
  onPage?: (doc: string, pageOf: string) => void;
  onHere?: (folder?: string) => void;
  onMove?: (folder: string, parent?: string, before?: string) => void;
  onFolderMenu?: (folder: Folded, at: { x: number; y: number }) => void;
  onHereMenu?: (at: { x: number; y: number }) => void;
  onDocMenu?: (doc: Filed, at: { x: number; y: number }) => void;
}

export default function Tree({
  papers,
  open,
  here,
  onOpen,
  onFile,
  onPage,
  onHere,
  onMove,
  onFolderMenu,
  onHereMenu,
  onDocMenu,
}: Props) {
  const [shut, setShut] = useState<Set<string>>(new Set());
  const [spread, setSpread] = useState<Set<string>>(new Set());
  const [over, setOver] = useState<string | null>(null);
  const [lifted, setLifted] = useState<{ id: string; kind: "doc" | "folder"; name: string } | null>(
    null,
  );
  const [reached, setReached] = useState<string | null>(null);
  const listed = useRef<HTMLDivElement>(null);

  const rows = () => Array.from(listed.current?.querySelectorAll<HTMLElement>("[data-row]") ?? []);

  const [carried, setCarried] = useState<Carried | null>(null);
  const holding = useRef<(Carried & { x: number; y: number; on: boolean }) | null>(null);
  const took = useRef(false);
  const pointed = useRef<{ x: number; y: number } | null>(null);
  const rolling = useRef(0);
  const rolled = useRef(0);

  const spotAt = (x: number, y: number): { spot: Spot; where: Where } | null => {
    const el = document.elementFromPoint(x, y)?.closest<HTMLElement>("[data-drop]");
    if (!el) return null;
    const box = el.getBoundingClientRect();
    const line = (el.dataset.dropLine ?? "").split("/").filter(Boolean);
    const spot: Spot = {
      id: el.dataset.drop ?? "",
      kind: (el.dataset.dropKind as Kind) ?? "doc",
      parent: el.dataset.dropParent || undefined,
      next: el.dataset.dropNext || undefined,
      holds: el.dataset.dropHolds === "yes",
      line,
      depth: line.length,
    };
    const thirds = spot.kind === "folder" || spot.holds;
    return { spot, where: zoneIn(box.top, box.height, y, thirds) };
  };

  const lineTo = (parent: string | null): string[] => {
    const all: string[] = [];
    let at = parent;
    while (at) {
      all.unshift(at);
      at = papers.folders.find((one) => one.id === at)?.parent ?? null;
      if (all.length > DEEPEST) break;
    }
    return all;
  };

  const tallOf = (id: string): number => {
    const kids = under(id);
    return 1 + Math.max(0, ...kids.map((one) => tallOf(one.id)));
  };

  const grab = (e: React.PointerEvent, id: string, kind: Kind) => {
    if (e.button !== 0 || e.ctrlKey) return;
    const tall = kind === "folder" ? tallOf(id) : 0;
    holding.current = { id, kind, tall, x: e.clientX, y: e.clientY, on: false };
    took.current = false;
    setLifted(null);
  };

  const tapped = (go: () => void) => () => {
    if (took.current) {
      took.current = false;
      return;
    }
    go();
  };

  const lit = (x: number, y: number) => {
    const held = holding.current;
    const found = spotAt(x, y);
    const takes = found && held && (found.spot.id === LOOSE || fits(held, found.spot, found.where));
    setOver(takes && found ? marked(found.spot, found.where) : null);
  };

  const roll = (now: number) => {
    const at = pointed.current;
    const sheet = listed.current?.closest<HTMLElement>(".scroller");
    const since = rolled.current ? (now - rolled.current) / 1000 : 0;
    rolled.current = now;
    if (at && sheet && since) {
      const box = sheet.getBoundingClientRect();
      const by = stepOf(speedAt(box.top, box.bottom, at.y), since);
      if (by) {
        const was = sheet.scrollTop;
        sheet.scrollTop = Math.max(0, was + by);
        if (sheet.scrollTop !== was) lit(at.x, at.y);
      }
    }
    rolling.current = requestAnimationFrame(roll);
  };

  const stopRolling = () => {
    cancelAnimationFrame(rolling.current);
    rolling.current = 0;
    rolled.current = 0;
    pointed.current = null;
  };

  useEffect(() => {
    const moved = (e: PointerEvent) => {
      const held = holding.current;
      if (!held) return;
      if (!held.on) {
        if (Math.hypot(e.clientX - held.x, e.clientY - held.y) < STIRS) return;
        held.on = true;
        setCarried({ id: held.id, kind: held.kind, tall: held.tall });
        if (!rolling.current) rolling.current = requestAnimationFrame(roll);
      }
      e.preventDefault();
      pointed.current = { x: e.clientX, y: e.clientY };
      lit(e.clientX, e.clientY);
    };

    const dropped = (e: PointerEvent) => {
      const held = holding.current;
      holding.current = null;
      stopRolling();
      if (!held?.on) return;
      took.current = true;
      setCarried(null);
      setOver(null);
      const found = spotAt(e.clientX, e.clientY);
      const move = settled(
        { id: held.id, kind: held.kind, tall: held.tall },
        found?.spot ?? null,
        found?.where ?? "in",
      );
      if (!move) return;
      if (move.pageOf) return onPage?.(move.moved, move.pageOf);
      if (move.kind === "doc") return onFile(move.moved, move.folder, move.before);
      onMove?.(move.moved, move.folder, move.before);
    };

    const quit = () => {
      if (holding.current?.on) took.current = true;
      holding.current = null;
      stopRolling();
      setCarried(null);
      setOver(null);
    };

    const let_go = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      if (!holding.current?.on) return;
      e.preventDefault();
      e.stopPropagation();
      quit();
      setLifted(null);
    };

    const swallow = (e: MouseEvent) => {
      if (!took.current) return;
      took.current = false;
      e.preventDefault();
      e.stopPropagation();
    };

    const pressed = () => {
      took.current = false;
    };

    const slid = () => {
      const at = pointed.current;
      if (at && holding.current?.on) lit(at.x, at.y);
    };

    window.addEventListener("pointerdown", pressed, true);
    window.addEventListener("pointermove", moved);
    window.addEventListener("pointerup", dropped);
    window.addEventListener("pointercancel", quit);
    window.addEventListener("keydown", let_go, true);
    window.addEventListener("click", swallow, true);
    window.addEventListener("scroll", slid, true);
    return () => {
      window.removeEventListener("pointerdown", pressed, true);
      window.removeEventListener("pointermove", moved);
      window.removeEventListener("pointerup", dropped);
      window.removeEventListener("pointercancel", quit);
      window.removeEventListener("keydown", let_go, true);
      window.removeEventListener("click", swallow, true);
      window.removeEventListener("scroll", slid, true);
    };
  });

  useEffect(() => () => cancelAnimationFrame(rolling.current), []);

  const first = papers.folders[0]?.id ?? papers.docs[0]?.id ?? "unfiled";
  const stops = (id: string) => (reached ?? first) === id;

  const walk = (from: string, by: number) => {
    const all = rows();
    const now = all.findIndex((row) => row.dataset.row === from);
    const next = all[now + by];
    if (!next) return;
    setReached(next.dataset.row ?? null);
    next.focus();
  };

  const land = (folder?: string) => {
    if (!lifted) return;
    if (lifted.kind === "folder" && lifted.id === folder) return;
    setLifted(null);
    if (lifted.kind === "doc") return onFile(lifted.id, folder);
    onMove?.(lifted.id, folder);
  };

  const typed = (
    e: React.KeyboardEvent,
    row: { id: string; kind: "doc" | "folder"; name: string },
    place?: string,
  ) => {
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      return walk(row.id, e.key === "ArrowDown" ? 1 : -1);
    }
    if (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10")) {
      e.preventDefault();
      const box = (e.target as HTMLElement).getBoundingClientRect();
      const at = { x: box.left + 24, y: box.bottom };
      if (row.kind === "doc") {
        const doc = papers.docs.find((one) => one.id === row.id);
        if (doc) onDocMenu?.(doc, at);
      } else if (row.id !== "unfiled") {
        const folder = papers.folders.find((one) => one.id === row.id);
        if (folder) onFolderMenu?.(folder, at);
      }
      return;
    }
    if (e.key === "Escape" && lifted) {
      e.preventDefault();
      return setLifted(null);
    }
    if (
      (e.ctrlKey || e.metaKey) &&
      (e.key === "x" || e.key === "X") &&
      row.id !== "unfiled" &&
      !papers.docs.some((one) => one.id === row.id && one.pageOf)
    ) {
      e.preventDefault();
      return setLifted(row);
    }
    if ((e.ctrlKey || e.metaKey) && (e.key === "v" || e.key === "V") && row.kind === "folder") {
      e.preventDefault();
      return land(place);
    }
    if (row.kind === "doc" && pagesOf(row.id).length > 0) {
      if (e.key === "ArrowRight" && !spread.has(row.id)) {
        e.preventDefault();
        return unfold(row.id);
      }
      if (e.key === "ArrowLeft" && spread.has(row.id)) {
        e.preventDefault();
        return unfold(row.id);
      }
    }
    if (row.kind === "folder" && row.id !== "unfiled") {
      if (e.key === "ArrowRight" && shut.has(row.id)) {
        e.preventDefault();
        return fold(row.id);
      }
      if (e.key === "ArrowLeft" && !shut.has(row.id)) {
        e.preventDefault();
        return fold(row.id);
      }
    }
  };

  const ICON = 18;
  const STEP = 15;
  const SPINE = 8 + ICON / 2;

  const shortcuts = (kind: "doc" | "folder" | "page") =>
    kind === "page"
      ? "Shift+F10"
      : lifted && kind === "folder"
        ? "Control+V Control+X Shift+F10"
        : "Control+X Shift+F10";

  const fold = (id: string) => {
    if (!shut.has(id)) setReached(id);
    setShut((were) => turned(were, id));
  };

  // A document arrives shut: its pages are inside it, not a level of the tree standing open.
  const unfold = (id: string) => {
    if (!spread.has(id)) setReached(id);
    setSpread((were) => turned(were, id));
  };

  const turned = (were: Set<string>, id: string) => {
    const now = new Set(were);
    if (now.has(id)) now.delete(id);
    else now.add(id);
    return now;
  };

  const under = (parent: string | null) => papers.folders.filter((one) => one.parent === parent);

  const inside = (folder: string | null) =>
    papers.docs.filter((one) => !one.archived && !one.pageOf && one.folder === folder);

  const pagesOf = (id: string) => papers.docs.filter((one) => one.pageOf === id);

  const away = papers.docs.filter((one) => one.archived && !one.pageOf);

  const takesPages = (doc: Filed) => !doc.pageOf && !doc.archived && Boolean(onPage);

  const nextOf = <T extends { id: string }>(all: T[], id: string) => {
    const at = all.findIndex((one) => one.id === id);
    return at < 0 ? undefined : all[at + 1]?.id;
  };

  const paper = (doc: Filed, depth: number, page = false) => {
    const name = doc.title || t("untitledDoc");
    const worn = page ? { mark: null, rest: name } : led(name);
    const pages = pagesOf(doc.id);
    const closed = !spread.has(doc.id) && !pages.some((one) => one.file === open);
    const mark = worn.mark ? (
      <span className="text-[12px] leading-none">{worn.mark}</span>
    ) : (
      <Glyph
        name={page ? "alignleft" : "page"}
        className={page ? "h-[11px] w-[11px] opacity-70" : "h-[13px] w-[13px]"}
      />
    );
    return (
      <li key={doc.id} className="relative">
        <div
          data-drop={page || doc.archived ? undefined : doc.id}
          data-drop-kind="doc"
          data-drop-parent={doc.folder ?? ""}
          data-drop-next={nextOf(inside(doc.folder ?? null), doc.id) ?? ""}
          data-drop-holds={takesPages(doc) ? "yes" : "no"}
          data-drop-line={lineTo(doc.folder ?? null).join("/")}
          className={`group/row relative flex items-center rounded-md focus-within:bg-hover ${
            over === doc.id ? "bg-accent-soft" : ""
          }${
            over === `${doc.id}:before`
              ? " before:absolute before:inset-x-0 before:-top-px before:h-0.5 before:rounded-full before:bg-accent"
              : ""
          }${
            over === `${doc.id}:after`
              ? " after:absolute after:inset-x-0 after:-bottom-px after:h-0.5 after:rounded-full after:bg-accent"
              : ""
          }`}
          onContextMenu={(e) => {
            if (!onDocMenu) return;
            e.preventDefault();
            onDocMenu(doc, { x: e.clientX, y: e.clientY });
          }}
        >
          {pages.length > 0 ? (
            <Grip
              at={8 + depth * STEP}
              open={!closed}
              label={fill(closed ? "showPages" : "hidePages", name)}
              controls={`pages-${doc.id}`}
              onPress={() => unfold(doc.id)}
            >
              {mark}
            </Grip>
          ) : (
            <span
              aria-hidden="true"
              className="grid h-5 w-[18px] shrink-0 place-items-center text-faint"
              style={{ marginLeft: `${8 + depth * STEP}px` }}
            >
              {mark}
            </span>
          )}
          <button
            type="button"
            onPointerDown={(e) => !page && grab(e, doc.id, "doc")}
            data-row={doc.id}
            tabIndex={stops(doc.id) ? 0 : -1}
            onFocus={() => setReached(doc.id)}
            onKeyDown={(e) => typed(e, { id: doc.id, kind: "doc", name })}
            aria-keyshortcuts={shortcuts(page ? "page" : "doc")}
            onClick={tapped(() => onOpen(doc))}
            aria-label={lifted?.id === doc.id ? fill("liftedIs", name) : name}
            aria-current={open === doc.file ? "true" : undefined}
            className={`flex min-w-0 flex-1 items-center gap-1.5 rounded-md py-1 pl-1.5 pr-2 text-left text-[13px] ${
              lifted?.id === doc.id ? "ring-1 ring-accent " : ""
            }${carried?.id === doc.id ? "opacity-45 " : ""}${doc.archived ? "opacity-55 " : ""}${
              open === doc.file
                ? "bg-active text-ink"
                : `${page ? "text-faint" : "text-soft"} hover:bg-hover`
            }`}
          >
            <span className="truncate">{worn.rest}</span>
            {doc.gone && (
              <span title={t("goneDoc")} className="shrink-0 text-[9px] text-urgent">
                ⚠
              </span>
            )}
            {pages.length > 0 && (
              <span className="ml-auto shrink-0 pl-2 text-[11px] text-faint">
                {pages.length === 1 ? t("pageHeld") : fill("pagesHeld", String(pages.length))}
              </span>
            )}
          </button>
        </div>
        {pages.length > 0 && !closed && (
          <span
            aria-hidden="true"
            className="absolute bottom-1 w-px bg-hair"
            style={{ left: `${SPINE + depth * STEP}px`, top: "26px" }}
          />
        )}
        {pages.length > 0 && !closed && (
          <ul id={`pages-${doc.id}`}>{pages.map((one) => paper(one, depth + 1, true))}</ul>
        )}
      </li>
    );
  };

  const branch = (folder: Folded, depth: number) => {
    const closed = shut.has(folder.id);
    const kids = under(folder.id);
    const papersIn = inside(folder.id);
    return (
      <li key={folder.id} className="relative">
        <div
          data-drop={folder.id}
          data-drop-kind="folder"
          data-drop-parent={folder.parent ?? ""}
          data-drop-next={nextOf(under(folder.parent ?? null), folder.id) ?? ""}
          data-drop-holds="yes"
          data-drop-line={lineTo(folder.parent ?? null).join("/")}
          className={`relative rounded-md ${over === folder.id ? "bg-accent-soft" : ""}${
            over === `${folder.id}:before`
              ? " before:absolute before:inset-x-0 before:-top-px before:h-0.5 before:rounded-full before:bg-accent"
              : ""
          }${
            over === `${folder.id}:after`
              ? " after:absolute after:inset-x-0 after:-bottom-px after:h-0.5 after:rounded-full after:bg-accent"
              : ""
          }`}
        >
          <div
            className="group/row flex items-center rounded-md"
            onContextMenu={(e) => {
              if (!onFolderMenu) return;
              e.preventDefault();
              onFolderMenu(folder, { x: e.clientX, y: e.clientY });
            }}
          >
            <Grip
              at={8 + depth * STEP}
              open={!closed}
              label={fill(closed ? "openFolder" : "closeFolder", folder.name)}
              controls={`holds-${folder.id}`}
              onPress={() => fold(folder.id)}
            >
              <span className={`flex items-center ${painted(folder.color)}`}>
                <Glyph name={folder.icon ?? "folder"} className="h-[15px] w-[15px]" />
              </span>
            </Grip>
            <button
              type="button"
              data-row={folder.id}
              tabIndex={stops(folder.id) ? 0 : -1}
              onFocus={() => setReached(folder.id)}
              onKeyDown={(e) =>
                typed(e, { id: folder.id, kind: "folder", name: folder.name }, folder.id)
              }
              aria-keyshortcuts={shortcuts("folder")}
              onPointerDown={(e) => grab(e, folder.id, "folder")}
              onClick={tapped(() => onHere?.(folder.id))}
              aria-label={lifted?.id === folder.id ? fill("liftedIs", folder.name) : folder.name}
              aria-current={here === folder.id ? "true" : undefined}
              className={`flex min-w-0 flex-1 items-center gap-1.5 py-1 pl-1.5 text-left text-[13px] ${
                here === folder.id ? "text-ink" : "text-soft"
              }`}
            >
              <span className="truncate">{folder.name}</span>
              <span className="ml-auto pr-1 text-[11px] text-faint opacity-0 transition-opacity group-hover/row:opacity-100 motion-reduce:transition-none">
                {folder.holds || ""}
              </span>
            </button>
          </div>
        </div>
        {!closed && (
          <span
            aria-hidden="true"
            className="absolute bottom-1 w-px bg-hair"
            style={{ left: `${SPINE + depth * STEP}px`, top: "26px" }}
          />
        )}
        {!closed && (
          <ul id={`holds-${folder.id}`}>
            {kids.flatMap((child) => [branch(child, depth + 1)])}
            {papersIn.flatMap((doc) => [paper(doc, depth + 1)])}
            {!kids.length && !papersIn.length && (
              <li
                className="py-0.5 text-[11px] text-faint italic"
                style={{ paddingLeft: `${8 + (depth + 1) * STEP + ICON}px` }}
              >
                {t("folderEmpty")}
              </li>
            )}
          </ul>
        )}
      </li>
    );
  };

  const loose = papers.docs.filter(
    (one) =>
      !one.archived &&
      !one.pageOf &&
      (one.folder === null || !papers.folders.some((at) => at.id === one.folder)),
  );

  const tree = (
    <ul aria-label={t("docs")} className="flex flex-col gap-px">
      {papers.folders.length === 0 && papers.docs.length === 0 && (
        <li className="px-2.5 py-2 text-[12px] text-faint">{t("noDocsYet")}</li>
      )}
      {lifted && (
        <li
          role="status"
          className="mx-1 mb-1 rounded-md bg-accent-soft px-2 py-1 text-[11.5px] text-accent"
        >
          {fill("liftedHint", lifted.name, onMac ? "⌘" : "Ctrl+")}
        </li>
      )}
      {under(null).flatMap((folder) => [branch(folder, 0)])}

      <li className="relative">
        <div
          data-drop={LOOSE}
          data-drop-kind="folder"
          data-drop-holds="yes"
          className={`group/row mt-1 flex items-center rounded-md ${
            over === LOOSE ? "bg-accent-soft" : ""
          }`}
        >
          <Grip
            at={8}
            open={!shut.has("unfiled")}
            label={fill(shut.has("unfiled") ? "openFolder" : "closeFolder", t("unfiled"))}
            controls="holds-unfiled"
            onPress={() => fold("unfiled")}
          >
            <Glyph name="inbox" className="h-[15px] w-[15px]" />
          </Grip>
          <button
            type="button"
            onContextMenu={(e) => {
              if (!onHereMenu) return;
              e.preventDefault();
              onHereMenu({ x: e.clientX, y: e.clientY });
            }}
            data-row="unfiled"
            tabIndex={stops("unfiled") ? 0 : -1}
            onFocus={() => setReached("unfiled")}
            onKeyDown={(e) => typed(e, { id: "unfiled", kind: "folder", name: t("unfiled") })}
            aria-keyshortcuts={shortcuts("folder")}
            onClick={() => onHere?.(undefined)}
            aria-label={t("unfiled")}
            aria-current={here === null ? "true" : undefined}
            className={`flex min-w-0 flex-1 items-center gap-1.5 py-1 pr-2 pl-1.5 text-left text-[13px] ${
              here === null ? "text-ink" : "text-faint"
            }`}
          >
            <span className="truncate">{t("unfiled")}</span>
            <span className="ml-auto text-[11px] opacity-0 transition-opacity group-hover/row:opacity-100 motion-reduce:transition-none">
              {loose.length || ""}
            </span>
          </button>
        </div>
        {!shut.has("unfiled") && (
          <span
            aria-hidden="true"
            className="absolute bottom-1 w-px bg-hair"
            style={{ left: `${SPINE}px`, top: "30px" }}
          />
        )}
        {!shut.has("unfiled") && <ul id="holds-unfiled">{loose.map((doc) => paper(doc, 1))}</ul>}
      </li>
    </ul>
  );

  const kept = away.length > 0 && (
    <div className="mt-2">
      <button
        type="button"
        onContextMenu={(e) => {
          if (!onHereMenu) return;
          e.preventDefault();
          onHereMenu({ x: e.clientX, y: e.clientY });
        }}
        onClick={() => fold("away")}
        aria-expanded={!shut.has("away")}
        aria-label={t("archived")}
        className="flex w-full items-center gap-1.5 px-2.5 py-1 text-[11px] font-semibold tracking-[0.06em] text-faint uppercase"
      >
        <span
          aria-hidden
          className={`text-[9px] transition-transform ${shut.has("away") ? "-rotate-90" : ""}`}
        >
          ▼
        </span>
        {t("archived")}
        <span className="ml-auto text-[11px] font-normal">{away.length}</span>
      </button>
      {!shut.has("away") && (
        <ul aria-label={t("archived")} className="flex flex-col gap-px">
          {away.map((doc) => paper(doc, 0))}
        </ul>
      )}
    </div>
  );

  return (
    <div ref={listed} className={carried ? "carrying select-none" : undefined}>
      {tree}
      {kept}
    </div>
  );
}

/// The mark is the button: the icon gives way to the chevron under the pointer, so nothing appears
/// out of thin air and every row keeps one column before the name.
function Grip({
  at,
  open,
  label,
  controls,
  onPress,
  children,
}: {
  at: number;
  open: boolean;
  label: string;
  controls: string;
  onPress: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onPress}
      aria-label={label}
      aria-expanded={open}
      aria-controls={controls}
      style={{ marginLeft: `${at}px` }}
      className="grid h-5 w-[18px] shrink-0 place-items-center rounded text-faint hover:text-ink"
    >
      <span className="col-start-1 row-start-1 flex items-center transition-opacity group-hover/row:opacity-0 group-focus-within/row:opacity-0 motion-reduce:transition-none">
        {children}
      </span>
      <Glyph
        name="chevron"
        className={`col-start-1 row-start-1 h-[13px] w-[13px] opacity-0 transition-[opacity,transform] group-hover/row:opacity-100 group-focus-within/row:opacity-100 motion-reduce:transition-none ${
          open ? "" : "-rotate-90"
        }`}
      />
    </button>
  );
}
