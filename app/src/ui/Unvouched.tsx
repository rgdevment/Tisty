import type { Unvouched as Seen } from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";

interface Props {
  seen: Seen[];
  held: boolean;
  vouch: () => void;
  className: string;
  label?: string;
}

export default function Unvouched({ seen, held, vouch, className, label }: Props) {
  const bytes = seen.reduce((all, one) => all + one.bytes, 0);
  return (
    <>
      <p className="text-[12.5px] leading-relaxed text-soft">{t("unvouchedWhat")}</p>
      {seen.length === 0 && <p className="mt-2 text-[12.5px] text-faint">{t("unvouchedNone")}</p>}
      {seen.length > 0 && (
        <>
          <p className="mt-2 text-[12.5px] tabular-nums text-soft">
            {fill("unvouchedCount", String(seen.length), weigh(bytes))}
          </p>
          <ul className="scroller mt-2 flex max-h-[14rem] flex-col gap-1 overflow-y-auto text-[12.5px]">
            {seen.map((one) => (
              <li key={one.at} className="flex items-baseline justify-between gap-4">
                <span className="font-mono text-[11.5px] break-all text-soft">
                  {one.at.split("/").pop()}
                </span>
                <span className="shrink-0 tabular-nums text-faint">{weigh(one.bytes)}</span>
              </li>
            ))}
          </ul>
          <div className="mt-2.5 flex items-center gap-2.5">
            <button type="button" disabled={held} onClick={vouch} className={className}>
              {label ?? t("unvouchedDo")}
            </button>
          </div>
        </>
      )}
    </>
  );
}
