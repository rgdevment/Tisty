import { useCallback, useEffect, useState } from "react";
import { type Repeated as Group, repeatedLists } from "../core";
import { fill, t } from "../locales";

interface Props {
  held: boolean;
  join: (then: () => void) => void;
  className: string;
}

export default function Repeated({ held, join, className }: Props) {
  const [seen, setSeen] = useState<Group[] | null>(null);
  const look = useCallback(() => {
    repeatedLists()
      .then(setSeen)
      .catch(() => setSeen(null));
  }, []);
  useEffect(look, [look]);

  return (
    <>
      <p className="text-[12.5px] leading-relaxed text-soft">{t("repeatedWhat")}</p>
      {seen?.length === 0 && <p className="mt-2 text-[12.5px] text-faint">{t("repeatedNone")}</p>}
      {seen && seen.length > 0 && (
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
            <button type="button" disabled={held} onClick={() => join(look)} className={className}>
              {t("repeatedDo")}
            </button>
          </div>
        </>
      )}
    </>
  );
}
