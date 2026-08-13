import type { Task } from "./core";
import { bandOf, monthOf } from "./format";

export type Row =
  | { kind: "one"; key: string; task: Task; band: string }
  | { kind: "many"; key: string; title: string; band: string; tasks: Task[] };

export function grouped(tasks: Task[]): Row[] {
  const seen = new Map<string, Task[]>();
  const order: string[] = [];

  for (const task of tasks) {
    const month = monthOf(task.completed_at);
    const key = `${month}\u0000${task.title}`;
    const held = seen.get(key);
    if (held) {
      held.push(task);
    } else {
      seen.set(key, [task]);
      order.push(key);
    }
  }

  return order.map((key) => {
    const held = seen.get(key) as Task[];
    const band = monthOf(held[0].completed_at);
    return held.length === 1
      ? { kind: "one" as const, key: held[0].id, task: held[0], band }
      : { kind: "many" as const, key, title: held[0].title, band, tasks: held };
  });
}

export function banded(tasks: Task[]): Row[] {
  return tasks.map((task) => ({
    kind: "one" as const,
    key: task.id,
    task,
    band: bandOf(task.date),
  }));
}
