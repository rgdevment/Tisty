import type { Counted } from "../core";

interface Props {
  tags: Counted[];
  chosen: string[];
  onToggle: (tag: string) => void;
}

export default function Tags({ tags, chosen, onToggle }: Props) {
  if (tags.length === 0) return null;

  return (
    <div className="scroller flex max-h-[38vh] flex-wrap content-start gap-2 px-2.5 pb-4">
      {tags.map(({ tag, tasks, docs }) => {
        const on = chosen.includes(tag);
        return (
          <button
            type="button"
            key={tag}
            onClick={() => onToggle(tag)}
            className={`flex items-center gap-2 rounded-lg px-3 py-1.5 text-[13px] ${
              on ? "bg-mark-tag text-ink" : "bg-hover text-soft hover:text-ink"
            }`}
          >
            #{tag}
            <span className="text-xs text-faint tabular-nums">
              {docs ? `${tasks} · ${docs}` : tasks}
            </span>
          </button>
        );
      })}
    </div>
  );
}
