import { useAsked } from "../asked";
import type { List, Page } from "../core";
import { taskStory } from "../core";
import { cadence, shortStamp, whenLabel, wroteAt } from "../format";
import { fill, t } from "../locales";
import { said } from "../quadrants";

interface Props {
  task: string;
  /// Anything that changes when the task's story grows — a status, a conversion, an entry
  /// count — so the trail asks again without the detail being closed and opened.
  moved?: string;
  heading?: React.ReactNode;
  lists: List[];
  onError?: (problem: unknown) => void;
  /// Drawn above the trail with the day the first page was written: the age of a task is only
  /// known here, and asking for the story twice to count it would read the log twice.
  before?: (from: string) => React.ReactNode;
  /// A settled task already shows its journal whole in the body, so here it is only counted.
  summed?: { count: number; last: string };
}

export default function Trail({ task, moved, lists, onError, heading, before, summed }: Props) {
  const told = useAsked(() => taskStory(task), [task, moved], onError);

  if (!told) return null;
  if (!told.pages.length) {
    return (
      <>
        {heading}
        <p className="px-1 text-[12.5px] text-faint">{t("trailEmpty")}</p>
      </>
    );
  }

  const named = (id?: string | null) => lists.find((one) => one.id === id)?.name;

  return (
    <>
      {before?.(told.pages[0].at)}
      {heading}
      <ol className="flex flex-col">
        {told.pages
          .filter((page) => !summed || (page.chapter !== "wrote" && page.chapter !== "rewrote"))
          .map((page) => (
            <li
              key={page.n}
              className="grid grid-cols-[76px_14px_minmax(0,1fr)] items-start gap-2.5 py-1.5"
            >
              <span
                title={wroteAt(page.at)}
                className="pt-px text-right text-[11.5px] whitespace-nowrap tabular-nums text-faint"
              >
                {shortStamp(page.at)}
              </span>
              <span className="text-center text-[11.5px] leading-5 text-faint">{glyph(page)}</span>
              <span
                className={`text-[12.5px] leading-relaxed ${page.undoing ? "text-faint" : "text-soft"}`}
              >
                {phrase(page, named)}
                {page.undoing && <span className="ml-1.5 text-[11.5px]">· {t("trailUndone")}</span>}
              </span>
            </li>
          ))}
      </ol>
      {summed && summed.count > 0 && (
        <p className="py-1.5 pl-[100px] text-[12.5px] leading-relaxed text-soft">
          {fill("trailJournalSummed", String(summed.count), summed.last)}
        </p>
      )}
    </>
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
    case "sectionwritten":
      return "✎";
    case "sectionadded":
    case "sectionrenamed":
    case "sectionremoved":
      return "§";
    case "joined":
    case "loosened":
    case "partjoined":
    case "partloosened":
      return "⊟";
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
    case "opened":
    case "shut":
      return "◆";
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
      return page.body.trim() ? t("trailRewrote") : t("trailErased");
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
    case "sectionadded":
      return fill("trailSectionAdded", page.section);
    case "sectionrenamed":
      return fill("trailSectionRenamed", page.from, page.to);
    case "sectionremoved":
      return fill("trailSectionRemoved", page.section);
    case "sectionwritten":
      return fill("trailSectionWritten", page.section);
    case "joined":
      return fill("trailJoined", page.title);
    case "loosened":
      return fill("trailLoosened", page.title);
    case "partjoined":
      return fill("trailPartJoined", page.title);
    case "partloosened":
      return fill("trailPartLoosened", page.title);
    case "closed":
      return t("trailClosed");
    case "dropped":
      return t("trailDropped");
    case "reopened":
      return t("trailReopened");
    case "converted":
      return page.to === "story"
        ? t("trailKeptStory")
        : page.to === "trace"
          ? t("trailReadTrace")
          : t("trailReadByItself");
    case "opened":
      return t("trailOpened");
    case "shut":
      return t("trailShut");
    default:
      return unreadable(page);
  }
}

function unreadable(page: never): string {
  void page;
  return "";
}
