import type { Counted, List } from "../core";
import { todayLong } from "../format";
import { fill, t } from "../locales";
import type { Chosen, Lean, Named } from "../views";
import DayCounts from "./DayCounts";
import { FIND_KEY, FINDER } from "./finding";
import Glyph from "./Glyph";
import { painted } from "./Hue";
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
  leaning: Lean;
  onLean: (on: Lean) => void;
  onChoose: (chosen: Chosen) => void;
  onSearch: (query: string) => void;
}

export default function SideTasks({
  counts,
  lists,
  tags,
  chosen,
  leaning,
  onLean,
  onChoose,
  onSearch,
}: Props) {
  const wide = leaning === "tasks";
  const plain = !chosen.list && !chosen.tags?.length;
  const on = (key: Named) => plain && chosen.named === key;
  const query = chosen.named === "search" ? (chosen.query ?? "") : "";

  return (
    <div className={`flex flex-col px-2.5 ${wide ? "min-h-0 flex-1" : "shrink-0 pb-2"}`}>
      <Finder query={query} onSearch={onSearch} />
      <div
        className={`rounded-[10px] border bg-sheet px-2 pt-2 pb-1 transition-colors duration-150 ${
          on("tasks") ? "border-accent/40" : "border-hair"
        }`}
      >
        <div className="flex items-center gap-2 pl-1.5">
          <button
            type="button"
            onClick={() => onChoose({ named: "tasks" })}
            aria-current={on("tasks") ? "page" : undefined}
            className={`text-[21px] leading-tight font-semibold hover:text-accent ${on("tasks") ? "text-accent" : "text-ink"}`}
          >
            {t("tasks")}
          </button>
          <span className="truncate text-[11.5px] text-faint">{todayLong()}</span>
          {counts.tasks ? (
            <span className="ml-auto shrink-0 text-[11.5px] text-faint tabular-nums">
              {counts.tasks}
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
        <div className="mt-2">
          <DayCounts counts={counts} onPick={(slice) => onChoose({ named: "tasks", slice })} />
        </div>
        <nav aria-label={t("tasksViews")} className="mt-2 flex gap-0.5 border-t border-hair pt-1">
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
                className={`grid h-8 flex-1 place-items-center rounded-md transition-colors duration-150 hover:bg-hover ${
                  on(key) ? "bg-active text-accent" : "text-faint hover:text-ink"
                }`}
              >
                <Glyph name={icon} />
              </button>
            );
          })}
        </nav>
      </div>
      {wide && (
        <Wide counts={counts} lists={lists} tags={tags} chosen={chosen} onChoose={onChoose} />
      )}
    </div>
  );
}

function Wide({
  counts,
  lists,
  tags,
  chosen,
  onChoose,
}: Omit<Props, "leaning" | "onLean" | "onSearch">) {
  const pane = useFades<HTMLDivElement>();
  const held = lists.filter((one) => !one.archived);
  const most = [...tags].sort((a, b) => b.tasks - a.tasks).slice(0, TAGS_SHOWN);
  return (
    <div ref={pane} className="scroller fading -mx-2.5 min-h-0 flex-1 px-2.5 pt-1 pb-3">
      <Head said={t("lists")} many={held.length} />
      {held.map((one) => {
        const picked = chosen.list === one.id;
        return (
          <button
            key={one.id}
            type="button"
            onClick={() => onChoose({ named: "lists", list: one.id })}
            aria-current={picked ? "page" : undefined}
            className={`flex w-full items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left text-[13px] hover:bg-hover ${
              picked ? "bg-active font-semibold text-ink" : "text-soft hover:text-ink"
            }`}
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
              <span className="ml-auto text-[11.5px] font-normal text-faint tabular-nums">
                {counts[one.id]}
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
              className={`flex items-baseline gap-1.5 rounded-md px-2 py-0.5 text-[11.5px] transition-colors duration-150 ${
                picked
                  ? "bg-accent-soft text-ink"
                  : "bg-hover text-soft hover:bg-active hover:text-ink"
              }`}
            >
              #{one.tag}
              {one.tasks ? <span className="text-faint tabular-nums">{one.tasks}</span> : null}
            </button>
          );
        })}
        {tags.length > most.length && (
          <button
            type="button"
            onClick={() => onChoose({ named: "tags" })}
            className="rounded-md px-2 py-0.5 text-[11.5px] text-faint hover:text-ink"
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
    <p className="flex items-center gap-2 px-2.5 pt-3 pb-1 text-[11.5px] font-semibold text-soft">
      {said}
      <span className="font-normal text-faint tabular-nums">{many || ""}</span>
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
