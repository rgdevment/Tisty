import { capped, exactly } from "../format";
import { t } from "../locales";
import type { Slice } from "../views";

export interface Tile {
  key: string;
  said: string;
  many: number;
  tone?: string;
}

const DAY: { key: string; said: Parameters<typeof t>[0]; tone: string; slice: Slice }[] = [
  { key: "overdue", said: "pulseOverdue", tone: "text-urgent", slice: "overdue" },
  { key: "late", said: "pulseLate", tone: "text-hue-amber", slice: "late" },
  { key: "dueToday", said: "pulseToday", tone: "text-accent", slice: "today" },
  { key: "upcoming", said: "pulseAhead", tone: "", slice: "upcoming" },
];

interface Props {
  counts: Record<string, number>;
  onPick?: (slice: Slice) => void;
}

export default function DayCounts({ counts, onPick }: Props) {
  const tiles = DAY.map(({ key, said, tone }) => ({
    key,
    said: t(said),
    tone,
    many: counts[key] ?? 0,
  }));
  const sliceOf = (key: string) => DAY.find((one) => one.key === key)?.slice ?? "today";
  return <Tiles tiles={tiles} onPick={onPick && ((key) => onPick(sliceOf(key)))} />;
}

export function Tiles({ tiles, onPick }: { tiles: Tile[]; onPick?: (key: string) => void }) {
  const box = "min-w-0 rounded-[10px] border border-hair px-2 py-1 text-left";
  const number = "block text-[21px] leading-tight font-semibold tabular-nums";
  const word = "block truncate text-[9px] leading-tight text-faint";

  if (!onPick) {
    return (
      <dl className="grid grid-cols-4 gap-1.5">
        {tiles.map(({ key, said, tone, many }) => (
          <div key={key} className={box}>
            <dt title={exactly(many)} className={`${number} ${tone ?? ""}`}>
              {capped(many)}
            </dt>
            <dd className={word}>{said}</dd>
          </div>
        ))}
      </dl>
    );
  }

  return (
    <div className="grid grid-cols-4 gap-1.5">
      {tiles.map(({ key, said, tone, many }) => (
        <button
          key={key}
          type="button"
          onClick={() => onPick(key)}
          className={`${box} transition-colors duration-150 hover:bg-hover`}
        >
          <span title={exactly(many)} className={`${number} ${tone ?? ""}`}>
            {capped(many)}
          </span>
          <span className={word}>{said}</span>
        </button>
      ))}
    </div>
  );
}
