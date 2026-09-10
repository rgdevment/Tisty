import type { Counted } from "../core";

interface Props {
  tags: Counted[];
  chosen: string[];
  onToggle: (tag: string) => void;
}

export default function Tags({ tags, chosen, onToggle }: Props) {
  if (tags.length === 0) return null;

  return (
    <div className="scroller flex max-h-[34vh] flex-wrap content-start gap-2 rounded-[10px] border border-hair bg-sheet p-2.5 shadow-lift">
      {tags.map(({ tag, tasks, docs }) => {
        const on = chosen.includes(tag);
        return (
          <button
            type="button"
            key={tag}
            onClick={() => onToggle(tag)}
            className={`flex items-center gap-2 rounded-md px-2.5 py-1 text-[12.5px] ${
              on ? "bg-mark-tag text-ink" : "bg-hover text-soft hover:text-ink"
            }`}
          >
            #{tag}
            <span className="text-[10.5px] text-faint tabular-nums">
              {docs ? `${tasks} · ${docs}` : tasks}
            </span>
          </button>
        );
      })}
    </div>
  );
}
