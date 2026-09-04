import { useAsked } from "../asked";
import type { List, Page } from "../core";
import { taskStory } from "../core";
import { cadence, shortStamp, whenLabel, wroteAt } from "../format";
import { fill, t } from "../locales";
import { said } from "../quadrants";

interface Props {
  task: string;
  heading?: React.ReactNode;
  lists: List[];
  onError?: (problem: unknown) => void;
  /// Drawn above the trail with the day the first page was written: the age of a task is only
  /// known here, and asking for the story twice to count it would read the log twice.
  before?: (from: string) => React.ReactNode;
}

export default function Trail({ task, lists, onError, heading, before }: Props) {
  const told = useAsked(() => taskStory(task), [task], onError);

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
        {told.pages.map((page) => (
          <li
            key={page.n}
            className="grid grid-cols-[76px_14px_minmax(0,1fr)] items-start gap-2.5 py-1.5"
          >
            <span
              title={wroteAt(page.at)}
              className="pt-px text-right text-[11px] whitespace-nowrap tabular-nums text-faint"
            >
              {shortStamp(page.at)}
            </span>
            <span className="text-center text-[11px] leading-5 text-faint">{glyph(page)}</span>
            <span
              className={`text-[12.5px] leading-relaxed ${page.undoing ? "text-faint" : "text-soft"}`}
            >
              {phrase(page, named)}
              {page.undoing && <span className="ml-1.5 text-[11px]">· {t("trailUndone")}</span>}
              {page.chapter === "wrote" && (
                <q className="mt-1 block border-l-2 border-hair pl-2.5 text-soft italic">
                  {plainly(page.body ?? "")}
                </q>
              )}
            </span>
          </li>
        ))}
      </ol>
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
    case "reopened":
      return t("trailReopened");
    default:
      return unreadable(page);
  }
}

function unreadable(page: never): string {
  void page;
  return "";
}

/// The trail is a record, not a page: a reference reads as what it was called, not as its syntax.
function plainly(body: string): string {
  return body
    .replace(/<span[^>]*data-ico="([a-z0-9-]+)"[^>]*>[\s\S]*?<\/span>/g, ":$1:")
    .replace(/!?\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/<[^>]+>/g, "")
    .trim();
}
