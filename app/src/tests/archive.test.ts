import { describe, expect, it } from "vitest";
import { monthly } from "../archive";
import type { Task } from "../core";

const done = (id: string, title: string, at: string): Task =>
  ({
    id,
    title,
    status: "done",
    order: "a0",
    steps: [],
    journal: [],
    tags: [],
    created: at,
    completed_at: at,
  }) as unknown as Task;

describe("the archive by month", () => {
  it("gives every closing a row of its own, because each one happened", () => {
    const rows = monthly([
      done("1", "take the bins out", "2026-08-25T09:00:00Z"),
      done("2", "take the bins out", "2026-08-18T09:00:00Z"),
      done("3", "take the bins out", "2026-08-11T09:00:00Z"),
    ]);

    expect(rows).toHaveLength(3);
    expect(new Set(rows.map((row) => row.band)).size).toBe(1);
  });

  it("keeps two months apart", () => {
    const rows = monthly([
      done("1", "take the bins out", "2026-09-01T09:00:00Z"),
      done("2", "take the bins out", "2026-08-25T09:00:00Z"),
    ]);

    expect(rows[0].band).not.toBe(rows[1].band);
  });

  it("does not confuse a title that starts with a year", () => {
    const rows = monthly([
      done("1", "2025 report", "2026-03-10T09:00:00Z"),
      done("2", "report", "2025-03-10T09:00:00Z"),
    ]);

    expect(rows[0].band).not.toBe(rows[1].band);
  });

  it("gives what never closed a band of its own instead of a date it does not have", () => {
    const rows = monthly([
      { ...done("1", "a", "2026-03-10T09:00:00Z"), completed_at: undefined },
      done("2", "b", "2026-03-10T09:00:00Z"),
    ] as never);

    expect(rows[0].band).toBe("");
    expect(rows[1].band).not.toBe("");
  });

  it("keeps the order it was handed, which is the order the archive asked for", () => {
    const rows = monthly([
      done("1", "second", "2026-08-18T09:00:00Z"),
      done("2", "first", "2026-08-25T09:00:00Z"),
    ]);

    expect(rows.map((row) => row.task.id)).toEqual(["1", "2"]);
  });
});
