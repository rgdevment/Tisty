import { useEffect, useMemo, useRef, useState } from "react";
import type { AgentKind } from "../agentDocs";
import { agentGroups } from "../agentGroups";
import {
  agentsDrawn,
  agentsPaint,
  agentsTree,
  askAgents,
  folderShown,
  holdsTistySkills,
  inAgents,
  ofAnother,
} from "../agentsSync";
import type { Filed, Folded, Papers, Wired } from "../core";
import { AGENTS, agentsHidden, BRAINSTORM, DIARY, useHidden } from "../hiding";
import { led } from "../leading";
import { fill, t } from "../locales";
import { placeOf, rulesOf, useRuling } from "../places";
import { compactedBy } from "../tidying";
import { clientNamed } from "../who";
import AgentGroupRows, { groupsDrawn, type RowOf } from "./AgentGroupRows";
import { useAgentsStage, useSkillsRow } from "./AgentsMark";
import { FolderMark } from "./AssistantMark";
import {
  type Drawn,
  grouped,
  inSkill,
  KIND_GLYPH,
  type SkillGroup,
  useAgentRows,
} from "./agentRows";
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
import { letterOf, modKey, onMac, shortcut } from "./macKeys";
import { pagesIn, remembered, standingIn, treeIndex } from "./treeIndex";

const STIRS = 4;

const KINDS: AgentKind[] = ["rule", "skill", "memory"];

const KIND_WORD = {
  rule: "agentsKindRule",
  skill: "agentsKindSkill",
  memory: "agentsKindMemory",
} as const;

const yearOf = (book: Filed): number => Number(book.title.match(/[0-9]{4}/)?.[0] ?? 0);

const guestSaid = (doc: Filed) => {
  if (doc.editing && doc.guest) return fill("guestEditing", doc.guest);
  return doc.guest ? fill("guestDoc", doc.guest) : t("guestDocPlain");
};

type Folds = ReturnType<typeof useFolds>;

/** Which folders and documents stand open; kept by whoever outlives the tree, so Settings does not shut them. */
export const useFolds = () => {
  // Nothing arrives open: with two hundred documents a tree that unfolds itself is a wall.
  const [opened, setOpened] = useState<Set<string>>(new Set());
  const [spread, setSpread] = useState<Set<string>>(new Set());
  return { opened, setOpened, spread, setSpread };
};

interface Props {
  papers: Papers;
  folds?: Folds;
  pinned?: string | null;
  ideas?: number;
  ideasMarked?: number;
  unfolded?: boolean;
  reveal?: { folder: string; at: number } | null;
  open?: string;
  here?: string | null;
  /// The row whose menu is open, so a right click says which one it landed on.
  acting?: string | null;
  onOpen: (doc: Filed) => void;
  onDiary?: () => void;
  onIdeas?: () => void;
  onDiaryMenu?: (at: { x: number; y: number }) => void;
  onIdeasMenu?: (at: { x: number; y: number }) => void;
  onAgentsMenu?: (at: { x: number; y: number }) => void;
  onFile: (doc: string, folder?: string, before?: string) => void;
  onPage?: (doc: string, pageOf: string) => void;
  onHere?: (folder?: string) => void;
  onMove?: (folder: string, parent?: string, before?: string) => void;
  onFolderMenu?: (folder: Folded, at: { x: number; y: number }) => void;
  onHereMenu?: (at: { x: number; y: number }) => void;
  onDocMenu?: (doc: Filed, at: { x: number; y: number }) => void;
  /// The documents picked with Ctrl or Shift and a click, for one action over all of them.
  picked?: ReadonlySet<string>;
  onPicked?: (docs: string[]) => void;
  onDelete?: (row: { id: string; kind: "doc" | "folder" }) => void;
  onRename?: (folder: Folded) => void;
  /// The connected agents Settings already read, for the Hooks group of Global.
  agents?: readonly Wired[] | null;
  onHooks?: () => void;
  onNewRule?: (level: string) => void;
}

