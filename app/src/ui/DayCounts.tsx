import { t } from "../locales";
import type { Slice } from "../views";

const DAY: { key: string; said: Parameters<typeof t>[0]; tone: string; slice: Slice }[] = [
  { key: "overdue", said: "pulseOverdue", tone: "text-urgent", slice: "today" },
  { key: "late", said: "pulseLate", tone: "text-hue-amber", slice: "today" },
  { key: "dueToday", said: "pulseToday", tone: "text-accent", slice: "today" },
  { key: "upcoming", said: "pulseAhead", tone: "", slice: "upcoming" },
];

interface Props {
  counts: Record<string, number>;
  onPick?: (slice: Slice) => void;
}

export default function DayCounts({ counts, onPick }: Props) {
  const box = "min-w-0 rounded-[10px] border border-hair px-2 py-1 text-left";
  const number = "block text-[21px] leading-tight font-semibold tabular-nums";
  const word = "block truncate text-[9px] leading-tight text-faint";

  if (!onPick) {
    return (
      <dl className="grid grid-cols-4 gap-1.5">
        {DAY.map(({ key, said, tone }) => (
          <div key={key} className={box}>
            <dt className={`${number} ${tone}`}>{counts[key] ?? 0}</dt>
            <dd className={word}>{t(said)}</dd>
          </div>
        ))}
      </dl>
    );
  }

  return (
    <div className="grid grid-cols-4 gap-1.5">
      {DAY.map(({ key, said, tone, slice }) => (
        <button
          key={key}
          type="button"
          onClick={() => onPick(slice)}
          className={`${box} transition-colors duration-150 hover:bg-hover`}
        >
          <span className={`${number} ${tone}`}>{counts[key] ?? 0}</span>
          <span className={word}>{t(said)}</span>
        </button>
      ))}
    </div>
  );
}
