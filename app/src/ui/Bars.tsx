import type { Found, List, Snapshot } from "../core";
import { t } from "../locales";
import { type Chosen, LAYERS, layerCount, layerWord, SLICES, type Slice } from "../views";
import Axis from "./Axis";
import Cover from "./Cover";
import Only from "./Only";
import Tally from "./Tally";

const sliceWord = (slice: Slice) =>
  slice === "today"
    ? ("today" as const)
    : slice === "upcoming"
      ? ("upcoming" as const)
      : slice === "repeating"
        ? ("repeating" as const)
        : ("sliceAll" as const);

interface Steering {
  chosen: Chosen;
  counts: Snapshot["counts"];
  setChosen: (chosen: Chosen) => void;
  setSelected: (id: string | undefined) => void;
}

export function SliceBar({
  chosen,
  counts,
  lists,
  setChosen,
  setSelected,
}: Steering & { lists: List[] }) {
  return (
    <div className="flex gap-1 px-2.5 pb-1">
      {SLICES.map((slice) => {
        const on = (chosen.slice ?? "today") === slice;
        const many = counts[slice === "today" ? "tasks" : slice];
        return (
          <button
            key={slice}
            type="button"
            aria-pressed={on}
            onClick={() => {
              setSelected(undefined);
              window.localStorage.setItem("tisty.slice", slice);
              setChosen({ named: "tasks", slice, lists: chosen.lists });
            }}
            className={`rounded-full border px-2.5 py-0.5 text-[11.5px] ${
              on ? "border-ink bg-ink text-bg" : "border-line text-faint hover:text-soft"
            }`}
          >
            {t(sliceWord(slice))}
            {many ? <span className="ml-1 tabular-nums opacity-70">{many}</span> : null}
          </button>
        );
      })}
      <Only
        lists={lists}
        chosen={chosen.lists ?? []}
        onChange={(lists) => {
          setSelected(undefined);
          window.localStorage.setItem("tisty.only", JSON.stringify(lists));
          setChosen({ ...chosen, named: "tasks", lists });
        }}
      />
    </div>
  );
}

export function ArchiveBar({
  chosen,
  counts,
  found,
  setChosen,
  setSelected,
  setFound,
  fail,
}: Steering & {
  found: Found | null;
  setFound: (found: Found | null) => void;
  fail: (problem: unknown) => void;
}) {
  return (
    <>
      {found === null && !chosen.folded && <Cover onError={fail} />}
      {found === null && !chosen.folded && <Tally counts={counts} onError={fail} />}
      <fieldset className="flex flex-wrap items-center gap-1 px-2.5 pb-1">
        <legend className="sr-only">{t("archiveShowing")}</legend>
        {LAYERS.map((layer) => {
          const on = !chosen.folded && (chosen.layer ?? "story") === layer;
          const many = counts[layerCount(layer)];
          return (
            <button
              key={layer}
              type="button"
              aria-pressed={on}
              onClick={() => {
                setSelected(undefined);
                setFound(null);
                setChosen({ named: "archive", layer });
              }}
              className={`rounded-full border px-2.5 py-0.5 text-[11.5px] ${
                on ? "border-ink bg-ink text-bg" : "border-line text-faint hover:text-soft"
              }`}
            >
              {t(layerWord(layer))}
              {many ? <span className="ml-1 tabular-nums opacity-70">{many}</span> : null}
            </button>
          );
        })}
        {counts.folded || chosen.folded ? (
          <button
            type="button"
            aria-pressed={chosen.folded === true}
            title={chosen.folded ? t("backToArchive") : undefined}
            onClick={() => {
              setSelected(undefined);
              setFound(null);
              setChosen({ named: "archive", folded: !chosen.folded });
            }}
            className={`rounded-full border px-2.5 py-0.5 text-[11.5px] ${
              chosen.folded ? "border-ink bg-ink text-bg" : "border-line text-faint hover:text-soft"
            }`}
          >
            {t("hiddenOnes")}
            {counts.folded ? (
              <span className="ml-1 tabular-nums opacity-70">{counts.folded}</span>
            ) : null}
          </button>
        ) : null}
        {!chosen.folded && (chosen.layer ?? "story") !== "routine" && (
          <>
            <span className="mx-1.5 h-3.5 w-px bg-hair" />
            <Axis
              axis={chosen.axis ?? "time"}
              onChange={(axis) => {
                setSelected(undefined);
                setChosen({ ...chosen, named: "archive", axis, folded: false });
              }}
            />
          </>
        )}
      </fieldset>
    </>
  );
}
