import type { Filed, Folded } from "../core";
import { fill, t, type Word } from "../locales";
import Glyph from "./Glyph";
import { painted } from "./Hue";

const counted = (many: number, one: Word, more: Word): string =>
  many === 0 ? "" : many === 1 ? t(one) : fill(more, String(many));

const asksForMenu = (e: React.KeyboardEvent) =>
  e.key === "ContextMenu" || (e.shiftKey && e.key === "F10");

const menuAt = (e: React.KeyboardEvent) => {
  const box = (e.target as HTMLElement).getBoundingClientRect();
  return { x: box.left + 24, y: box.bottom };
};

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
  const under = folder ? folders.filter((one) => one.parent === folder.id) : [];
  const inside = docs.filter(
    (one) =>
      !one.archived &&
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
        {under.length + inside.length === 0
          ? t("folderHoldsNothing")
          : [
              counted(under.length, "folderIsOne", "foldersAre"),
              counted(inside.length, "paperIsOne", "papersAre"),
            ]
              .filter(Boolean)
              .join(" · ")}
      </p>

      {under.length > 0 && (
        <>
          <h2 className="mt-5 mb-2 text-[10.5px] font-semibold tracking-[0.05em] text-faint uppercase">
            {t("foldersHere")}
          </h2>
          <ul className="grid grid-cols-[repeat(auto-fill,minmax(190px,1fr))] gap-2">
            {under.map((one) => (
              <li key={one.id}>
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
                  className="flex w-full cursor-pointer items-center gap-2.5 rounded-[10px] border border-hair bg-panel px-3 py-2.5 text-left hover:bg-hover"
                >
                  <span className={`shrink-0 ${painted(one.color)}`}>
                    <Glyph name={one.icon ?? "folder"} />
                  </span>
                  <span className="min-w-0">
                    <span className="block truncate text-[13px] font-medium text-ink">
                      {one.name}
                    </span>
                    <span className="mt-px block text-[11.5px] text-faint">
                      {one.holds ? counted(one.holds, "paperIsOne", "papersAre") : t("folderEmpty")}
                    </span>
                  </span>
                </button>
              </li>
            ))}
          </ul>
        </>
      )}

      {inside.length > 0 && (
        <>
          <h2 className="mt-5 mb-2 text-[10.5px] font-semibold tracking-[0.05em] text-faint uppercase">
            {t("papersHere")}
          </h2>
          <ul className="grid grid-cols-[repeat(auto-fill,minmax(190px,1fr))] gap-2">
            {inside.map((one) => (
              <li key={one.id}>
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
                  className="flex w-full cursor-pointer items-center gap-2.5 rounded-[10px] border border-hair bg-panel px-3 py-2.5 text-left hover:bg-hover"
                >
                  <span className="shrink-0 text-faint">
                    <Glyph name="page" />
                  </span>
                  <span className="min-w-0">
                    <span className="block truncate text-[13px] font-medium text-ink">
                      {one.title || t("untitledDoc")}
                    </span>
                    {one.gone && (
                      <span className="mt-px block text-[11.5px] text-urgent">{t("goneDoc")}</span>
                    )}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        </>
      )}
    </main>
  );
}
