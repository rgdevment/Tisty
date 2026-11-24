import { useState } from "react";
import {
  agentsPaint,
  agentsRoot,
  agentsStage,
  folderShown,
  holdsTistySkills,
  syncAgents,
} from "../agentsSync";
import type { Filed, Folded } from "../core";
import { countedAs, dateOf, longAgo, shortStamp } from "../format";
import { led } from "../leading";
import { fill, t } from "../locales";
import { weighed } from "../previews";
import { atRandom, type Recalled, useForgotten } from "../useForgotten";
import { useAgentsState } from "./AgentsMark";
import { FolderMark } from "./AssistantMark";
import Glyph from "./Glyph";
import { painted } from "./Hue";
import { asksForMenu, menuAt } from "./menuKeys";
import { TistySkillsEntry } from "./TistySkills";

const PEEK = 8;
const FORGOTTEN_AT_A_GLANCE = 5;

const INDEX = "columns-1 gap-x-8 md:columns-2 xl:columns-3";
const ROW =
  "-ml-1.5 flex w-full cursor-pointer items-center gap-2 truncate rounded-md px-1.5 py-[3px] text-left text-[12.5px] hover:bg-hover";

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
  const [allForgotten, setAllForgotten] = useState(false);
  const forgotten = useForgotten(docs, !folder);
  const root = agentsRoot(folders)?.id ?? null;
  const isAgents = !!folder && folder.id === root;
  const stance = useAgentsState(isAgents ? root : null);
  const stage = isAgents ? agentsStage(root, stance) : null;
  const tistySkillsHere = holdsTistySkills(folder, folders);
  const fold = (id: string) =>
    setShut((was) => {
      const now = new Set(was);
      if (!now.delete(id)) now.add(id);
      return now;
    });

  const under = folder ? folders.filter((one) => one.parent === folder.id) : [];
  const shelved = Boolean(folder?.away);
  const inside = docs.filter(
    (one) =>
      one.away === shelved &&
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
      many += docs.filter((one) => one.away === shelved && !one.pageOf && one.folder === at).length;
      for (const one of folders) if (one.parent === at) left.push(one.id);
    }
    return many;
  };

  const deeper = folder ? below(folder.id) - inside.length : 0;
  const missing = inside.filter((one) => one.gone).length;
  const said = [
    countedAs(under.length, "folderIsOne", "foldersAre"),
    deeper > 0
      ? countedAs(inside.length + deeper, "paperInAllIsOne", "papersInAll")
      : countedAs(inside.length, "paperIsOne", "papersAre"),
    deeper > 0 ? countedAs(inside.length, "paperRightHereIsOne", "papersRightHere") : "",
    countedAs(missing, "paperMissingIsOne", "papersMissing"),
  ]
    .filter(Boolean)
    .join(" · ");

  const pages = (id: string) => docs.filter((one) => one.pageOf === id).length;
  const folderName = (id: string | null) => {
    const one = folders.find((at) => at.id === id);
    return one ? folderShown(one) : t("unfiled");
  };
  const forgottenShown = allForgotten ? forgotten : forgotten.slice(0, FORGOTTEN_AT_A_GLANCE);
  const rolled = () => {
    atRandom(docs)
      .then((one) => one && onOpen(one))
      .catch(() => {});
  };

  const paper = (one: Filed, recalled?: Recalled) => {
    const worn = led(one.title || t("untitledDoc"));
    return (
      <li key={one.id} className="break-inside-avoid">
        <button
          type="button"
          title={
            recalled &&
            fill(
              recalled.lastOpenedHere ? "forgottenOpenedOn" : "forgottenWroteOn",
              dateOf(recalled.last),
            )
          }
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
          {recalled && (
            <span className="ml-auto shrink-0 text-[11.5px] text-faint">
              {[folderName(one.folder), longAgo(recalled.last)].join(" · ")}
            </span>
          )}
          {!recalled && !one.gone && (one.wrote || one.bytes) && (
            <span className="ml-auto shrink-0 text-[11.5px] text-faint tabular-nums">
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
        <FolderMark folder={one} root={root} className="h-3.5 w-3.5" />
      </span>
      <span className="truncate font-medium text-ink">{folderShown(one)}</span>
      <span className="shrink-0 text-[11.5px] text-faint">
        {one.holds
          ? countedAs(one.holds, "paperIsOne", "papersAre")
          : !holdsTistySkills(one, folders) && t("folderEmpty")}
      </span>
    </button>
  );

  return (
    <main className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="scroller min-h-0 flex-1 px-7 pb-8">
        {!folder && docs.length > 0 && (
          <section aria-label={t("forgottenHeading")} className="mb-5">
            <div className="flex items-center gap-2">
              {forgotten.length > 0 && (
                <h2 className="text-[10.5px] font-semibold tracking-[0.05em] text-faint uppercase">
                  {t("forgottenHeading")}
                </h2>
              )}
              <span className="h-px min-w-6 flex-1 bg-hair" />
              <button
                type="button"
                onClick={rolled}
                className="flex shrink-0 cursor-pointer items-center gap-1.5 rounded-md px-1.5 py-[3px] text-[12.5px] text-soft hover:bg-hover hover:text-ink"
              >
                <Glyph name="dice" className="h-3.5 w-3.5" />
                {t("atRandom")}
              </button>
            </div>
            {forgottenShown.length > 0 && (
              <ul className={`mt-1 ${INDEX}`}>
                {forgottenShown.map((one) => paper(one.doc, one))}
              </ul>
            )}
            {forgotten.length > FORGOTTEN_AT_A_GLANCE && (
              <button
                type="button"
                aria-expanded={allForgotten}
                onClick={() => setAllForgotten(!allForgotten)}
                className="-ml-1.5 mt-1 cursor-pointer rounded-md px-1.5 py-[3px] text-[12.5px] text-faint hover:bg-hover hover:text-soft"
              >
                {allForgotten
                  ? t("forgottenSeeFewer")
                  : fill("forgottenSeeAll", String(forgotten.length))}
              </button>
            )}
          </section>
        )}
        {trail.length > 0 && (
          <nav aria-label={t("whereYouAre")} className="mb-2 flex flex-wrap items-center gap-1.5">
            {trail.map((up, at) => (
              <span key={up.id} className="flex items-center gap-1.5">
                {at > 0 && (
                  <span aria-hidden="true" className="text-[10.5px] text-faint">
                    ›
                  </span>
                )}
                <button
                  type="button"
                  onClick={() => onHere(up.id)}
                  className="cursor-pointer text-[11.5px] text-faint hover:text-soft"
                >
                  {folderShown(up)}
                </button>
              </span>
            ))}
          </nav>
        )}

        <h1 className="text-[21px] font-semibold">
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
            <span
              className={
                isAgents ? agentsPaint(stage) : folder ? painted(folder.color) : "text-faint"
              }
            >
              {folder ? (
                <FolderMark folder={folder} root={root} className="h-[19px] w-[19px]" />
              ) : (
                <Glyph name="inbox" className="h-[19px] w-[19px]" />
              )}
            </span>
            {folder ? folderShown(folder) : t("unfiled")}
          </button>
        </h1>
        {stage === "brought" && (
          <p className="mt-1 text-[12.5px] text-faint">
            {t("agentsNotSynced")} ·{" "}
            <button
              type="button"
              onClick={syncAgents}
              className="cursor-pointer text-accent underline-offset-2 hover:underline"
            >
              {t("agentsSync")}
            </button>
          </p>
        )}
        <p className="mt-1 text-[12.5px] text-faint">
          {under.length + inside.length > 0 ? said : !tistySkillsHere && t("folderHoldsNothing")}
        </p>

        {under.map((one) => {
          const closed = shut.has(one.id);
          const deeper = folders.filter((at) => at.parent === one.id);
          const held = docs.filter(
            (at) => at.away === shelved && !at.pageOf && at.folder === one.id,
          );
          const shown = held.slice(0, Math.max(PEEK - deeper.length, 2));
          const left = held.length - shown.length;
          const skillsIn = holdsTistySkills(one, folders);
          return (
            <section key={one.id} className="mt-4">
              <div className="flex items-center gap-1">
                <button
                  type="button"
                  onClick={() => fold(one.id)}
                  aria-label={fill(closed ? "openFolder" : "closeFolder", folderShown(one))}
                  aria-expanded={!closed}
                  aria-controls={`holds-${one.id}`}
                  className="grid h-5 w-3 shrink-0 cursor-pointer place-items-center rounded-md text-[9px] text-faint hover:text-ink"
                >
                  <span className={`transition-transform ${closed ? "-rotate-90" : ""}`}>▼</span>
                </button>
                {named(one)}
                <span className="h-px min-w-6 flex-1 bg-hair" />
              </div>
              {!closed && (deeper.length > 0 || held.length > 0 || skillsIn) && (
                <ul id={`holds-${one.id}`} className={`mt-1 pl-5 ${INDEX}`}>
                  {deeper.map((at) => (
                    <li key={at.id} className="break-inside-avoid">
                      {named(at)}
                    </li>
                  ))}
                  {shown.map((one) => paper(one))}
                  {skillsIn && <TistySkillsEntry className={ROW} />}
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

        {(inside.length > 0 || tistySkillsHere) && (
          <>
            {under.length > 0 && (
              <h2 className="mt-5 mb-1 text-[10.5px] font-semibold tracking-[0.05em] text-faint uppercase">
                {t("papersHere")}
              </h2>
            )}
            <ul className={`mt-3 ${INDEX}`}>
              {inside.map((one) => paper(one))}
              {tistySkillsHere && <TistySkillsEntry className={ROW} />}
            </ul>
          </>
        )}
      </div>
    </main>
  );
}
