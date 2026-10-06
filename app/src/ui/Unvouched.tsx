import { useCallback, useEffect, useState } from "react";
import { type Unvouched as One, unvouchedAttachments } from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";

interface Props {
  held: boolean;
  vouch: (then: () => void) => void;
  className: string;
}

export default function Unvouched({ held, vouch, className }: Props) {
  const [seen, setSeen] = useState<One[] | null>(null);
  const look = useCallback(() => {
    unvouchedAttachments()
      .then(setSeen)
      .catch(() => setSeen(null));
  }, []);
  useEffect(look, [look]);

  const bytes = (seen ?? []).reduce((all, one) => all + one.bytes, 0);
  return (
    <>
      <p className="text-[12.5px] leading-relaxed text-soft">{t("unvouchedWhat")}</p>
      {seen?.length === 0 && <p className="mt-2 text-[12.5px] text-faint">{t("unvouchedNone")}</p>}
      {seen && seen.length > 0 && (
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
            <button type="button" disabled={held} onClick={() => vouch(look)} className={className}>
              {t("unvouchedDo")}
            </button>
          </div>
        </>
      )}
    </>
  );
}
