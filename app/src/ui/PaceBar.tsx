import type { Pace, Paced } from "../deadlines";
import { percentOf } from "../format";
import { t, type Word } from "../locales";

const SAID: Record<Pace, { word: Word; tone: string }> = {
  on_track: { word: "paceOnTrack", tone: "bg-accent" },
  slow: { word: "paceSlow", tone: "bg-hue-amber" },
  behind: { word: "paceBehind", tone: "bg-urgent" },
};

export const paceSaid = (paced: Paced): string =>
  `${t(SAID[paced.pace].word)} · ${percentOf(paced.time)} ${t("paceTime")} · ${percentOf(paced.done)} ${t("paceDone")}`;

export default function PaceBar({ paced, wide = false }: { paced: Paced; wide?: boolean }) {
  const { tone } = SAID[paced.pace];
  const said = paceSaid(paced);
  return (
    <div
      role="img"
      aria-label={said}
      title={said}
      className={`relative mt-1 h-1 rounded-full bg-line ${wide ? "w-full" : "w-full max-w-[220px]"}`}
    >
      <div className={`h-full rounded-full ${tone}`} style={{ width: `${paced.done}%` }} />
      <div
        aria-hidden="true"
        className="absolute -top-0.5 h-2 w-px bg-ink"
        style={{ left: `${paced.time}%` }}
      />
    </div>
  );
}
