import { type Counted, type List, sow } from "../core";
import { capped, exactly, todayLong } from "../format";
import { fill, t } from "../locales";
import { keepOnly } from "../only";
import type { Chosen, Lean, Named } from "../views";
import DayCounts from "./DayCounts";
import { FIND_KEY, FINDER } from "./finding";
import Glyph from "./Glyph";
import { painted } from "./Hue";
import { asksForMenu, menuAt } from "./menuKeys";
import { useFades } from "./room";

const TAGS_SHOWN = 14;

const VIEWS: { key: Named; icon: string }[] = [
  { key: "quadrants", icon: "grid" },
  { key: "spread", icon: "calendar" },
  { key: "lists", icon: "rows" },
  { key: "tags", icon: "tag" },
  { key: "archive", icon: "archive" },
];

interface Props {
  counts: Record<string, number>;
  lists: List[];
  tags: Counted[];
  chosen: Chosen;
  acting?: string | null;
  onListMenu?: (list: List, at: { x: number; y: number }) => void;
  leaning: Lean;
  onLean: (on: Lean) => void;
  onChoose: (chosen: Chosen) => void;
  query: string;
  onSearch: (query: string) => void;
}

export default function SideTasks({
  counts,
  lists,
  tags,
  chosen,
  acting,
  onListMenu,
  leaning,
  onLean,
  onChoose,
  query,
  onSearch,
}: Props) {
  const wide = leaning === "tasks";
  const plain = !chosen.list && !chosen.tags?.length;
  const on = (key: Named) => plain && chosen.named === key;

  return (
    <div className={`flex flex-col px-2.5 pb-1 ${wide ? "min-h-0 flex-1" : "shrink-0"}`}>
      <Finder query={query} onSearch={onSearch} />
      <div className="flex items-center gap-2 pl-2.5">
        <button
          type="button"
          onClick={() => onChoose({ named: "tasks" })}
          aria-current={on("tasks") ? "page" : undefined}
          className={`flex items-center gap-2 font-semibold hover:text-accent ${
            wide ? "text-[13px]" : "text-[12.5px]"
          } ${on("tasks") ? "text-accent" : wide ? "text-ink" : "text-faint"}`}
        >
          {wide && <Glyph name="sun" className="h-4 w-4 text-accent" />}
          {t("tasks")}
        </button>
        <span className="truncate text-[11.5px] text-faint">{todayLong()}</span>
        {counts.tasks ? (
          <span
            title={exactly(counts.tasks)}
            className="ml-auto shrink-0 text-[11.5px] text-faint tabular-nums"
          >
            {capped(counts.tasks)}
          </span>
        ) : null}
        <span className={counts.tasks ? "" : "ml-auto"}>
          <Leaning
            toward={wide ? "docs" : "tasks"}
            label={t(wide ? "shrinkTasks" : "growTasks")}
            onLean={onLean}
          />
        </span>
      </div>
      <div className="mt-1.5">
        <DayCounts
          counts={counts}
          onPick={(slice) => {
            keepOnly([]);
            onChoose({ named: "tasks", slice });
          }}
        />
      </div>
      <nav
        aria-label={t("tasksViews")}
        className="mt-2 grid grid-cols-6 rounded-[10px] bg-hover p-0.5"
      >
        {VIEWS.map(({ key, icon }) => {
          const said =
            key === "lists"
              ? `${t(key)} · ${lists.length}`
              : counts[key]
                ? `${t(key)} · ${counts[key]}`
                : t(key);
          return (
            <button
              key={key}
              type="button"
              onClick={() => onChoose({ named: key })}
              aria-label={said}
              aria-current={on(key) ? "page" : undefined}
              title={said}
              className={`grid h-7 place-items-center rounded-md transition-colors duration-150 ${
                on(key) ? "bg-sheet text-accent ring-1 ring-hair" : "text-faint hover:text-ink"
              }`}
            >
              <Glyph name={icon} />
            </button>
          );
        })}
        <button
          type="button"
          onClick={() => void sow().catch(() => {})}
          aria-label={t("captureNew")}
          title={t("captureNew")}
          className="grid h-7 place-items-center rounded-md text-accent transition-colors duration-150 hover:bg-sheet"
        >
          <Glyph name="plus" />
        </button>
      </nav>
      {wide && (
        <Wide
          counts={counts}
          lists={lists}
          tags={tags}
          chosen={chosen}
          acting={acting}
          onListMenu={onListMenu}
          onChoose={onChoose}
        />
      )}
    </div>
  );
}

