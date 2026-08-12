import { useState } from "react";
import type { DateSpec } from "../core";
import { t } from "../locales";
import When from "./When";

interface Props {
  on?: DateSpec | null;
  taken: string[];
  onAdd: (at: string) => void;
  onClose: () => void;
}

const AHEAD = [60, 30, 15, 0] as const;
const SAID = {
  60: "anHourBefore",
  30: "halfHourBefore",
  15: "quarterBefore",
  0: "onTheDot",
} as const;
const OPENS = "09:00";

export default function Recall({ on, taken, onAdd, onClose }: Props) {
  const [day, setDay] = useState<string | null>(null);
  const worth = (at: string) => new Date(at) > new Date() && !taken.includes(at);
  // «At the time» only where there is a time to be at: on an all-day task it
  // would name the nine o'clock this file invents, which is not what it says.
  const offsets = on?.has_time ? AHEAD : AHEAD.filter((m) => m !== 0);
  const ahead = on
    ? offsets.map((m) => [m, before(on, m)] as const).filter(([, at]) => worth(at))
    : [];

  if (day !== null) {
    return (
      <When
        never
        confirm={t("addReminder")}
        value={day || undefined}
        clock={OPENS}
        onPick={(at) => {
          const when = at.length > 10 ? at : `${at}T${OPENS}:00`;
          if (!taken.includes(when) && new Date(when) > new Date()) onAdd(when);
          else onClose();
        }}
        onClear={onClose}
        onClose={onClose}
      />
    );
  }

  return (
    <>
      {ahead.map(([minutes, at]) => (
        <Row key={minutes} onPick={() => onAdd(at)}>
          {t(SAID[minutes])}
        </Row>
      ))}
      <Row onPick={() => setDay(on?.at.slice(0, 10) ?? "")}>{t("pickIt")}</Row>
    </>
  );
}

function Row({ children, onPick }: { children: React.ReactNode; onPick: () => void }) {
  return (
    <button
      type="button"
      onClick={onPick}
      className="block w-full rounded-md px-2.5 py-1.5 text-left text-ink hover:bg-hover"
    >
      {children}
    </button>
  );
}

function before(spec: DateSpec, minutes: number): string {
  const base = spec.has_time ? spec.at : `${spec.at.slice(0, 10)}T${OPENS}:00`;
  const at = new Date(base);
  at.setMinutes(at.getMinutes() - minutes);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${at.getFullYear()}-${pad(at.getMonth() + 1)}-${pad(at.getDate())}T${pad(at.getHours())}:${pad(at.getMinutes())}:00`;
}
