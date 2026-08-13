import { useRef, useState } from "react";
import type { Filed, Folded, Papers } from "../core";
import { fill, t } from "../locales";
import { drawn, useIcons } from "./Icons";

interface Props {
  papers: Papers;
  open?: string;
  here?: string | null;
  onOpen: (doc: Filed) => void;
  onFile: (doc: string, folder?: string) => void;
  onHere?: (folder?: string) => void;
  onMove?: (folder: string, parent?: string) => void;
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
  onHere,
  onMove,
  onFolderMenu,
  onHereMenu,
  onDocMenu,
}: Props) {
  const icons = useIcons();
  const [shut, setShut] = useState<Set<string>>(new Set());
  const [over, setOver] = useState<string | null>(null);
  const [lifted, setLifted] = useState<{ id: string; kind: "doc" | "folder"; name: string } | null>(
    null,
  );
  const [reached, setReached] = useState<string | null>(null);
  const listed = useRef<HTMLUListElement>(null);

  const rows = () =>
    Array.from(listed.current?.querySelectorAll<HTMLElement>("[data-row]") ?? []);

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
    if ((e.ctrlKey || e.metaKey) && (e.key === "x" || e.key === "X") && row.id !== "unfiled") {
      e.preventDefault();
      return setLifted(row);
    }
    if ((e.ctrlKey || e.metaKey) && (e.key === "v" || e.key === "V") && row.kind === "folder") {
      e.preventDefault();
      return land(place);
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

  const ICON = 20;

  const shortcuts = (kind: "doc" | "folder") =>
    lifted && kind === "folder" ? "Control+V Control+X Shift+F10" : "Control+X Shift+F10";

  const fold = (id: string) =>
    setShut((were) => {
      const now = new Set(were);
      if (now.has(id)) now.delete(id);
      else now.add(id);
      return now;
    });

  const under = (parent: string | null) => papers.folders.filter((one) => one.parent === parent);

  const inside = (folder: string | null) =>
    papers.docs.filter((one) => !one.archived && one.folder === folder);

  const away = papers.docs.filter((one) => one.archived);

  const dropOn = (folder?: string) => ({
    onDragOver: (e: React.DragEvent) => {
      e.preventDefault();
      setOver(folder ?? "unfiled");
    },
    onDragLeave: () => setOver(null),
    onDrop: (e: React.DragEvent) => {
      e.preventDefault();
      setOver(null);
      const doc = e.dataTransfer.getData("text/tisty-doc");
      if (doc) return onFile(doc, folder);
      const moved = e.dataTransfer.getData("text/tisty-folder");
      if (moved && moved !== folder) onMove?.(moved, folder);
    },
  });

  const paper = (doc: Filed, depth: number) => (
    <li
      key={doc.id}
      className="group/paper flex items-center focus-within:bg-hover"
      onContextMenu={(e) => {
        if (!onDocMenu) return;
        e.preventDefault();
        onDocMenu(doc, { x: e.clientX, y: e.clientY });
      }}
    >
      <button
        type="button"
        draggable
        data-row={doc.id}
        tabIndex={stops(doc.id) ? 0 : -1}
        onFocus={() => setReached(doc.id)}
        onKeyDown={(e) =>
          typed(e, { id: doc.id, kind: "doc", name: doc.title || t("untitledDoc") })
        }
        aria-keyshortcuts={shortcuts("doc")}
        onDragStart={(e) => e.dataTransfer.setData("text/tisty-doc", doc.id)}
        onClick={() => onOpen(doc)}
        aria-label={
          lifted?.id === doc.id
            ? fill("liftedIs", doc.title || t("untitledDoc"))
            : doc.title || t("untitledDoc")
        }
        aria-current={open === doc.file ? "true" : undefined}
        style={{ paddingLeft: `${8 + depth * 17 + ICON}px` }}
        className={`flex min-w-0 flex-1 items-center gap-1.5 rounded-md py-1 pr-2 text-left text-[12.5px] ${
          lifted?.id === doc.id ? "ring-1 ring-accent " : ""
        }${doc.archived ? "opacity-55 " : ""}${
          open === doc.file ? "bg-active text-ink" : "text-soft hover:bg-hover"
        }`}
      >
        <span className="w-3 shrink-0 text-center text-[9px] text-faint">▸</span>
        <span className="truncate">{doc.title || t("untitledDoc")}</span>
      </button>
    </li>
  );

  const branch = (folder: Folded, depth: number) => {
    const closed = shut.has(folder.id);
    return (
      <li key={folder.id}>
        <div
          {...dropOn(folder.id)}
          className={`rounded-md ${over === folder.id ? "bg-accent-soft" : ""}`}
        >
          <div
            className="group/folder flex items-center rounded-md"
            onContextMenu={(e) => {
              if (!onFolderMenu) return;
              e.preventDefault();
              onFolderMenu(folder, { x: e.clientX, y: e.clientY });
            }}
          >
            <button
              type="button"
              onClick={() => fold(folder.id)}
              aria-label={fill(closed ? "openFolder" : "closeFolder", folder.name)}
              aria-expanded={!closed}
              style={{ marginLeft: `${8 + depth * 17}px` }}
              className="grid h-5 w-3 shrink-0 place-items-center rounded text-[9px] text-faint hover:text-ink"
            >
              <span className={`transition-transform ${closed ? "-rotate-90" : ""}`}>▼</span>
            </button>
            <button
              type="button"
              draggable
              data-row={folder.id}
              tabIndex={stops(folder.id) ? 0 : -1}
              onFocus={() => setReached(folder.id)}
              onKeyDown={(e) =>
                typed(e, { id: folder.id, kind: "folder", name: folder.name }, folder.id)
              }
              aria-keyshortcuts={shortcuts("folder")}
              onDragStart={(e) => e.dataTransfer.setData("text/tisty-folder", folder.id)}
              onClick={() => {
                onHere?.(folder.id);
                fold(folder.id);
              }}
              aria-expanded={!closed}
              aria-label={lifted?.id === folder.id ? fill("liftedIs", folder.name) : folder.name}
              aria-current={here === folder.id ? "true" : undefined}
              className={`flex min-w-0 flex-1 items-center gap-1.5 py-1 pl-1.5 text-left text-[12.5px] ${
                here === folder.id ? "text-ink" : "text-soft"
              }`}
            >
              <span className="shrink-0">{drawn(icons, folder.icon ?? undefined) ?? "🗂"}</span>
              <span className="truncate">{folder.name}</span>
              <span className="ml-auto pr-1 text-[11px] text-faint">{folder.holds || ""}</span>
            </button>
          </div>
        </div>
        {!closed && (
          <ul>
            {under(folder.id).map((child) => branch(child, depth + 1))}
            {inside(folder.id).map((doc) => paper(doc, depth + 1))}
          </ul>
        )}
      </li>
    );
  };

  const loose = papers.docs.filter(
    (one) =>
      !one.archived && (one.folder === null || !papers.folders.some((at) => at.id === one.folder)),
  );

  const tree = (
    <ul ref={listed} aria-label={t("docs")} className="flex flex-col gap-px">
      {papers.folders.length === 0 && papers.docs.length === 0 && (
        <li className="px-2.5 py-2 text-[12px] text-faint">{t("noDocsYet")}</li>
      )}
      {lifted && (
        <li
          role="status"
          className="mx-1 mb-1 rounded-md bg-accent-soft px-2 py-1 text-[11.5px] text-accent"
        >
          {fill("liftedHint", lifted.name)}
        </li>
      )}
      {under(null).map((folder) => branch(folder, 0))}

      <li>
        <div
          {...dropOn(undefined)}
          className={`mt-1 rounded-md ${over === "unfiled" ? "bg-accent-soft" : ""}`}
        >
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
            onClick={() => {
              onHere?.(undefined);
              fold("unfiled");
            }}
            aria-expanded={!shut.has("unfiled")}
            aria-current={here === null ? "true" : undefined}
            className={`flex w-full items-center gap-1.5 py-1 pr-2 pl-[26px] text-left text-[12.5px] ${
              here === null ? "text-ink" : "text-faint"
            }`}
          >
            <span className="shrink-0">📥</span>
            <span className="truncate">{t("unfiled")}</span>
            <span className="ml-auto text-[11px]">{loose.length || ""}</span>
          </button>
        </div>
        {!shut.has("unfiled") && (
          <ul {...dropOn(undefined)}>{loose.map((doc) => paper(doc, 1))}</ul>
        )}
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
    <>
      {tree}
      {kept}
    </>
  );
}