function Wide({
  counts,
  lists,
  tags,
  chosen,
  acting,
  onListMenu,
  onChoose,
}: Omit<Props, "leaning" | "onLean" | "query" | "onSearch">) {
  const pane = useFades<HTMLDivElement>();
  const held = lists.filter((one) => !one.archived);
  const most = [...tags].sort((a, b) => b.tasks - a.tasks).slice(0, TAGS_SHOWN);
  return (
    <div ref={pane} className="scroller fading -mx-2.5 mt-1 min-h-0 flex-1 px-2.5 pb-2">
      <Head said={t("lists")} many={held.length} />
      {held.map((one) => {
        const picked = chosen.list === one.id;
        return (
          <button
            key={one.id}
            type="button"
            onClick={() => onChoose({ named: "lists", list: one.id })}
            onContextMenu={(e) => {
              if (!onListMenu) return;
              e.preventDefault();
              onListMenu(one, { x: e.clientX, y: e.clientY });
            }}
            onKeyDown={(e) => {
              if (!onListMenu || !asksForMenu(e)) return;
              e.preventDefault();
              onListMenu(one, menuAt(e));
            }}
            aria-current={picked ? "page" : undefined}
            className={`flex w-full items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left text-[13px] hover:bg-hover ${
              picked ? "bg-accent-soft font-semibold text-accent" : "text-soft hover:text-ink"
            } ${acting === one.id ? "bg-active" : ""}`}
          >
            <span className={`flex w-4 shrink-0 justify-center ${painted(one.color)}`}>
              {one.icon ? (
                <Glyph name={one.icon} className="h-[14px] w-[14px]" />
              ) : (
                <span aria-hidden="true" className="size-2 rounded-full bg-current" />
              )}
            </span>
            <span className="truncate">{one.name}</span>
            {counts[one.id] ? (
              <span
                title={exactly(counts[one.id])}
                className="ml-auto text-[11.5px] font-normal text-faint tabular-nums"
              >
                {capped(counts[one.id])}
              </span>
            ) : null}
          </button>
        );
      })}
      <Head said={t("tags")} many={tags.length} />
      <div className="flex flex-wrap gap-1 px-1.5">
        {most.map((one) => {
          const picked = chosen.tags?.includes(one.tag) ?? false;
          return (
            <button
              key={one.tag}
              type="button"
              onClick={() => onChoose({ named: "tags", tags: [one.tag] })}
              aria-pressed={picked}
              className={`flex items-baseline gap-1.5 rounded-full px-2.5 py-0.5 text-[11.5px] transition-colors duration-150 ${
                picked
                  ? "bg-accent-soft text-ink"
                  : "bg-hover text-soft hover:bg-active hover:text-ink"
              }`}
            >
              #{one.tag}
              {one.tasks ? (
                <span title={exactly(one.tasks)} className="text-faint tabular-nums">
                  {capped(one.tasks)}
                </span>
              ) : null}
            </button>
          );
        })}
        {tags.length > most.length && (
          <button
            type="button"
            onClick={() => onChoose({ named: "tags" })}
            className="rounded-full px-2.5 py-0.5 text-[11.5px] text-faint hover:text-ink"
          >
            {fill("tagsSeeAll", String(tags.length))}
          </button>
        )}
        {!tags.length && <p className="px-1 text-[11.5px] text-faint">{t("noTagsYet")}</p>}
      </div>
    </div>
  );
}

function Head({ said, many }: { said: string; many: number }) {
  return (
    <p className="flex items-center gap-2 px-2.5 pt-3 pb-1 text-[11.5px] font-semibold text-faint">
      {said}
      <span title={exactly(many)} className="font-normal text-faint tabular-nums">
        {many ? capped(many) : ""}
      </span>
    </p>
  );
}

export function Leaning({
  toward,
  label,
  onLean,
}: {
  toward: Lean;
  label: string;
  onLean: (on: Lean) => void;
}) {
  const opens = toward === "tasks";
  return (
    <button
      type="button"
      onClick={() => onLean(toward)}
      aria-label={label}
      title={label}
      className="grid size-6 shrink-0 place-items-center rounded-md text-faint hover:bg-hover hover:text-ink"
    >
      <Glyph
        name="chevron"
        className={`h-[13px] w-[13px] transition-transform duration-200 motion-reduce:transition-none ${
          opens ? "" : "rotate-180"
        }`}
      />
    </button>
  );
}

function Finder({ query, onSearch }: { query: string; onSearch: (query: string) => void }) {
  return (
    <form
      onSubmit={(e) => e.preventDefault()}
      className="mb-2 flex h-[30px] shrink-0 items-center gap-2 rounded-md bg-hover px-2.5 text-faint focus-within:bg-active"
    >
      <Glyph name="search" className="h-[13px] w-[13px]" />
      <input
        id={FINDER}
        value={query}
        placeholder={t("search")}
        aria-label={t("sideSearch")}
        onChange={(e) => onSearch(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Escape" && query) onSearch("");
        }}
        className="min-w-0 flex-1 bg-transparent text-[12.5px] text-ink outline-none placeholder:text-soft"
      />
      {query ? (
        <button
          type="button"
          onClick={() => onSearch("")}
          aria-label={t("sideSearchClear")}
          className="grid size-5 place-items-center rounded-md hover:bg-active hover:text-ink"
        >
          <Glyph name="close" className="h-3 w-3" />
        </button>
      ) : (
        <span aria-hidden="true" className="text-[11.5px] text-faint">
          {FIND_KEY}
        </span>
      )}
    </form>
  );
}
