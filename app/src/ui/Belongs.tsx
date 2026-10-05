import { useAsked } from "../asked";
import { type Task, type Whole, wholesOffered } from "../core";
import { fill, t } from "../locales";
import Hang from "./Hang";
import Parts from "./Parts";
import Section from "./Section";

interface Props {
  task: Task;
  whole?: Whole;
  partOf?: string;
  holding: boolean;
  onAddPart?: (title: string) => void;
  onHang?: (whole: string | null) => void;
  onOpenPart?: (id: string) => void;
  onCompletePart?: (id: string, title: string) => void;
  onError?: (problem: unknown) => void;
}

export default function Belongs({
  task,
  whole,
  partOf,
  holding,
  onAddPart,
  onHang,
  onOpenPart,
  onCompletePart,
  onError,
}: Props) {
  const hangs = Boolean(onHang && holding && !task.part_of && !whole);
  const offered =
    useAsked(
      () => (hangs ? wholesOffered(task.id) : Promise.resolve([])),
      [task.id, hangs],
      onError,
    ) ?? [];

  if (task.part_of) {
    const named = partOf ?? "…";
    return (
      <>
        <Section label={t("partOfHeading")} />
        <div className="flex items-center gap-2.5 py-1 text-[13px]">
          <button
            type="button"
            onClick={() => task.part_of && onOpenPart?.(task.part_of)}
            className="min-w-0 flex-1 truncate text-left text-accent hover:underline"
          >
            ⌂ {named}
          </button>
          {task.status === "open" && onHang && (
            <button
              type="button"
              aria-label={fill("letGoOf", named)}
              onClick={() => onHang(null)}
              className="shrink-0 text-[12.5px] text-faint hover:text-ink"
            >
              {t("letItGo")}
            </button>
          )}
        </div>
      </>
    );
  }

  if (!whole && !(holding && onAddPart)) return null;
  return (
    <>
      <Section
        label={t("parts")}
        note={whole ? `${whole.closed}/${whole.open + whole.closed}` : undefined}
      />
      <Parts
        task={task}
        whole={whole}
        onAdd={holding ? onAddPart : undefined}
        beside={
          hangs && onHang && offered.length > 0 ? (
            <Hang offered={offered} onHang={onHang} />
          ) : undefined
        }
        onOpen={(id) => onOpenPart?.(id)}
        onComplete={(id, title) => onCompletePart?.(id, title)}
        onError={onError}
      />
    </>
  );
}
