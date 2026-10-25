import { useAsked } from "../asked";
import { type Task, taskSeries } from "../core";
import { wroteAt } from "../format";
import { fill, t } from "../locales";
import Written from "./Written";

interface Props {
  task: Task;
  onError?: (problem: unknown) => void;
  onDoc?: (id: string) => void;
}

// A routine keeps one page for every round, so what each round left is read here, round by round.
export default function Rounds({ task, onError, onDoc }: Props) {
  const routine = Boolean(task.repeat || task.after);
  const told = useAsked(
    () => (routine ? taskSeries(task.id) : Promise.resolve(null)),
    [task.id, task.status, routine],
    onError,
  );
  const past = (told?.turns ?? []).filter((turn) => turn.status !== "open").reverse();
  if (past.length === 0) return null;
  return (
    <details className="mt-6 border-t border-hair pt-3">
      <summary className="cursor-pointer text-[10.5px] font-semibold tracking-[0.06em] text-faint uppercase select-none hover:text-soft">
        {t("rounds")} · {past.length}
      </summary>
      <ul className="mt-3 flex flex-col gap-3">
        {past.map((turn) => (
          <li key={turn.id}>
            <span className="flex items-baseline gap-1.5 text-[11.5px] tabular-nums text-faint">
              <span aria-hidden="true">{turn.status === "dropped" ? "⨯" : "✓"}</span>
              {turn.closed ? wroteAt(turn.closed, turn.zone) : ""}
              {turn.steps ? (
                <span>{fill("roundSteps", `${turn.steps_done ?? 0}`, `${turn.steps}`)}</span>
              ) : null}
            </span>
            {(turn.said ?? []).map((said, at) => (
              <Written
                // biome-ignore lint/suspicious/noArrayIndexKey: what a round said never moves.
                key={at}
                label={t("rounds")}
                onError={onError}
                onDoc={onDoc}
                text={said}
                className="prose text-[13px] leading-relaxed"
              />
            ))}
          </li>
        ))}
      </ul>
    </details>
  );
}