export default function Tree({
  papers,
  folds,
  pinned,
  ideas,
  ideasMarked,
  unfolded = false,
  reveal,
  open,
  here,
  acting,
  onOpen,
  onDiary,
  onIdeas,
  onDiaryMenu,
  onIdeasMenu,
  onAgentsMenu,
  onFile,
  onPage,
  onHere,
  onMove,
  onFolderMenu,
  onHereMenu,
  onDocMenu,
  picked,
  onPicked,
  onDelete,
  onRename,
  agents = null,
  onHooks,
  onNewRule,
}: Props) {
  useRuling();
  const { root, stage, stance } = useAgentsStage(papers);
  const skillsRow = useSkillsRow(stance);
  const hidden = useHidden();
  // While the sidebar is sifted everything shows, so a match never sits behind a hidden row.
  const veiled = (key: string) => !unfolded && hidden.includes(key);
  const agentsVeiled = !unfolded && agentsHidden(hidden, root);
  const unborn = !root && !agentsVeiled && !unfolded;
  const agentRows = useAgentRows(papers);
  const groups = useMemo(
    () => (root ? agentGroups(papers.docs, papers.folders, agentRows, root) : null),
    [papers, agentRows, root],
  );
  const index = useMemo(() => treeIndex(papers.docs), [papers.docs]);
  // What Global draws in its groups is not drawn again where it is kept.
  const projected = useMemo(
    () =>
      new Set(
        (groups?.rules ?? [])
          .flatMap((one) => one.docs)
          .concat(
            (groups?.skills ?? []).flatMap((one) => ("doc" in one ? [one.doc] : one.skill.docs)),
          )
          .map((one) => one.id),
      ),
    [groups],
  );
  const own = useFolds();
  const { opened, setOpened, spread, setSpread } = folds ?? own;
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
    const walled =
      kind === "folder" && !inAgents(id, papers.folders) ? agentsTree(papers.folders) : undefined;
    holding.current = { id, kind, tall, walled, x: e.clientX, y: e.clientY, on: false };
    took.current = false;
    setLifted(null);
  };

  const tapped = (go: (e: React.MouseEvent) => void) => (e: React.MouseEvent) => {
    if (took.current) {
      took.current = false;
      return;
    }
    go(e);
  };

  const anchor = useRef<string | null>(null);
  const isDoc = (id: string) => papers.docs.some((one) => one.id === id);

  // Ctrl or Cmd adds one, Shift adds the run since the last one; a plain click lets them all go.
  const picking = (e: React.MouseEvent, id: string): boolean => {
    if (!onPicked) return false;
    const now = new Set(picked ?? []);
    if (e.ctrlKey || e.metaKey) {
      if (now.has(id)) now.delete(id);
      else now.add(id);
      anchor.current = id;
      onPicked([...now]);
      return true;
    }
    if (e.shiftKey && anchor.current) {
      const order = rows()
        .map((row) => row.dataset.row ?? "")
        .filter(isDoc);
      const [from, to] = [order.indexOf(anchor.current), order.indexOf(id)].sort((a, b) => a - b);
      if (from >= 0) onPicked([...new Set([...now, ...order.slice(from, to + 1)])]);
      return true;
    }
    anchor.current = id;
    if (now.size) onPicked([]);
    return false;
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
        setCarried({ id: held.id, kind: held.kind, tall: held.tall, walled: held.walled });
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
        { id: held.id, kind: held.kind, tall: held.tall, walled: held.walled },
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

  const answered = useRef(0);
  useEffect(() => {
    if (!reveal || reveal.at === answered.current) return;
    answered.current = reveal.at;
    const way = lineTo(reveal.folder);
    setOpened((were) => new Set([...were, ...way]));
    requestAnimationFrame(() =>
      listed.current
        ?.querySelector<HTMLElement>(`[data-row="${reveal.folder}"]`)
        ?.scrollIntoView({ block: "nearest" }),
    );
  });

  useEffect(() => {
    if (reached && !listed.current?.querySelector(`[data-row="${reached}"]`)) setReached(null);
  });

  // What holds the open document opens once when it is opened, and folds again when asked.
  const shownOpen = useRef<string | undefined>(undefined);
  useEffect(() => {
    if (!open || shownOpen.current === open) return;
    const doc = papers.docs.find((one) => one.file === open);
    if (!doc) return;
    shownOpen.current = open;
    const up = doc.pageOf ? papers.docs.find((one) => one.id === doc.pageOf) : undefined;
    const row = agentRows.get(doc.file);
    const holders = [
      up?.id,
      doc.diary || up?.diary ? DIARY : undefined,
      row?.skill ? `skill:${doc.folder ?? ""}:${row.level}:${row.skill}` : undefined,
    ].filter((one): one is string => Boolean(one));
    if (holders.length) setSpread((were) => new Set([...were, ...holders]));
  });

  const stops = (id: string) => (reached ?? first) === id;

  const walk = (from: string, by: number) => {
    const all = rows();
    const now = all.findIndex((row) => row.dataset.row === from);
    const next = all[now + by];
    if (!next) return;
    setReached(next.dataset.row ?? null);
    next.focus();
  };

  // A row drawn only, never a folder: the arrow keys reach it, and nothing is dropped on it.
  const rowOf: RowOf = (key) =>
    ({
      "data-row": key,
      tabIndex: stops(key) ? 0 : -1,
      onFocus: () => setReached(key),
      onKeyDown: (e: React.KeyboardEvent) => {
        if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
        e.preventDefault();
        walk(key, e.key === "ArrowDown" ? 1 : -1);
      },
    }) as ReturnType<RowOf>;

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
    if (e.key === "Home" || e.key === "End") {
      e.preventDefault();
      const all = rows();
      const to = e.key === "Home" ? all[0] : all[all.length - 1];
      return to && walk(to.dataset.row ?? "", 0);
    }
    if (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10")) {
      e.preventDefault();
      const box = (e.target as HTMLElement).getBoundingClientRect();
      const at = { x: box.left + 24, y: box.bottom };
      if (row.kind === "doc") {
        const doc = papers.docs.find((one) => one.id === row.id);
        if (doc) onDocMenu?.(doc, at);
      } else if (row.id === "unfiled") {
        onHereMenu?.(at);
      } else {
        const folder = papers.folders.find((one) => one.id === row.id);
        if (folder) onFolderMenu?.(folder, at);
      }
      return;
    }
    const deleting = e.key === "Delete" || (onMac && e.metaKey && e.key === "Backspace");
    if (deleting && row.id !== "unfiled") {
      e.preventDefault();
      // As its menu: an archived folder comes back before it can go.
      const shelved =
        row.kind === "folder" && papers.folders.some((one) => one.id === row.id && one.away);
      return shelved || onDelete?.({ id: row.id, kind: row.kind });
    }
    if (e.key === "F2" && row.kind === "folder") {
      e.preventDefault();
      // As its menu: a folder of Agents is named after its assistant, and one archived waits to come back.
      const folder = papers.folders.find((one) => one.id === row.id);
      return folder && !folder.away && !inAgents(folder.id, papers.folders) && onRename?.(folder);
    }
    if (e.key === "Escape" && picked?.size) {
      e.preventDefault();
      return onPicked?.([]);
    }
    if (e.key === "Escape" && lifted) {
      e.preventDefault();
      return setLifted(null);
    }
    if (
      modKey(e) &&
      letterOf(e) === "x" &&
      row.id !== "unfiled" &&
      !papers.docs.some(
        (one) =>
          one.id === row.id &&
          (one.pageOf ||
            (one.away && !one.archived) ||
            !rulesOf(placeOf(one, papers, pinned)).moves),
      )
    ) {
      e.preventDefault();
      return setLifted(row);
    }
    if (modKey(e) && letterOf(e) === "v" && row.kind === "folder") {
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
      if (e.key === "ArrowRight" && !opened.has(row.id)) {
        e.preventDefault();
        return fold(row.id);
      }
      if (e.key === "ArrowLeft" && opened.has(row.id)) {
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
    if (opened.has(id)) setReached(id);
    setOpened((were) => turned(were, id));
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

  const shelved = (folder: string | null) =>
    papers.folders.some((one) => one.id === folder && one.away);

  const under = (parent: string | null) =>
    papers.folders.filter((one) => one.parent === parent && one.away === shelved(parent));

  const pin = papers.docs.find((one) => one.file === pinned && !one.away && !one.pageOf);
  const pinShown = pin && !veiled(BRAINSTORM) ? pin : undefined;
  const ideasUnborn = !pin && Boolean(onIdeas) && !unfolded && !veiled(BRAINSTORM);

  const drawnUnder = (parent: string | null) =>
    agentsDrawn(under(parent), papers.folders, root).filter(
      (one) =>
        one.away ||
        (!veiled(one.id) && !(one.id === root && agentsVeiled) && !groups?.emptied.has(one.id)),
    );

  const gathered = (one: Filed) => Boolean(onDiary && one.diary && !one.pageOf && !one.away);
  const diaries = papers.docs
    .filter(gathered)
    .sort((one, two) => yearOf(two) - yearOf(one) || two.title.localeCompare(one.title));
  const diaryShown = Boolean(onDiary) && (!unfolded || diaries.length > 0) && !veiled(DIARY);
  // Only a row the tree draws can hold the tab stop: a page or a book folded away never would.
  const first =
    pinShown?.id ??
    (ideasUnborn ? BRAINSTORM : undefined) ??
    (diaryShown ? DIARY : undefined) ??
    drawnUnder(null)[0]?.id ??
    (unborn ? AGENTS : undefined) ??
    "unfiled";

  const inside = remembered((folder: string | null) =>
    standingIn(index, folder).filter(
      (one) =>
        one !== pin && !gathered(one) && !projected.has(one.id) && one.away === shelved(folder),
    ),
  );

  const pagesOf = (id: string) => pagesIn(index, id);

  /// The shelf keeps the whole of a folder that was put away, and the loose ones under it in a row.
  const shelves = papers.folders.filter(
    (one) => one.archived && !papers.folders.some((up) => up.id === one.parent && up.away),
  );

  const away = papers.docs.filter((one) => one.away && !one.pageOf && !shelved(one.folder ?? null));

  const folderNamed = (id: string) => papers.folders.find((one) => one.id === id);

  const namesOf = (id: string) => lineTo(id).map((at) => folderNamed(at)?.name ?? "");

  const sameWay = (one: string[], two: string[]) =>
    one.length === two.length && one.every((name, at) => name === two[at]);

  const pathOf = (doc: Filed): string[] => {
    if (doc.folder && folderNamed(doc.folder)) return lineTo(doc.folder);
    const was = doc.folderWas ?? [];
    const again = was.length
      ? papers.folders.find((one) => !one.away && sameWay(namesOf(one.id), was))
      : undefined;
    return again ? lineTo(again.id) : [];
  };

  const trace = (folder: Folded, depth: number, key: string) => (
    <li key={key} className="relative">
      <div className="group/row flex items-center rounded-md" title={t("folderTraceWhy")}>
        <span
          aria-hidden="true"
          className="grid h-5 w-[18px] shrink-0 place-items-center"
          style={{ marginLeft: `${8 + depth * STEP}px` }}
        >
          <span className={`flex items-center ${painted(folder.color)}`}>
            <FolderMark folder={folder} root={root} className="h-[15px] w-[15px]" />
          </span>
        </span>
        <span className="flex min-w-0 flex-1 items-center gap-1.5 py-1 pl-1.5 text-[13px] text-faint">
          <span className="truncate">{folderShown(folder)}</span>
          <span className="ml-auto shrink-0 pr-1 text-[10.5px] opacity-0 transition-opacity group-hover/row:opacity-100 motion-reduce:transition-none">
            {t("folderTrace")}
          </span>
        </span>
      </div>
    </li>
  );

  const lost = (name: string, depth: number, key: string) => (
    <li key={key} className="relative">
      <div className="group/row flex items-center rounded-md" title={t("folderTraceGone")}>
        <span
          aria-hidden="true"
          className="grid h-5 w-[18px] shrink-0 place-items-center text-faint"
          style={{ marginLeft: `${8 + depth * STEP}px` }}
        >
          <Glyph name="folder" className="h-[15px] w-[15px]" />
        </span>
        <span className="flex min-w-0 flex-1 items-center gap-1.5 py-1 pl-1.5 text-[13px] text-faint">
          <span className="truncate">{name}</span>
          <span className="ml-auto shrink-0 pr-1 text-[10.5px] opacity-0 transition-opacity group-hover/row:opacity-100 motion-reduce:transition-none">
            {t("folderTrace")}
          </span>
        </span>
      </div>
    </li>
  );

  const shelfOf = (doc: Filed) => {
    const path = pathOf(doc);
    const named = path.length ? namesOf(path[path.length - 1]) : (doc.folderWas ?? []);
    return [...named, "\u0000", ...path].join("/");
  };

  const pathed = () => {
    const rows: React.ReactNode[] = [];
    const drawn = new Set<string>();
    const sorted = [...away].sort((one, two) => shelfOf(one).localeCompare(shelfOf(two)));
    for (const doc of sorted) {
      const path = pathOf(doc);
      path.forEach((id, at) => {
        const key = path.slice(0, at + 1).join("/");
        const folder = folderNamed(id);
        if (drawn.has(key) || !folder) return;
        drawn.add(key);
        rows.push(trace(folder, at + 1, `trace-${key}`));
      });
      const was = doc.folderWas ?? [];
      if (!path.length && was.length) {
        was.forEach((name, at) => {
          const key = `lost:${was.slice(0, at + 1).join("/")}`;
          if (drawn.has(key)) return;
          drawn.add(key);
          rows.push(lost(name, at + 1, `trace-${key}`));
        });
      }
      rows.push(paper(doc, (path.length || was.length) + 1));
    }
    return rows;
  };

  const takesPages = (doc: Filed) => !doc.pageOf && !doc.away && Boolean(onPage);

  const nextOf = <T extends { id: string }>(all: T[], id: string) => {
    const at = all.findIndex((one) => one.id === id);
    return at < 0 ? undefined : all[at + 1]?.id;
  };

  const paper = (doc: Filed, depth: number, page = false, inGroup = false) => {
    const rules = rulesOf(placeOf(doc, papers, pinned));
    const row = page ? undefined : agentRows.get(doc.file);
    const name = row
      ? inGroup
        ? inSkill(row)
        : row.label
      : doc === pin
        ? t("brainstorm")
        : doc.title || t("untitledDoc");
    const worn = page ? { mark: null, rest: name } : led(name);
    const pages = pagesOf(doc.id);
    const closed = !unfolded && !spread.has(doc.id);
    const mark = worn.mark ? (
      <span className="text-[12.5px] leading-none">{worn.mark}</span>
    ) : (
      <Glyph
        name={
          row
            ? KIND_GLYPH[row.kind]
            : page
              ? doc.archived
                ? "archive"
                : "alignleft"
              : doc === pin
                ? "idea"
                : "page"
        }
        className={
          page
            ? "h-[11px] w-[11px] opacity-70"
            : doc === pin
              ? "brainstorm-mark h-[14px] w-[14px]"
              : "h-[13px] w-[13px]"
        }
      />
    );
    return (
      <li key={doc.id} className="relative">
        <div
          data-drop={
            page || inGroup || projected.has(doc.id) || doc.away || !rules.moves
              ? undefined
              : doc.id
          }
          data-drop-kind="doc"
          data-drop-parent={doc.folder ?? ""}
          data-drop-next={nextOf(inside(doc.folder ?? null), doc.id) ?? ""}
          data-drop-holds={takesPages(doc) && rules.pages ? "yes" : "no"}
          data-drop-line={lineTo(doc.folder ?? null).join("/")}
          className={`group/row relative flex items-center rounded-md has-[:focus-visible]:bg-hover ${
            acting === doc.id ? "bg-active " : ""
          }${over === doc.id || picked?.has(doc.id) ? "bg-accent-soft" : ""}${
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
            onPointerDown={(e) => !page && rules.moves && grab(e, doc.id, "doc")}
            data-row={doc.id}
            tabIndex={stops(doc.id) ? 0 : -1}
            onFocus={() => setReached(doc.id)}
            onKeyDown={(e) => typed(e, { id: doc.id, kind: "doc", name })}
            aria-keyshortcuts={shortcuts(page ? "page" : "doc")}
            onClick={tapped((e) => !picking(e, doc.id) && onOpen(doc))}
            title={row ? `${row.level} · ${row.name}` : undefined}
            aria-label={
              picked?.has(doc.id)
                ? fill("pickedIs", name)
                : lifted?.id === doc.id
                  ? fill("liftedIs", name)
                  : doc.away
                    ? `${name} — ${t("isArchived")}`
                    : doc.flagged
                      ? `${name} — ${t("docFlagged")}`
                      : name
            }
            aria-current={open === doc.file ? "true" : undefined}
            className={`flex min-w-0 flex-1 items-center gap-1.5 rounded-md py-1 pl-1.5 pr-2 text-left text-[13px] ${
              lifted?.id === doc.id ? "ring-1 ring-accent " : ""
            }${carried?.id === doc.id ? "opacity-45 " : ""}${doc.away ? "text-faint " : ""}${
              open === doc.file
                ? "bg-accent-soft font-semibold text-accent"
                : `${page ? "text-faint" : "text-soft"} hover:bg-hover`
            }`}
          >
            <span className="truncate">{worn.rest}</span>
            {doc.gone && (
              <span title={t("goneDoc")} className="shrink-0 text-[9px] text-urgent">
                ⚠
              </span>
            )}
            {doc.flagged && !doc.away && (
              <span
                title={
                  clientNamed(doc.flagged.via)
                    ? fill("docFlaggedBy", clientNamed(doc.flagged.via) as string)
                    : t("docFlagged")
                }
                className="shrink-0 text-[9px] text-hue-teal"
              >
                ◆
              </span>
            )}
            {doc === pin && ideasMarked !== undefined && ideasMarked > 0 && (
              <span
                title={
                  ideasMarked === 1 ? t("ideaMarked") : fill("ideasMarked", String(ideasMarked))
                }
                className="shrink-0 text-[9px] text-hue-teal"
              >
                ◆
              </span>
            )}
            {doc.compacted && !doc.away && (
              <span title={compactedBy(doc)} className="shrink-0 text-[9px] text-hue-teal">
                ◇
              </span>
            )}
            {doc.guest !== null && doc.guest !== undefined && (
              <span title={guestSaid(doc)} className="shrink-0 truncate text-[10.5px] text-faint">
                {doc.guest || t("guestNone")}
              </span>
            )}
            {doc === pin && ideas !== undefined && ideas > 0 && (
              <span
                title={fill("ideasWaiting", String(ideas))}
                className="ml-auto shrink-0 rounded-full bg-hover px-1.5 text-[10.5px] text-soft"
              >
                {ideas}
              </span>
            )}
            {pages.length > 0 && (
              <span className="ml-auto shrink-0 pl-2 text-[11.5px] text-faint">
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

  const skill = (group: SkillGroup, depth: number) => {
    const closed = !unfolded && !spread.has(group.key);
    const mark = <Glyph name={KIND_GLYPH.skill} className="h-[13px] w-[13px]" />;
    const toggle = fill(closed ? "agentsSkillShow" : "agentsSkillHide", group.label);
    return (
      <li key={group.key} className="relative">
        <div className="group/row relative flex items-center rounded-md has-[:focus-visible]:bg-hover">
          <Grip
            at={8 + depth * STEP}
            open={!closed}
            label={toggle}
            controls={`files-${group.key}`}
            onPress={() => unfold(group.key)}
          >
            {mark}
          </Grip>
          <button
            type="button"
            data-row={group.key}
            data-skill=""
            tabIndex={stops(group.key) ? 0 : -1}
            onFocus={() => setReached(group.key)}
            onKeyDown={(e) => {
              const shut = e.key === "ArrowLeft" && !closed;
              if (shut || (e.key === "ArrowRight" && closed)) {
                e.preventDefault();
                return unfold(group.key);
              }
              if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
              e.preventDefault();
              walk(group.key, e.key === "ArrowDown" ? 1 : -1);
            }}
            onClick={() => unfold(group.key)}
            aria-expanded={!closed}
            aria-controls={`files-${group.key}`}
            aria-label={`${group.label}, ${fill("agentsSkillFiles", String(group.docs.length))}`}
            className="flex min-w-0 flex-1 items-center gap-1.5 rounded-md py-1 pl-1.5 pr-2 text-left text-[13px] text-soft hover:bg-hover"
          >
            <span className="truncate">{group.label}</span>
            <span className="ml-auto shrink-0 pl-2 text-[11.5px] text-faint">
              {fill("agentsSkillFiles", String(group.docs.length))}
            </span>
          </button>
        </div>
        {!closed && (
          <span
            aria-hidden="true"
            className="absolute bottom-1 w-px bg-hair"
            style={{ left: `${SPINE + depth * STEP}px`, top: "26px" }}
          />
        )}
        {!closed && (
          <ul id={`files-${group.key}`}>
            {group.docs.map((one) => paper(one, depth + 1, false, true))}
          </ul>
        )}
      </li>
    );
  };

  // Inside an assistant's folder its rules, skills and memory stand apart, only as drawn: no file moves.
  const byKind = (
    drawn: Drawn[],
    depth: number,
  ): (Drawn | { kind: AgentKind; heading: React.ReactNode })[] => {
    const kindOf = (one: Drawn): AgentKind | undefined =>
      "skill" in one ? "skill" : agentRows.get(one.doc.file)?.kind;
    const kinds = KINDS.filter((kind) => drawn.some((one) => kindOf(one) === kind));
    if (kinds.length < 2 || drawn.some((one) => !kindOf(one))) return drawn;
    return kinds.flatMap((kind) => [
      {
        kind,
        heading: (
          <li
            key={`kind-${kind}`}
            className="pt-1.5 pb-0.5 text-[10.5px] font-semibold tracking-[0.06em] text-faint uppercase"
            style={{ paddingLeft: `${8 + depth * STEP + 4}px` }}
          >
            {t(KIND_WORD[kind])}
          </li>
        ),
      },
      ...drawn.filter((one) => kindOf(one) === kind),
    ]);
  };

  // With no idea written yet, Ideas still stands in the library, and opening it makes the sheet.
  const ideasRow = () => (
    <li key={BRAINSTORM} className="relative">
      <div
        className="group/row relative flex items-center rounded-md has-[:focus-visible]:bg-hover"
        onContextMenu={(e) => {
          if (!onIdeasMenu) return;
          e.preventDefault();
          onIdeasMenu({ x: e.clientX, y: e.clientY });
        }}
      >
        <span
          aria-hidden="true"
          className="ml-2 grid h-5 w-[18px] shrink-0 place-items-center text-faint"
        >
          <Glyph name="idea" className="brainstorm-mark h-[14px] w-[14px]" />
        </span>
        <button
          type="button"
          data-row={BRAINSTORM}
          tabIndex={stops(BRAINSTORM) ? 0 : -1}
          onFocus={() => setReached(BRAINSTORM)}
          onKeyDown={(e) => {
            if (onIdeasMenu && (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10"))) {
              e.preventDefault();
              const box = e.currentTarget.getBoundingClientRect();
              return onIdeasMenu({ x: box.left + 24, y: box.bottom });
            }
            if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
            e.preventDefault();
            walk(BRAINSTORM, e.key === "ArrowDown" ? 1 : -1);
          }}
          aria-keyshortcuts={onIdeasMenu ? "Shift+F10" : undefined}
          onClick={onIdeas}
          className="flex min-w-0 flex-1 items-center gap-1.5 rounded-md py-1 pl-1.5 pr-2 text-left text-[13px] text-soft hover:bg-hover"
        >
          <span className="truncate">{t("brainstorm")}</span>
        </button>
      </div>
    </li>
  );

  const diary = (onToday: () => void) => {
    const closed = !unfolded && !spread.has(DIARY);
    const mark = <Glyph name="book-heart" className="diary-mark h-[14px] w-[14px]" />;
    return (
      <li key={DIARY} className="relative">
        <div
          className="group/row relative flex items-center rounded-md has-[:focus-visible]:bg-hover"
          onContextMenu={(e) => {
            if (!onDiaryMenu) return;
            e.preventDefault();
            onDiaryMenu({ x: e.clientX, y: e.clientY });
          }}
        >
          {diaries.length > 0 ? (
            <Grip
              at={8}
              open={!closed}
              label={fill(closed ? "showPages" : "hidePages", t("diary"))}
              controls="pages-diary"
              onPress={() => unfold(DIARY)}
            >
              {mark}
            </Grip>
          ) : (
            <span
              aria-hidden="true"
              className="ml-2 grid h-5 w-[18px] shrink-0 place-items-center text-faint"
            >
              {mark}
            </span>
          )}
          <button
            type="button"
            data-row={DIARY}
            tabIndex={stops(DIARY) ? 0 : -1}
            onFocus={() => setReached(DIARY)}
            onKeyDown={(e) => {
              if (onDiaryMenu && (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10"))) {
                e.preventDefault();
                const box = e.currentTarget.getBoundingClientRect();
                return onDiaryMenu({ x: box.left + 24, y: box.bottom });
              }
              if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
              e.preventDefault();
              walk(DIARY, e.key === "ArrowDown" ? 1 : -1);
            }}
            aria-keyshortcuts={onDiaryMenu ? "Shift+F10" : undefined}
            onClick={onToday}
            title={t("diaryOpenToday")}
            className="flex min-w-0 flex-1 items-center gap-1.5 rounded-md py-1 pl-1.5 pr-2 text-left text-[13px] text-soft hover:bg-hover"
          >
            <span className="truncate">{t("diary")}</span>
          </button>
        </div>
        {diaries.length > 0 && !closed && (
          <span
            aria-hidden="true"
            className="absolute bottom-1 w-px bg-hair"
            style={{ left: `${SPINE}px`, top: "26px" }}
          />
        )}
        {diaries.length > 0 && !closed && (
          <ul id="pages-diary">{diaries.map((book) => paper(book, 1))}</ul>
        )}
      </li>
    );
  };

  const branch = (folder: Folded, depth: number) => {
    const closed = !unfolded && !opened.has(folder.id);
    const isRoot = folder.id === root;
    // Nothing brought yet: the row opens the first time instead of an empty folder.
    const dim = isRoot && stage === "unbrought";
    const shown = folderShown(folder);
    const kids = drawnUnder(folder.id);
    const papersIn = inside(folder.id);
    const global = holdsTistySkills(folder, papers.folders);
    const theirs = ofAnother(folder);
    return (
      <li key={folder.id} className={folder.away || theirs ? "relative text-faint" : "relative"}>
        <div
          data-drop={folder.away ? undefined : folder.id}
          data-drop-kind="folder"
          data-drop-parent={folder.parent ?? ""}
          data-drop-next={nextOf(under(folder.parent ?? null), folder.id) ?? ""}
          data-drop-holds={folder.away ? "no" : "yes"}
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
            className={`group/row flex items-center rounded-md ${
              acting === folder.id ? "bg-active" : ""
            }`}
            onContextMenu={(e) => {
              if (!onFolderMenu) return;
              e.preventDefault();
              onFolderMenu(folder, { x: e.clientX, y: e.clientY });
            }}
          >
            <Grip
              at={8 + depth * STEP}
              open={!closed}
              label={fill(closed ? "openFolder" : "closeFolder", shown)}
              controls={`holds-${folder.id}`}
              onPress={() => fold(folder.id)}
            >
              <span
                className={`flex items-center ${isRoot ? agentsPaint(stage) : painted(folder.color)}`}
              >
                <FolderMark
                  folder={folder}
                  root={root}
                  className={theirs ? "h-[15px] w-[15px] opacity-50" : "h-[15px] w-[15px]"}
                />
              </span>
            </Grip>
            <button
              type="button"
              data-row={folder.id}
              tabIndex={stops(folder.id) ? 0 : -1}
              onFocus={() => setReached(folder.id)}
              onKeyDown={(e) => typed(e, { id: folder.id, kind: "folder", name: shown }, folder.id)}
              aria-keyshortcuts={shortcuts("folder")}
              onPointerDown={(e) => grab(e, folder.id, "folder")}
              onClick={tapped(() => (dim ? askAgents() : onHere?.(folder.id)))}
              aria-label={folderSaid(folder)}
              aria-current={here === folder.id ? "true" : undefined}
              className={`flex min-w-0 flex-1 items-center gap-1.5 py-1 pl-1.5 text-left text-[13px] ${
                dim || (theirs && here !== folder.id)
                  ? "text-faint"
                  : here === folder.id
                    ? "text-ink"
                    : "text-soft"
              }`}
            >
              <span className="truncate">{shown}</span>
              {!dim && (
                <span className="ml-auto pr-1 text-[11.5px] text-faint opacity-0 transition-opacity group-hover/row:opacity-100 motion-reduce:transition-none">
                  {folder.holds || ""}
                </span>
              )}
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
            {byKind(grouped(papersIn, agentRows), depth + 1).map((one) =>
              "kind" in one
                ? one.heading
                : "doc" in one
                  ? paper(one.doc, depth + 1)
                  : skill(one.skill, depth + 1),
            )}
            {global && groups && (
              <AgentGroupRows
                groups={groups}
                depth={depth + 1}
                step={STEP}
                paper={paper}
                skill={skill}
                rowOf={rowOf}
                tistySkills={skillsRow}
                agents={agents}
                onHooks={onHooks}
                onNewRule={onNewRule}
              />
            )}
            {!agentsDrawn(under(folder.id), papers.folders, root).length &&
              !papersIn.length &&
              !(global && groups && groupsDrawn(groups, skillsRow, agents, !!onHooks)) && (
                <li
                  className="py-0.5 text-[11.5px] text-faint italic"
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

  // The state the sidebar shows by colour alone is said in words to whoever cannot see it.
  const folderSaid = (folder: Folded) => {
    const shown = folderShown(folder);
    if (lifted?.id === folder.id) return fill("liftedIs", shown);
    if (folder.id !== root) return shown;
    if (stage === "unbrought") return `${shown} — ${t("agentsUnbroughtSaid")}`;
    return stage === "brought" ? `${shown} — ${t("agentsNotSynced")}` : shown;
  };

  // With no folder yet, Agents still stands in the sidebar, grey, so the person learns it exists.
  const unbornRow = () => (
    <li key={AGENTS} className="relative">
      <div
        className="group/row flex items-center rounded-md has-[:focus-visible]:bg-hover"
        onContextMenu={(e) => {
          if (!onAgentsMenu) return;
          e.preventDefault();
          onAgentsMenu({ x: e.clientX, y: e.clientY });
        }}
      >
        <span
          aria-hidden="true"
          className={`ml-2 grid h-5 w-[18px] shrink-0 place-items-center ${agentsPaint("unbrought")}`}
        >
          <Glyph name="robot" className="h-[15px] w-[15px]" />
        </span>
        <button
          type="button"
          data-row={AGENTS}
          tabIndex={stops(AGENTS) ? 0 : -1}
          onFocus={() => setReached(AGENTS)}
          onKeyDown={(e) => {
            if (onAgentsMenu && (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10"))) {
              e.preventDefault();
              const box = e.currentTarget.getBoundingClientRect();
              return onAgentsMenu({ x: box.left + 24, y: box.bottom });
            }
            if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
            e.preventDefault();
            walk(AGENTS, e.key === "ArrowDown" ? 1 : -1);
          }}
          aria-keyshortcuts={onAgentsMenu ? "Shift+F10" : undefined}
          onClick={askAgents}
          aria-label={`${t("agentsName")} — ${t("agentsUnbroughtSaid")}`}
          className="flex min-w-0 flex-1 items-center gap-1.5 py-1 pl-1.5 text-left text-[13px] text-faint"
        >
          <span className="truncate">{t("agentsName")}</span>
        </button>
      </div>
    </li>
  );

  const loose = papers.docs.filter(
    (one) =>
      one !== pin &&
      !gathered(one) &&
      !one.away &&
      !one.pageOf &&
      (one.folder === null || !papers.folders.some((at) => at.id === one.folder)),
  );
  const looseOpen = unfolded || opened.has("unfiled");

  const tree = (
    <ul aria-label={t("docs")} className="flex flex-col gap-px">
      {papers.folders.length === 0 && papers.docs.length === 0 && (
        <li className="px-2.5 py-2 text-[12.5px] text-faint">{t("noDocsYet")}</li>
      )}
      {lifted && (
        <li
          role="status"
          className="mx-1 mb-1 rounded-md bg-accent-soft px-2 py-1 text-[11.5px] text-accent"
        >
          {fill("liftedHint", lifted.name, shortcut("V"))}
        </li>
      )}
      {pinShown && paper(pinShown, 0)}
      {ideasUnborn && ideasRow()}
      {onDiary && diaryShown && diary(onDiary)}
      {drawnUnder(null).flatMap((folder) => [branch(folder, 0)])}
      {unborn && unbornRow()}

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
            open={looseOpen}
            label={fill(looseOpen ? "closeFolder" : "openFolder", t("unfiled"))}
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
            <span className="ml-auto text-[11.5px] opacity-0 transition-opacity group-hover/row:opacity-100 motion-reduce:transition-none">
              {loose.length || ""}
            </span>
          </button>
        </div>
        {looseOpen && (
          <span
            aria-hidden="true"
            className="absolute bottom-1 w-px bg-hair"
            style={{ left: `${SPINE}px`, top: "30px" }}
          />
        )}
        {looseOpen && <ul id="holds-unfiled">{loose.map((doc) => paper(doc, 1))}</ul>}
      </li>
    </ul>
  );

  const held = shelves.length + away.length;

  const kept = held > 0 && (
    <div className="mt-2">
      <button
        type="button"
        onClick={() => fold("away")}
        aria-expanded={opened.has("away")}
        aria-label={t("archived")}
        style={{ top: "var(--tisty-docs-head)" }}
        className="pinned-head sticky z-10 flex w-full items-center gap-1.5 bg-rail px-2.5 py-1 text-[11.5px] font-semibold tracking-[0.06em] text-faint uppercase"
      >
        <span
          aria-hidden
          className={`text-[9px] transition-transform ${opened.has("away") ? "" : "-rotate-90"}`}
        >
          ▼
        </span>
        {t("archived")}
        <span className="ml-auto text-[11.5px] font-normal">{held}</span>
      </button>
      {opened.has("away") && (
        <ul aria-label={t("archived")} className="flex flex-col gap-px">
          {shelves.map((folder) => branch(folder, 1))}
          {pathed()}
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
      tabIndex={-1}
      onClick={onPress}
      aria-label={label}
      aria-expanded={open}
      aria-controls={controls}
      style={{ marginLeft: `${at}px` }}
      className="grid h-5 w-[18px] shrink-0 place-items-center rounded-md text-faint hover:text-ink"
    >
      <span className="col-start-1 row-start-1 flex items-center transition-opacity group-hover/row:opacity-0 group-has-[:focus-visible]/row:opacity-0 motion-reduce:transition-none">
        {children}
      </span>
      <Glyph
        name="chevron"
        className={`col-start-1 row-start-1 h-[13px] w-[13px] opacity-0 transition-[opacity,transform] group-hover/row:opacity-100 group-has-[:focus-visible]/row:opacity-100 motion-reduce:transition-none ${
          open ? "" : "-rotate-90"
        }`}
      />
    </button>
  );
}
