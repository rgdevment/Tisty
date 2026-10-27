import type { List, Standing, Task } from "./core";
import { bandOf, daysFrom, monthOf } from "./format";
import { locale, t, type Word } from "./locales";
import { said } from "./quadrants";

export type Axis = "time" | "list" | "tag" | "quadrant";

export const AXES: Axis[] = ["time", "list", "tag", "quadrant"];

const STANDING: Record<Task["priority"], number> = {
  do: 0,
  decide: 1,
  delegate: 2,
  unset: 3,
  minor: 4,
};

export interface Row {
  kind: "one";
  key: string;
  task: Task;
  band: string;
}

export function monthly(tasks: Task[]): Row[] {
  return tasks.map((task) => ({
    kind: "one" as const,
    key: task.id,
    task,
    band: monthOf(task.completed_at),
  }));
}

export function shelved(tasks: Task[], axis: Axis, lists: List[]): Row[] {
  const named = (id?: string) => lists.find((one) => one.id === id)?.name;
  const rank = new Map<string, number>();
  const rows: Row[] = [];

  const put = (task: Task, band: string, order: number) => {
    if (!rank.has(band)) rank.set(band, order);
    rows.push({ kind: "one", key: `${band}\u0000${task.id}`, task, band });
  };

  for (const task of tasks) {
    if (axis === "list") {
      const name = named(task.list);
      const at = lists.findIndex((one) => one.id === task.list);
      put(task, name ?? t("noList"), name ? at : lists.length);
    } else if (axis === "quadrant") {
      put(task, said(task.priority), STANDING[task.priority]);
    } else if (!task.tags?.length) {
      put(task, t("noTags"), 1);
    } else {
      for (const tag of task.tags) put(task, `#${tag}`, 0);
    }
  }

  return rows.sort((one, two) => {
    const by = (rank.get(one.band) ?? 0) - (rank.get(two.band) ?? 0);
    return by !== 0 ? by : one.band.localeCompare(two.band, locale());
  });
}

export function regrouped(tasks: Task[], axis: Axis, lists: List[], now = new Date()): Row[] {
  const spoken = (task: Task) =>
    Boolean(task.resolved) &&
    task.status === "open" &&
    !(task.date && daysFrom(task.date.at, now) <= 0);
  const waiting: Row[] = tasks
    .filter(spoken)
    .map((task) => ({ kind: "one" as const, key: task.id, task, band: t("agentBand") }));
  return [
    ...shelved(
      tasks.filter((task) => !spoken(task)),
      axis,
      lists,
    ),
    ...waiting,
  ];
}

// What claims today, most urgent first; anything else keeps its day and the core's order.
const URGENT: Partial<Record<Standing, { rank: number; band: Word }>> = {
  overdue: { rank: 0, band: "overdue" },
  due_soon: { rank: 1, band: "dueSoonBand" },
  late: { rank: 2, band: "lateBand" },
  today: { rank: 3, band: "todayBand" },
  underway: { rank: 4, band: "underwayBand" },
};

export function banded(
  tasks: Task[],
  standings: Record<string, Standing> = {},
  now = new Date(),
): Row[] {
  const waiting: Row[] = [];
  const rest: (Row & { rank: number })[] = [];
  for (const task of tasks) {
    const urgent = URGENT[standings[task.id] ?? "ahead"];
    const spoken = Boolean(task.resolved) && task.status === "open" && !urgent;
    const row = {
      kind: "one" as const,
      key: task.id,
      task,
      band: spoken ? t("agentBand") : urgent ? t(urgent.band) : bandOf(task.date, now),
    };
    if (spoken) waiting.push(row);
    else rest.push({ ...row, rank: urgent?.rank ?? Object.keys(URGENT).length });
  }
  return [
    ...rest.sort((one, two) => one.rank - two.rank).map(({ rank, ...row }) => row),
    ...waiting,
  ];
}
