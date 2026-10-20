import type { Repeated as Group } from "../core";
import { fill, t } from "../locales";

interface Props {
  seen: Group[];
  held: boolean;
  join: () => void;
  className: string;
}

export default function Repeated({ seen, held, join, className }: Props) {
  return (
    <>
      <p className="text-[12.5px] leading-relaxed text-soft">{t("repeatedWhat")}</p>
      {seen.length === 0 && <p className="mt-2 text-[12.5px] text-faint">{t("repeatedNone")}</p>}
      {seen.length > 0 && (
        <>
          <ul className="mt-2 flex flex-col gap-1 text-[12.5px]">
            {seen.map((one) => (
              <li key={one.name} className="flex items-baseline justify-between gap-4">
                <span className="text-ink">{one.name}</span>
                <span className="tabular-nums text-faint">
                  {fill("repeatedRow", String(one.lists), String(one.tasks))}
                </span>
              </li>
            ))}
          </ul>
          <div className="mt-2.5 flex items-center gap-2.5">
            <button type="button" disabled={held} onClick={join} className={className}>
              {t("repeatedDo")}
            </button>
          </div>
        </>
      )}
    </>
  );
}
