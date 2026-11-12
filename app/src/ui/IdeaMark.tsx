import type { Filed, IdeaMarked, Marked, Task } from "../core";
import { stamped } from "../format";
import { fill, t, type Word } from "../locales";
import type { Note } from "../notes";
import { clientNamed } from "../who";
import { Reason, reasoned } from "./Tidied";

const MILD = "rounded-[10px] border border-line px-2 py-0.5 text-[11.5px] hover:bg-hover";
const RISKY = `${MILD} text-urgent`;

const PLAIN: Record<string, Word> = {
  wrong: "ideaWrong",
  obsolete: "ideaObsolete",
  scratch: "flagScratch",
};

export const markOf = (marks: IdeaMarked[], note: Note): IdeaMarked | undefined =>
  marks.find((one) => one.heading === note.heading && one.text === note.text);

export const markedBy = (mark: Marked): string => {
  const named = clientNamed(mark.via);
  return named ? fill("noteMarkedBy", named) : t("noteMarked");
};

const sharedClient = (marks: IdeaMarked[]): string | null => {
  const named = marks.map((one) => clientNamed(one.mark.via));
  return named.every((one) => one && one === named[0]) ? (named[0] as string) : null;
};

const countSaid = (marks: IdeaMarked[]): string => {
  const named = sharedClient(marks);
  if (marks.length === 1) return named ? fill("ideaMarkedBy", named) : t("ideaMarked");
  const many = String(marks.length);
  return named ? fill("ideasMarkedBy", named, many) : fill("ideasMarked", many);
};

interface BarProps {
  marks: IdeaMarked[];
  reading: boolean;
  onDropAll: () => void;
  onUnmarkAll: () => void;
}

export function IdeaMarkBar({ marks, reading, onDropAll, onUnmarkAll }: Readonly<BarProps>) {
  const many = String(marks.length);
  const latest = marks.map((one) => one.mark.at).sort((one, other) => one.localeCompare(other))[
    marks.length - 1
  ];
  return (
    <div className="mt-3 flex flex-wrap items-center gap-x-3 gap-y-1 text-[11.5px]">
      <span className="font-medium text-hue-teal">
        <span aria-hidden="true">◆ </span>
        {countSaid(marks)}
      </span>
      {latest && <span className="text-faint">{stamped(latest)}</span>}
      {!reading && (
        <>
          <button type="button" onClick={onDropAll} className={RISKY}>
            {marks.length === 1 ? t("ideaDropIt") : fill("ideasDropAll", many)}
          </button>
          <button type="button" onClick={onUnmarkAll} className={MILD}>
            {marks.length === 1 ? t("unflagIt") : fill("flagUnmarkAll", many)}
          </button>
        </>
      )}
    </div>
  );
}

interface Reached {
  notes: Note[];
  known: Filed[];
  onDoc?: (file: string) => void;
  onTask?: (task: Task) => void;
}

const plainOfIdea = (reason?: string | null): Word | undefined =>
  reason ? PLAIN[reason] : undefined;

// A mark that points at another idea names it by its words, which only the sheet knows.
const ideaNamed = (mark: Marked, notes: Note[]): string | null | undefined => {
  const cut = mark.instead?.indexOf("#") ?? -1;
  if (cut < 0) return undefined;
  const heading = mark.instead?.slice(cut + 1).trim();
  return notes.find((one) => one.heading === heading)?.text ?? null;
};

function IdeaReason({ mark, notes, ...reached }: Readonly<{ mark: Marked } & Reached>) {
  if (!reasoned(mark, plainOfIdea)) return markedBy(mark);
  return <Reason mark={mark} plainOf={plainOfIdea} named={ideaNamed(mark, notes)} {...reached} />;
}

interface StripProps extends Reached {
  mark: IdeaMarked;
  reading: boolean;
  onDrop: () => void;
  onUnmark: () => void;
}

export function IdeaMarkStrip({
  mark,
  reading,
  onDrop,
  onUnmark,
  ...reached
}: Readonly<StripProps>) {
  return (
    <div className="sticky-mark">
      <p className="sticky-mark-why">
        <span aria-hidden="true">◆ </span>
        <IdeaReason mark={mark.mark} {...reached} />
      </p>
      {mark.mark.said && <p className="sticky-mark-said">{mark.mark.said}</p>}
      {!reading && (
        <span className="sticky-mark-tools">
          <button type="button" onClick={onDrop} className="text-urgent">
            {t("ideaDropIt")}
          </button>
          <button type="button" onClick={onUnmark}>
            {t("unflagIt")}
          </button>
        </span>
      )}
    </div>
  );
}
