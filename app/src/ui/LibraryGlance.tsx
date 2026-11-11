import { useMemo } from "react";
import type { Papers } from "../core";
import { capped, exactly } from "../format";
import { lately } from "../lately";
import { t } from "../locales";
import type { Lean } from "../views";
import { Tiles } from "./DayCounts";
import { Leaning } from "./SideTasks";

interface Props {
  papers: Papers;
  many: number;
  lit: boolean;
  onLean: (on: Lean) => void;
  onLibrary: () => void;
}

export default function LibraryGlance({ papers, many, lit, onLean, onLibrary }: Props) {
  const day = new Date().toDateString();
  const seen = useMemo(() => lately(papers, new Date(day)), [papers, day]);
  const tiles = [
    { key: "today", said: t("glanceToday"), many: seen.today, tone: "text-hue-amber" },
    { key: "week", said: t("glanceWeek"), many: seen.week },
    { key: "folders", said: t("glanceFolders"), many: seen.folders },
    {
      key: "marked",
      said: t("glanceMarked"),
      many: seen.marked,
      tone: seen.marked ? "text-urgent" : "",
    },
  ];

  return (
    <section aria-label={t("library")} className="shrink-0 px-2.5 pb-2">
      <span aria-hidden="true" className="mx-2.5 mt-2 mb-3 block h-px bg-hair" />
      <div className="flex items-center gap-2 pl-2.5">
        <button
          type="button"
          onClick={onLibrary}
          aria-current={lit ? "page" : undefined}
          className={`text-[12.5px] font-semibold hover:text-hue-amber ${
            lit ? "text-hue-amber" : "text-faint"
          }`}
        >
          {t("library")}
        </button>
        {many ? (
          <span
            title={exactly(many)}
            className="ml-auto shrink-0 text-[11.5px] text-faint tabular-nums"
          >
            {capped(many)}
          </span>
        ) : null}
        <span className={many ? "" : "ml-auto"}>
          <Leaning toward="docs" label={t("growLibrary")} onLean={onLean} />
        </span>
      </div>
      <div className="mt-1.5">
        <Tiles tiles={tiles} />
      </div>
    </section>
  );
}
