import { useMemo } from "react";
import { fields, overgrown } from "../front";
import { t } from "../locales";

interface Props {
  body: string;
  onEdit?: () => void;
  skip?: readonly string[];
}

export default function Properties({ body, onEdit, skip = [] }: Readonly<Props>) {
  const found = useMemo(() => fields(body), [body]);
  const over = useMemo(() => overgrown(body), [body]);
  if (found.length === 0 && !over) return null;
  const shown = found.filter((one) => !skip.includes(one.key.toLowerCase()));
  if (shown.length === 0 && !over && !onEdit) return null;
  return (
    <section className="flex flex-col gap-2">
      <div className="flex items-baseline justify-between gap-2">
        <h3 className="text-[10.5px] tracking-[0.07em] text-faint uppercase">{t("frontNamed")}</h3>
        {onEdit && (
          <button
            type="button"
            onClick={onEdit}
            className="text-[11.5px] text-faint hover:text-ink hover:underline"
          >
            {t("frontEdit")}
          </button>
        )}
      </div>
      {over && <p className="text-[12.5px] text-faint">{t("frontOvergrown")}</p>}
      {shown.length > 0 && (
        <dl className="flex flex-col gap-1.5 text-[12.5px]">
          {shown.map((one) => (
            <div key={one.key} className="flex items-baseline justify-between gap-2.5">
              <dt className="shrink-0 text-faint">{one.key}</dt>
              <dd className="min-w-0 truncate text-right text-ink" title={one.values.join(", ")}>
                {one.values.join(", ")}
              </dd>
            </div>
          ))}
        </dl>
      )}
    </section>
  );
}
