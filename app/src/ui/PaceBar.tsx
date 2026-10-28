import type { Pace, Paced } from "../deadlines";
import { countedAs, percentOf } from "../format";
import { t, type Word } from "../locales";

const SAID: Record<Pace, { word: Word; tone: string }> = {
  on_track: { word: "paceOnTrack", tone: "bg-accent" },
  slow: { word: "paceSlow", tone: "bg-hue-amber" },
  behind: { word: "paceBehind", tone: "bg-urgent" },
};

export const paceSaid = (paced: Paced): string =>
  `${t(SAID[paced.pace].word)} · ${percentOf(paced.time)} ${t("paceTime")} · ${percentOf(paced.done)} ${t("paceDone")}`;

// Said on the folded band itself, so a task falling behind is seen without unfolding what is underway.
export function PaceTally({ paces }: { paces: (Paced | undefined)[] }) {
  const many = (pace: Pace) => paces.filter((one) => one?.pace === pace).length;
  const told = [
    {
      tone: "bg-urgent/10 text-urgent",
      said: countedAs(many("behind"), "paceBehindOne", "paceBehindMany"),
    },
    {
      tone: "bg-hue-amber/10 text-hue-amber",
      said: countedAs(many("slow"), "paceSlowOne", "paceSlowMany"),
    },
  ].filter((one) => one.said);
  return told.map(({ tone, said }) => (
    <span
      key={said}
      className={`rounded-md px-1.5 text-[10.5px] font-normal tracking-normal normal-case ${tone}`}
    >
      {said}
    </span>
  ));
}

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
