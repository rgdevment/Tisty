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
  onPick: (slice: Slice) => void;
}

export default function DayCounts({ counts, onPick }: Props) {
  const tiles = DAY.map(({ key, said, tone }) => ({
    key,
    said: t(said),
    tone,
    many: counts[key] ?? 0,
  }));
  const sliceOf = (key: string) => DAY.find((one) => one.key === key)?.slice ?? "today";
  return <Tiles tiles={tiles} onPick={(key) => onPick(sliceOf(key))} />;
}

export function Tiles({ tiles, onPick }: { tiles: Tile[]; onPick?: (key: string) => void }) {
  const strip = "grid grid-cols-4 rounded-[10px] bg-hover p-0.5";
  const cell = (at: number) =>
    `relative min-w-0 rounded-md px-1 py-1.5 text-center ${
      at ? "before:absolute before:inset-y-3 before:left-0 before:w-px before:bg-hair" : ""
    }`;
  const number = "block text-[21px] leading-tight font-semibold tabular-nums";
  const word = "block truncate text-[10.5px] leading-tight text-faint";

  if (!onPick) {
    return (
      <dl className={strip}>
        {tiles.map(({ key, said, tone, many }, at) => (
          <div key={key} className={cell(at)}>
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
    <div className={strip}>
      {tiles.map(({ key, said, tone, many }, at) => (
        <button
          key={key}
          type="button"
          onClick={() => onPick(key)}
          className={`${cell(at)} transition-colors duration-150 hover:bg-active`}
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
