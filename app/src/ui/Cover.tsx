import { useAsked } from "../asked";
import { archiveShape } from "../core";
import { fill, locale, t } from "../locales";

interface Props {
  onError?: (problem: unknown) => void;
}

export default function Cover({ onError }: Props) {
  const shape = useAsked(() => archiveShape(), [], onError);

  if (!shape || shape.closed === 0) return null;

  const most = Math.max(...shape.months.map((one) => one.closed), 1);
  const last = shape.months.length - 1;
  const peak = shape.months.reduce(
    (best, one, at) => (one.closed > shape.months[best].closed ? at : best),
    0,
  );

  return (
    <section aria-label={t("coverTitle")} className="px-2.5 pt-1 pb-2">
      <p className="text-[12px] text-faint">
        {shape.since && (
          <>
            {fill("coverSince", month(shape.since))}
            {" · "}
          </>
        )}
        {shape.told === 1 ? t("coverToldOne") : fill("coverTold", String(shape.told))}
        {shape.dropped > 0 &&
          ` · ${shape.dropped === 1 ? t("coverDroppedOne") : fill("coverDropped", String(shape.dropped))}`}
      </p>

      {shape.months.length > 1 && (
        <>
          <div
            className="mt-2.5 flex h-9 items-end gap-[2px] border-b border-hair"
            role="img"
            aria-label={fill("coverStrip", String(shape.months.length))}
          >
            {shape.months.map((one, at) => (
              <span
                key={one.key}
                title={`${named(one.key)} · ${one.closed}`}
                style={{ height: `${Math.max((one.closed / most) * 100, one.closed ? 6 : 2)}%` }}
                className={`flex-1 rounded-t-[3px] bg-accent ${
                  at === last ? "opacity-100" : "opacity-40"
                }`}
              />
            ))}
          </div>
          <div className="mt-1 flex justify-between text-[11px] tabular-nums text-faint">
            <span>{named(shape.months[0].key)}</span>
            <span>
              {fill("coverPeak", `${shape.months[peak].closed} · ${named(shape.months[peak].key)}`)}
            </span>
            <span>{named(shape.months[last].key)}</span>
          </div>
        </>
      )}
    </section>
  );
}

const named = (key: string): string => {
  const at = new Date(`${key}-01T12:00:00`);
  if (Number.isNaN(at.getTime())) return key;
  return new Intl.DateTimeFormat(locale(), { month: "short", year: "2-digit" }).format(at);
};

const month = (iso: string): string => {
  const at = new Date(iso.includes("T") ? iso : `${iso}T12:00:00`);
  if (Number.isNaN(at.getTime())) return "";
  return new Intl.DateTimeFormat(locale(), { month: "long", year: "numeric" }).format(at);
};
