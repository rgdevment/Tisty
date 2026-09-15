import type { Task } from "../core";
import { stamped } from "../format";
import { fill, t } from "../locales";
import { agentNamed } from "../who";

export function spokenLabel(task: Task): string {
  const bits = [task.title];
  if (task.status !== "open") bits.push(t(task.status));
  if (task.resolved) bits.push(saidBy(task));
  return bits.join(" — ");
}

function saidBy(task: Task): string {
  const named = task.resolved ? agentNamed(task.resolved.by) : undefined;
  if (named) return fill("agentNamedSaidDone", named);
  return task.status === "open" ? t("agentSaidDone") : t("agentSettled");
}

export function Pip({ task }: { task: Task }) {
  if (!task.resolved) return null;
  return (
    <span
      aria-hidden="true"
      className="h-1.5 w-1.5 rounded-full bg-hue-teal"
      title={task.resolved ? fill("agentSaidWhen", stamped(task.resolved.at)) : undefined}
    />
  );
}

export function Lozenge({ task }: { task: Task }) {
  if (!task.resolved) return null;
  return (
    <span aria-hidden="true" className="text-hue-teal" title={saidBy(task)}>
      ◆
    </span>
  );
}
