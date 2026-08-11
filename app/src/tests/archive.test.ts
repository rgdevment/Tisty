import { describe, expect, it } from "vitest";
import { grouped } from "../archive";
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
  it("folds what was done more than once in a month", () => {
    const rows = grouped([
      done("1", "sacar la basura", "2026-08-25T09:00:00Z"),
      done("2", "sacar la basura", "2026-08-18T09:00:00Z"),
      done("3", "sacar la basura", "2026-08-11T09:00:00Z"),
    ]);

    expect(rows).toHaveLength(1);
    expect(rows[0].kind).toBe("many");
    if (rows[0].kind === "many") expect(rows[0].tasks).toHaveLength(3);
  });

  /// The same habit in two months is two lines, not one of six.
  it("never folds across months", () => {
    const rows = grouped([
      done("1", "sacar la basura", "2026-09-01T09:00:00Z"),
      done("2", "sacar la basura", "2026-08-25T09:00:00Z"),
    ]);

    expect(rows).toHaveLength(2);
    expect(rows.every((row) => row.kind === "one")).toBe(true);
  });

  it("leaves a one-off alone", () => {
    const rows = grouped([
      done("1", "comprar pan", "2026-08-25T09:00:00Z"),
      done("2", "sacar la basura", "2026-08-18T09:00:00Z"),
      done("3", "sacar la basura", "2026-08-11T09:00:00Z"),
    ]);

    expect(rows.map((row) => row.kind)).toEqual(["one", "many"]);
  });

  /// The archive is ordered by when things closed, so repetitions of one month
  /// arrive with other work in between.
  it("gathers repetitions that are not next to each other", () => {
    const rows = grouped([
      done("1", "sacar la basura", "2026-08-25T09:00:00Z"),
      done("2", "pagar la luz", "2026-08-20T09:00:00Z"),
      done("3", "sacar la basura", "2026-08-11T09:00:00Z"),
    ]);

    expect(rows).toHaveLength(2);
    expect(rows[0].kind).toBe("many");
    expect(rows[1].kind).toBe("one");
  });

  /// Folding by title alone would put a bin from August under one from July.
  it("keeps the first closing of the group as its month", () => {
    const rows = grouped([done("1", "sacar la basura", "2026-08-25T09:00:00Z")]);
    expect(rows[0].month).toBe(grouped([done("2", "x", "2026-08-01T09:00:00Z")])[0].month);
  });

  /// «March» + «2025 informe» and «March 2025» + «informe» are the same string
  /// once joined by a space, and they are not the same group.
  it("does not confuse a title that starts with a year", () => {
    const rows = grouped([
      done("1", "2025 informe", "2026-03-10T09:00:00Z"),
      done("2", "informe", "2025-03-10T09:00:00Z"),
    ]);

    expect(rows).toHaveLength(2);
    expect(rows.every((row) => row.kind === "one")).toBe(true);
  });

  it("leaves out what never closed instead of piling it into one heap", () => {
    const rows = grouped([
      { ...done("1", "a", "2026-03-10T09:00:00Z"), completed_at: undefined },
      { ...done("2", "b", "2026-03-10T09:00:00Z"), completed_at: undefined },
    ] as never);

    expect(rows).toHaveLength(2);
  });
});
