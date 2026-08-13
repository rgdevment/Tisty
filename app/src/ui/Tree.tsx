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
  onRename?: (folder: Folded) => void;
  onDrop?: (folder: Folded) => void;
  onDropDoc?: (doc: Filed) => void;
}

export default function Tree({
  papers,
  open,
  here,
  onOpen,
  onFile,
  onHere,
  onMove,
  onRename,
  onDrop,
  onDropDoc,
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

  const shortcuts = (kind: "doc" | "folder") =>
    lifted && kind === "folder" ? "Control+V Control+X" : "Control+X";

  const fold = (id: string) =>
    setShut((were) => {
      const now = new Set(were);
      if (now.has(id)) now.delete(id);
      else now.add(id);
      return now;
    });

  const under = (parent: string | null) => papers.folders.filter((one) => one.parent === parent);

  const inside = (folder: string | null) => papers.docs.filter((one) => one.folder === folder);

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
    <li key={doc.id} className="group/paper flex items-center focus-within:bg-hover">
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
        style={{ paddingLeft: `${8 + depth * 13}px` }}
        className={`flex min-w-0 flex-1 items-center gap-1.5 rounded-md py-1 pr-2 text-left text-[12.5px] ${
          lifted?.id === doc.id ? "ring-1 ring-accent " : ""
        }${open === doc.file ? "bg-active text-ink" : "text-soft hover:bg-hover"}`}
      >
        <span className="w-3 shrink-0 text-center text-[9px] text-faint">▸</span>
        <span className="truncate">{doc.title || t("untitledDoc")}</span>
      </button>
      {onDropDoc && (
        <button
          type="button"
          onClick={() => onDropDoc(doc)}
          aria-label={fill("dropDoc", doc.title || t("untitledDoc"))}
          title={t("dropDocShort")}
          tabIndex={-1}
          className="mr-1 grid h-5 w-5 shrink-0 place-items-center rounded text-[11px] text-faint opacity-0 group-hover/paper:opacity-100 group-focus-within/paper:opacity-100 hover:bg-urgent hover:text-bg focus:opacity-100"
        >
          ✕
        </button>
      )}
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
            className={`group/folder flex items-center rounded-md ${
              here === folder.id ? "bg-active text-ink" : ""
            }`}
          >
            <button
              type="button"
              onClick={() => fold(folder.id)}
              aria-label={fill(closed ? "openFolder" : "closeFolder", folder.name)}
              aria-expanded={!closed}
              style={{ marginLeft: `${8 + depth * 13}px` }}
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
              onClick={() => onHere?.(folder.id)}
              aria-label={lifted?.id === folder.id ? fill("liftedIs", folder.name) : folder.name}
              aria-current={here === folder.id ? "true" : undefined}
              className={`flex min-w-0 flex-1 items-center gap-1.5 rounded-md py-1 pl-1.5 text-left text-[12.5px] ${
                here === folder.id ? "text-ink" : "text-soft hover:bg-hover"
              }`}
            >
              <span className="shrink-0">{drawn(icons, folder.icon ?? undefined) ?? "🗂"}</span>
              <span className="truncate">{folder.name}</span>
              <span className="ml-auto pr-1 text-[11px] text-faint">{folder.holds || ""}</span>
            </button>
            {onRename && (
              <button
                type="button"
                onClick={() => onRename(folder)}
                aria-label={fill("renameFolder", folder.name)}
                title={t("rename")}
                tabIndex={-1}
                className="mr-0.5 grid h-5 w-5 shrink-0 place-items-center rounded text-[11px] text-faint opacity-0 group-hover/folder:opacity-100 group-focus-within/folder:opacity-100 hover:bg-hover hover:text-ink focus:opacity-100"
              >
                ✎
              </button>
            )}
            {onDrop && (
              <button
                type="button"
                onClick={() => onDrop(folder)}
                aria-label={fill("dropFolder", folder.name)}
                title={t("dropFolderShort")}
                tabIndex={-1}
                className="mr-1 grid h-5 w-5 shrink-0 place-items-center rounded text-[11px] text-faint opacity-0 group-hover/folder:opacity-100 group-focus-within/folder:opacity-100 hover:bg-urgent hover:text-bg focus:opacity-100"
              >
                ✕
              </button>
            )}
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

  const loose = inside(null);

  return (
    <ul ref={listed} aria-label={t("docs")} className="flex flex-col gap-px">
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
            data-row="unfiled"
            tabIndex={stops("unfiled") ? 0 : -1}
            onFocus={() => setReached("unfiled")}
            onKeyDown={(e) => typed(e, { id: "unfiled", kind: "folder", name: t("unfiled") })}
            aria-keyshortcuts={shortcuts("folder")}
            onClick={() => onHere?.(undefined)}
            aria-current={here === null ? "true" : undefined}
            className={`flex w-full items-center gap-1.5 rounded-md py-1 pr-2 pl-[26px] text-left text-[12.5px] ${
              here === null ? "bg-active text-ink" : "text-faint hover:bg-hover"
            }`}
          >
            <span className="shrink-0">📥</span>
            <span className="truncate">{t("unfiled")}</span>
            <span className="ml-auto text-[11px]">{loose.length || ""}</span>
          </button>
        </div>
        <ul {...dropOn(undefined)}>{loose.map((doc) => paper(doc, 1))}</ul>
      </li>
    </ul>
  );
}
