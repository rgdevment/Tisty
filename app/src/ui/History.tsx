import type { Task } from "../core";
import { useSigning } from "../crediting";
import { wroteAt } from "../format";
import { t } from "../locales";
import Signature from "./Signature";
import Written from "./Written";

interface Props {
  task: Task;
  onError?: (problem: unknown) => void;
  onDoc?: (id: string) => void;
}

export default function History({ task, onError, onDoc }: Props) {
  const signing = useSigning("inside");
  const kept = (task.log ?? []).filter((entry) => entry.body.trim());
  if (kept.length === 0) return null;
  return (
    <details open className="mt-6 border-t border-hair pt-3">
      <summary className="cursor-pointer text-[10.5px] font-semibold tracking-[0.06em] text-faint uppercase select-none hover:text-soft">
        {t("history")} · {kept.length}
      </summary>
      <ul className="mt-3 flex flex-col gap-3">
        {kept.map((entry) => (
          <li key={entry.id}>
            <span className="flex items-baseline gap-1.5 text-[11.5px] tabular-nums text-faint">
              {wroteAt(entry.at, entry.tz)}
              <Signature by={entry.by} via={entry.via} signing={signing} />
            </span>
            <Written
              label={t("history")}
              onError={onError}
              onDoc={onDoc}
              text={entry.body}
              steps={task.steps?.map((one) => one.text)}
              className="prose text-[13px] leading-relaxed"
            />
          </li>
        ))}
      </ul>
    </details>
  );
}
