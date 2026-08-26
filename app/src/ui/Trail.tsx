import { useEffect, useState } from "react";
import type { List, Page, Story } from "../core";
import { taskStory } from "../core";
import { cadence, whenLabel, wroteAt } from "../format";
import { fill, t } from "../locales";
import { said } from "../quadrants";

interface Props {
  task: string;
  lists: List[];
  onError?: (problem: unknown) => void;
}

export default function Trail({ task, lists, onError }: Props) {
  const [told, setTold] = useState<Story | null>(null);

  useEffect(() => {
    let alive = true;
    setTold(null);
    taskStory(task)
      .then((story) => {
        if (alive) setTold(story);
      })
      .catch((problem) => onError?.(problem));
    return () => {
      alive = false;
    };
  }, [task, onError]);

  if (!told) return null;
  if (!told.pages.length) {
    return <p className="px-1 text-[12.5px] text-faint">{t("trailEmpty")}</p>;
  }

  const named = (id?: string | null) => lists.find((one) => one.id === id)?.name;

  return (
    <ol className="flex flex-col">
      {told.pages.map((page) => (
        <li
          key={page.n}
          className="relative grid grid-cols-[86px_14px_minmax(0,1fr)] items-start gap-2.5 py-1.5"
        >
          <span
            aria-hidden="true"
            className="absolute top-0 bottom-0 left-[92px] w-px bg-hair last:h-3"
          />
          <span className="pt-px text-right text-[11.5px] tabular-nums text-faint">
            {wroteAt(page.at)}
          </span>
          <span className="relative z-10 bg-bg text-center text-[11px] leading-5 text-faint">
            {glyph(page)}
          </span>
          <span
            className={`text-[12.5px] leading-relaxed ${page.undoing ? "text-faint" : "text-soft"}`}
          >
            {phrase(page, named)}
            {page.undoing && <span className="ml-1.5 text-[11px]">· {t("trailUndone")}</span>}
            {page.chapter === "wrote" && (
              <q className="mt-1 block border-l-2 border-hair pl-2.5 text-soft italic">
                {page.body}
              </q>
            )}
          </span>
        </li>
      ))}
    </ol>
  );
}

function glyph(page: Page): string {
  switch (page.chapter) {
    case "born":
      return "◦";
    case "dated":
    case "bounded":
      return "⚑";
    case "placed":
      return "⊞";
    case "filed":
      return "▤";
    case "tagged":
      return "◈";
    case "cadenced":
      return "↻";
    case "described":
    case "wrote":
    case "rewrote":
      return "✎";
    case "planned":
    case "unplanned":
    case "reworded":
      return "▫";
    case "ticked":
      return "✓";
    case "unticked":
      return "▫";
    case "closed":
      return "▣";
    case "dropped":
      return "⨯";
    case "reopened":
      return "⊕";
    default:
      return "·";
  }
}

function phrase(page: Page, named: (id?: string | null) => string | undefined): string {
  switch (page.chapter) {
    case "born":
      return fill("trailBorn", page.title);
    case "retitled":
      return fill("trailRetitled", page.to);
    case "dated":
      return page.to ? fill("trailDated", whenLabel(page.to)) : t("trailUndated");
    case "bounded":
      if (!page.to) return t("trailUnbounded");
      return page.from
        ? fill("trailRebounded", whenLabel(page.to))
        : fill("trailBounded", whenLabel(page.to));
    case "placed":
      return fill("trailPlaced", said(page.to));
    case "filed":
      return page.to ? fill("trailFiled", named(page.to) ?? "") : t("trailUnfiled");
    case "tagged":
      return page.added.length
        ? fill("trailTagged", page.added.map((one) => `#${one}`).join(" "))
        : fill("trailUntagged", page.gone.map((one) => `#${one}`).join(" "));
    case "cadenced":
      return page.to ? fill("trailCadenced", cadence(page.to)) : t("trailUncadenced");
    case "described":
      return page.emptied ? t("trailUndescribed") : t("trailDescribed");
    case "wrote":
      return t("trailWrote");
    case "rewrote":
      return t("trailRewrote");
    case "planned":
      return fill("trailPlanned", page.text);
    case "ticked":
      return fill("trailTicked", page.text);
    case "unticked":
      return fill("trailUnticked", page.text);
    case "reworded":
      return fill("trailReworded", page.to);
    case "unplanned":
      return fill("trailUnplanned", page.text);
    case "closed":
      return t("trailClosed");
    case "dropped":
      return t("trailDropped");
    default:
      return t("trailReopened");
  }
}
