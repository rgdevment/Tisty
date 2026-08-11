import type { Task } from "./core";
import { bandOf, monthOf } from "./format";

export type Row =
  | { kind: "one"; key: string; task: Task; band: string }
  | { kind: "many"; key: string; title: string; band: string; tasks: Task[] };

/**
 * Fifty-two rows a year per habit is what turns the archive into noise, and the
 * archive by month is the screen the whole product is built around.
 */
export function grouped(tasks: Task[]): Row[] {
  const seen = new Map<string, Task[]>();
  const order: string[] = [];

  for (const task of tasks) {
    const month = monthOf(task.completed_at);
    // A separator no title can hold: «March 2025 informe» is both «March» plus
    // «2025 informe» and «March 2025» plus «informe».
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

/**
 * Labels each task with the day it belongs under, without reordering: the core
 * already sorts dated before undated, so a band never comes back twice.
 */
export function banded(tasks: Task[]): Row[] {
  return tasks.map((task) => ({
    kind: "one" as const,
    key: task.id,
    task,
    band: bandOf(task.date),
  }));
}
