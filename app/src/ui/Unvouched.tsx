import { useCallback, useEffect, useState } from "react";
import { type Unvouching, unvouchedAttachments } from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";

interface Props {
  held: boolean;
  vouch: (then: () => void) => void;
  className: string;
}

export default function Unvouched({ held, vouch, className }: Props) {
  const [told, setTold] = useState<Unvouching | null>(null);
  const look = useCallback(() => {
    unvouchedAttachments()
      .then(setTold)
      .catch(() => setTold(null));
  }, []);
  useEffect(look, [look]);

  const seen = told?.found;
  const bytes = (seen ?? []).reduce((all, one) => all + one.bytes, 0);
  return (
    <>
      <p className="text-[12.5px] leading-relaxed text-soft">{t("unvouchedWhat")}</p>
      {told?.unread != null && (
        <p className="mt-2 text-[12.5px] text-soft">
          {fill("unvouchedUnread", told.unread || t("untitledDoc"))}
        </p>
      )}
      {seen?.length === 0 && told?.unread == null && (
        <p className="mt-2 text-[12.5px] text-faint">{t("unvouchedNone")}</p>
      )}
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
